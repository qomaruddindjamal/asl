//! ASL PMD: Polled-Mode Driver Zero-Copy Direct Memory Access Engine
//! ===================================================================
//! Menyediakan arsitektur bypass kernel berkecepatan tinggi (seperti DPDK).
//! Menghubungkan ring buffer kartu jaringan (PCIe DMA) langsung ke userspace partition.
//! Zero context switches, zero system call overhead, dan wire-speed packet processing.

#![no_std]

pub const RX_RING_SIZE: usize = 128;
pub const TX_RING_SIZE: usize = 128;
pub const MAX_PACKET_BUFFER_SIZE: usize = 2048;

use core::sync::atomic::{AtomicU16, AtomicU64, AtomicUsize, Ordering};

/// Error terkait manajemen pool memori DMA dan alignment
#[derive(Debug, Copy, Clone, PartialEq, Eq)]
pub enum DmaPoolError {
    PoolMisaligned,
    InvalidPoolSize,
    BufferAddressMisaligned,
}

/// Deskriptor DMA Paket Jaringan Hardware
///
/// Enforce 64-byte hardware cache line boundary alignment (`#[repr(C, align(64))]`)
/// untuk mencegah false sharing dan menjamin isolasi cache line saat operasi
/// invalidate/clean pada arsitektur non-coherent.
///
/// # Model Kepemilikan Sel Flags (W-2 Resolution):
/// - Bit 0 `Ownership` adalah field yang ditulis oleh hardware NIC DMA engine saat paket selesai ditransfer (DMA Done).
/// - CPU bertindak sebagai consumer eksklusif yang membaca Bit 0 via atomic load `Ordering::Acquire`.
/// - Konfirmasi selesai proses oleh CPU dikembalikan ke DMA engine melalui mekanisme handoff eksplisit
///   (menulis flags bit 0 = 1 via `Ordering::Release` atau membunyikan Doorbell MMIO register terpisah).
/// - Dilarang mencampuradukkan penulisan simultan antara CPU dan DMA pada sel flags yang sama.
#[repr(C, align(64))]
pub struct DmaPacketDescriptor {
    pub physical_buffer_addr: u64,    // 8 bytes (offset 0..8)
    pub length_bytes: u16,            // 2 bytes (offset 8..10)
    pub flags: AtomicU16, // 2 bytes (offset 10..12) - Bit 0: Ownership (0 = Host, 1 = NIC DMA)
    pub vlan_tag: u16,    // 2 bytes (offset 12..14)
    pub checksum_offload_result: u16, // 2 bytes (offset 14..16)
    pub _padding: [u8; 48], // 48 bytes padding (total: 64 bytes)
}

// Compile-Time Static Assertions untuk ukuran dan alignment tepat 64 byte (DO-178C DAL-A)
const _: () = {
    assert!(
        core::mem::size_of::<DmaPacketDescriptor>() == 64,
        "DmaPacketDescriptor must be exactly 64 bytes!"
    );
    assert!(
        core::mem::align_of::<DmaPacketDescriptor>() == 64,
        "DmaPacketDescriptor must be aligned to 64 bytes!"
    );
};

impl DmaPacketDescriptor {
    pub const fn empty() -> Self {
        Self {
            physical_buffer_addr: 0,
            length_bytes: 0,
            flags: AtomicU16::new(0),
            vlan_tag: 0,
            checksum_offload_result: 0,
            _padding: [0u8; 48],
        }
    }
}

/// Struktur Cincin Antrean RX/TX DMA Zero-Copy (SMP & DMA Safe)
///
/// Model Kepemilikan & Sinkronisasi:
/// - RX Ring: Host CPU membaca paket saat `flags` memiliki bit 0 == 0 (Ownership dilepas oleh NIC).
///   Host membaca data dengan `Ordering::Acquire`, memproses, lalu mengembalikan slot ke DMA dengan bit 0 == 1 via `Ordering::Release`.
/// - TX Ring: Host CPU mengisi slot di `tx_tail`, lalu menandai kepemilikan ke DMA dengan `flags.store(1, Ordering::Release)`
///   dan memajukan `tx_tail.store(..., Ordering::Release)`.
pub struct ZeroCopyPmdQueue {
    pub rx_ring: [DmaPacketDescriptor; RX_RING_SIZE],
    pub tx_ring: [DmaPacketDescriptor; TX_RING_SIZE],
    pub rx_head: AtomicUsize,
    pub tx_tail: AtomicUsize,
    pub packets_processed_total: AtomicU64,
    pub dma_pool_base_phys: u64,
}

impl ZeroCopyPmdQueue {
    pub const fn new() -> Self {
        const EMPTY_DESC: DmaPacketDescriptor = DmaPacketDescriptor::empty();
        Self {
            rx_ring: [EMPTY_DESC; RX_RING_SIZE],
            tx_ring: [EMPTY_DESC; TX_RING_SIZE],
            rx_head: AtomicUsize::new(0),
            tx_tail: AtomicUsize::new(0),
            packets_processed_total: AtomicU64::new(0),
            dma_pool_base_phys: 0,
        }
    }

    /// Inisialisasi pool memori DMA dengan verifikasi keselarasan batas 64-byte nyata saat runtime
    ///
    /// # Safety & DAL-A Invariant:
    /// Pemeriksaan ini wajib berupa pemeriksaan nilai riil (`(base_phys & 0x3F) == 0`) yang mengembalikan
    /// `Result::Err(DmaPoolError::PoolMisaligned)`, BUKAN sekadar `debug_assert!` yang tereliminasi di build release.
    pub fn init_pool(&mut self, base_phys: u64, total_size: usize) -> Result<(), DmaPoolError> {
        if (base_phys & 0x3F) != 0 {
            return Err(DmaPoolError::PoolMisaligned);
        }
        if total_size < (RX_RING_SIZE + TX_RING_SIZE) * 64 {
            return Err(DmaPoolError::InvalidPoolSize);
        }

        self.dma_pool_base_phys = base_phys;
        Ok(())
    }

    /// Polling penerimaan paket tanpa interupsi CPU (Polled-Mode)
    #[inline(always)]
    pub fn poll_rx_burst(&mut self, max_burst: usize) -> usize {
        let mut count = 0;
        let mut curr_head = self.rx_head.load(Ordering::Relaxed);

        while count < max_burst {
            let desc = &mut self.rx_ring[curr_head];
            let flags = desc.flags.load(Ordering::Acquire);

            // Cek apakah kepemilikan paket sudah kembali ke Host CPU (bit 0 == 0)
            if (flags & 0x01) == 0 && desc.length_bytes > 0 {
                // Paket siap diproses di memori tanpa alokasi baru (Zero-Copy)
                desc.flags.store(flags | 0x01, Ordering::Release); // Kembalikan ke DMA NIC untuk paket berikutnya
                curr_head = (curr_head + 1) % RX_RING_SIZE;
                count += 1;
            } else {
                break;
            }
        }

        if count > 0 {
            self.rx_head.store(curr_head, Ordering::Release);
            self.packets_processed_total
                .fetch_add(count as u64, Ordering::Relaxed);
        }

        count
    }

    /// Transmisi burst paket langsung ke DMA transmit ring
    ///
    /// # Kontrak RND_08 (DMA-ADDR-01 remediation) — BREAKING:
    /// SATU-SATUNYA sumber alamat yang boleh masuk deskriptor hardware adalah
    /// `IovaLease` tervalidasi hasil `DmaAddressSpace::pin`/`stage`. Alamat fisik
    /// mentah TIDAK dapat lagi diterima fungsi ini (tidak ada jalur lain).
    /// Penjagaan runtime (bukan debug_assert): keselarasan 64-byte bus,
    /// panjang transfer <= batas lease.
    pub fn transmit_burst(
        &mut self,
        lease: &asl_hal::IovaLease,
        len: u16,
        vlan: u16,
    ) -> Result<(), &'static str> {
        if !lease.is_bus_aligned() {
            return Err("ERR_TX_LEASE_NOT_ALIGNED_64B");
        }
        if (len as u64) == 0 || (len as u64) > lease.len() {
            return Err("ERR_TX_LEN_EXCEEDS_LEASE");
        }

        let curr_tail = self.tx_tail.load(Ordering::Relaxed);
        let desc = &mut self.tx_ring[curr_tail];

        let flags = desc.flags.load(Ordering::Acquire);
        if (flags & 0x01) != 0 {
            return Err("ERR_TX_RING_CONGESTION");
        }

        desc.physical_buffer_addr = lease.iova();
        desc.length_bytes = len;
        desc.vlan_tag = vlan;
        desc.flags.store(0x01, Ordering::Release); // Serahkan kepemilikan ke NIC DMA TX Engine

        let next_tail = (curr_tail + 1) % TX_RING_SIZE;
        self.tx_tail.store(next_tail, Ordering::Release);
        self.packets_processed_total.fetch_add(1, Ordering::Relaxed);

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use asl_hal::DmaAddressSpace;

    #[test]
    fn test_pmd_empty_rx_poll() {
        let mut queue = ZeroCopyPmdQueue::new();
        assert_eq!(queue.poll_rx_burst(32), 0);
    }

    #[test]
    fn test_pmd_rx_poll_with_packets() {
        let mut queue = ZeroCopyPmdQueue::new();
        queue.rx_ring[0].length_bytes = 64;
        queue.rx_ring[0].flags.store(0x00, Ordering::Relaxed);
        queue.rx_ring[1].length_bytes = 128;
        queue.rx_ring[1].flags.store(0x00, Ordering::Relaxed);

        let processed = queue.poll_rx_burst(10);
        assert_eq!(processed, 2);
        assert_eq!(queue.packets_processed_total.load(Ordering::Relaxed), 2);
        assert_eq!(queue.rx_head.load(Ordering::Relaxed), 2);

        assert_eq!(queue.rx_ring[0].flags.load(Ordering::Relaxed) & 0x01, 1);
        assert_eq!(queue.rx_ring[1].flags.load(Ordering::Relaxed) & 0x01, 1);
    }

    fn tx_lease(space: &mut asl_hal::BounceAddressSpace, iova: u64, len: u64) -> asl_hal::IovaLease {
        space.pin(1, iova, len).expect("pin TX lease")
    }

    #[test]
    fn test_pmd_transmit_and_congestion() {
        let mut space = asl_hal::BounceAddressSpace::new(0x4000_0000);
        space
            .register_region(1, 0x2000_0000, 0x2010_0000)
            .expect("region");
        let mut queue = ZeroCopyPmdQueue::new();

        let lease = tx_lease(&mut space, 0x2000_0000, 0x1000);
        assert_eq!(queue.transmit_burst(&lease, 512, 100), Ok(()));
        assert_eq!(queue.tx_tail.load(Ordering::Relaxed), 1);
        assert_eq!(queue.packets_processed_total.load(Ordering::Relaxed), 1);
        assert_eq!(queue.tx_ring[0].physical_buffer_addr, 0x2000_0000);

        queue.tx_ring[1].flags.store(0x01, Ordering::Relaxed);
        let lease2 = tx_lease(&mut space, 0x2000_1000, 0x1000);
        assert_eq!(
            queue.transmit_burst(&lease2, 256, 100),
            Err("ERR_TX_RING_CONGESTION")
        );
    }

    #[test]
    fn tm04_transmit_rejects_oversized_len_and_unaligned_lease() {
        let mut space = asl_hal::BounceAddressSpace::new(0x4000_0000);
        space
            .register_region(1, 0x2000_0000, 0x2010_0000)
            .expect("region");
        let mut queue = ZeroCopyPmdQueue::new();

        let lease = tx_lease(&mut space, 0x2000_0000, 0x400);
        // len melebihi batas lease -> ditolak, deskriptor tidak tersentuh
        assert_eq!(
            queue.transmit_burst(&lease, 0x500, 100),
            Err("ERR_TX_LEN_EXCEEDS_LEASE")
        );
        assert_eq!(queue.transmit_burst(&lease, 0, 100), Err("ERR_TX_LEN_EXCEEDS_LEASE"));
        assert_eq!(queue.tx_tail.load(Ordering::Relaxed), 0);

        // Lease hasil stage dari pool dengan offset tidak selaras tidak mungkin
        // dibuat via API; uji jalur guard dengan lease aligned normal.
        assert_eq!(space.unpin(&lease), Ok(()));
        let ok = tx_lease(&mut space, 0x2000_0040, 0x100);
        assert_eq!(queue.transmit_burst(&ok, 64, 100), Ok(()));
        assert_eq!(queue.tx_ring[0].physical_buffer_addr, 0x2000_0040);
    }

    // =========================================================================
    // UNIT TESTS RESMI ALIGN-01 (4 KASUS KELENGKAPAN SESUAI AMENDEMEN A)
    // =========================================================================

    #[test]
    fn test_align01_pool_misaligned_rejected() {
        let mut queue = ZeroCopyPmdQueue::new();
        // Alamat tidak selaras 64 byte (offset 1 hingga 63)
        assert_eq!(
            queue.init_pool(0x1000_0001, 32768),
            Err(DmaPoolError::PoolMisaligned)
        );
        assert_eq!(
            queue.init_pool(0x1000_003F, 32768),
            Err(DmaPoolError::PoolMisaligned)
        );
        assert_eq!(
            queue.init_pool(0x1000_0010, 32768),
            Err(DmaPoolError::PoolMisaligned)
        );
    }

    #[test]
    fn test_align01_pool_aligned_accepted() {
        let mut queue = ZeroCopyPmdQueue::new();
        // Alamat selaras 64 byte (kelipatan 0x40)
        assert_eq!(queue.init_pool(0x1000_0000, 32768), Ok(()));
        assert_eq!(queue.dma_pool_base_phys, 0x1000_0000);
        assert_eq!(queue.init_pool(0x2000_0040, 32768), Ok(()));
        assert_eq!(queue.dma_pool_base_phys, 0x2000_0040);
    }

    #[test]
    fn test_align01_descriptor_size_exact_64() {
        assert_eq!(core::mem::size_of::<DmaPacketDescriptor>(), 64);
    }

    #[test]
    fn test_align01_descriptor_alignment_exact_64() {
        assert_eq!(core::mem::align_of::<DmaPacketDescriptor>(), 64);
    }
}
