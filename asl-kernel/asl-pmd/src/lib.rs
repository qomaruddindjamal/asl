//! ASL PMD: Polled-Mode Driver Zero-Copy Direct Memory Access Engine
//! ===================================================================
//! Menyediakan arsitektur bypass kernel berkecepatan tinggi (seperti DPDK).
//! Menghubungkan ring buffer kartu jaringan (PCIe DMA) langsung ke userspace partition.
//! Zero context switches, zero system call overhead, dan wire-speed packet processing.

#![no_std]

pub const RX_RING_SIZE: usize = 128;
pub const TX_RING_SIZE: usize = 128;
pub const MAX_PACKET_BUFFER_SIZE: usize = 2048;

use core::sync::atomic::{AtomicU16, AtomicUsize, AtomicU64, Ordering};

/// Deskriptor DMA Paket Jaringan Hardware
#[repr(C, align(64))]
pub struct DmaPacketDescriptor {
    pub physical_buffer_addr: u64,
    pub length_bytes: u16,
    pub flags: AtomicU16, // Bit 0: Ownership (0 = Host, 1 = NIC DMA), Bit 1: EOP, Bit 2: Error
    pub vlan_tag: u16,
    pub checksum_offload_result: u16,
}

impl DmaPacketDescriptor {
    pub const fn empty() -> Self {
        Self {
            physical_buffer_addr: 0,
            length_bytes: 0,
            flags: AtomicU16::new(0),
            vlan_tag: 0,
            checksum_offload_result: 0,
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
        }
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
            self.packets_processed_total.fetch_add(count as u64, Ordering::Relaxed);
        }

        count
    }

    /// Transmisi burst paket langsung ke DMA transmit ring
    pub fn transmit_burst(&mut self, phys_addr: u64, len: u16, vlan: u16) -> Result<(), &'static str> {
        let curr_tail = self.tx_tail.load(Ordering::Relaxed);
        let desc = &mut self.tx_ring[curr_tail];

        let flags = desc.flags.load(Ordering::Acquire);
        if (flags & 0x01) != 0 {
            return Err("ERR_TX_RING_CONGESTION");
        }

        desc.physical_buffer_addr = phys_addr;
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

    #[test]
    fn test_pmd_empty_rx_poll() {
        let mut queue = ZeroCopyPmdQueue::new();
        // Semua descriptor awalnya length_bytes = 0, sehingga burst poll harus 0
        assert_eq!(queue.poll_rx_burst(32), 0);
    }

    #[test]
    fn test_pmd_rx_poll_with_packets() {
        let mut queue = ZeroCopyPmdQueue::new();
        // Simulasikan NIC DMA mengisi 2 paket (ownership flag bit 0 = 0)
        queue.rx_ring[0].length_bytes = 64;
        queue.rx_ring[0].flags.store(0x00, Ordering::Relaxed);
        queue.rx_ring[1].length_bytes = 128;
        queue.rx_ring[1].flags.store(0x00, Ordering::Relaxed);

        let processed = queue.poll_rx_burst(10);
        assert_eq!(processed, 2);
        assert_eq!(queue.packets_processed_total.load(Ordering::Relaxed), 2);
        assert_eq!(queue.rx_head.load(Ordering::Relaxed), 2);

        // Setelah diproses, ownership dikembalikan ke NIC (bit 0 = 1)
        assert_eq!(queue.rx_ring[0].flags.load(Ordering::Relaxed) & 0x01, 1);
        assert_eq!(queue.rx_ring[1].flags.load(Ordering::Relaxed) & 0x01, 1);
    }

    #[test]
    fn test_pmd_transmit_and_congestion() {
        let mut queue = ZeroCopyPmdQueue::new();
        assert_eq!(queue.transmit_burst(0x2000_0000, 512, 100), Ok(()));
        assert_eq!(queue.tx_tail.load(Ordering::Relaxed), 1);
        assert_eq!(queue.packets_processed_total.load(Ordering::Relaxed), 1);

        // Slot 1 yang baru dialokasikan masih dimiliki NIC jika flags bit 0 == 1
        queue.tx_ring[1].flags.store(0x01, Ordering::Relaxed);
        assert_eq!(queue.transmit_burst(0x2000_1000, 256, 100), Err("ERR_TX_RING_CONGESTION"));
    }
}

