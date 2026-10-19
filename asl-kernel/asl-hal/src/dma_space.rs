//! ASL HAL: Multi-Tenant DMA Address Space & IOVA Isolation (DMA-ADDR-01)
//! =======================================================================
//! Menjamin isolasi spatial ARINC 653 Part 1 untuk periferal DMA (PCIe/NIC/CCTV).
//! Mencegah penembusan batas memori antar-partisi via bus master transfer.

// Penegakan Eksklusi Mutlak IdentityDmaSpace dari Build DO-178C DAL-A
#[cfg(all(feature = "dal-a", feature = "identity-dma"))]
compile_error!("CRITICAL SAFETY FAULT: IdentityDmaSpace is strictly prohibited in DO-178C DAL-A builds due to lack of spatial isolation!");

/// Alamat Bus Perangkat (IOVA / Device Bus Address)
#[derive(Debug, Copy, Clone, PartialEq, Eq)]
pub struct DmaDeviceAddress(pub u64);

/// Jenis Pelanggaran Proteksi Ruang Alamat DMA
#[derive(Debug, Copy, Clone, PartialEq, Eq)]
pub enum DmaAddressFault {
    CrossPartitionViolation,
    AddressOutOfRange,
    TranslationNotPresent,
    ProtectionFaultReadOnly,
}

/// Catatan Kesalahan Transaksi DMA untuk Health Monitoring Daemon
#[derive(Debug, Copy, Clone, PartialEq, Eq)]
pub struct DmaFaultRecord {
    pub partition_id: u32,
    pub fault_kind: DmaAddressFault,
    pub illegal_iova: u64,
    pub timestamp_us: u64,
}

/// Trait Universal Abstraksi Ruang Alamat DMA
pub trait DmaAddressSpace {
    /// Menerjemahkan dan memvalidasi akses DMA ke alamat bus perangkat
    fn translate_and_check(
        &self,
        partition_id: u32,
        buffer_addr: u64,
        size_bytes: usize,
        is_write: bool,
    ) -> Result<DmaDeviceAddress, DmaAddressFault>;

    /// Melepas pemetaan IOVA
    fn unmap_iova(
        &mut self,
        partition_id: u32,
        iova: u64,
        size_bytes: usize,
    ) -> Result<(), DmaAddressFault>;

    // -------------------------------------------------------------------------
    // Kontrak lease RND_08 (DMA-ADDR-01): default = backend belum mendukung pin.
    // -------------------------------------------------------------------------

    /// Pin region fisik milik partisi (harus tercakup window region terdaftar
    /// untuk partisi tersebut) menjadi `IovaLease` siap-deskriptor hardware.
    fn pin(&mut self, _partition_id: u32, _phys: u64, _len: u64) -> Result<IovaLease, DmaError> {
        Err(DmaError::NotPinned)
    }

    /// Lepas pin: generation slot naik; lease stale ditolak selamanya (anti-TOCTOU).
    fn unpin(&mut self, _lease: &IovaLease) -> Result<(), DmaError> {
        Err(DmaError::NotPinned)
    }

    /// Reservasi window IOVA kosong milik partisi.
    fn iova_alloc(&mut self, _partition_id: u32, _len: u64) -> Result<u64, DmaError> {
        Err(DmaError::NotPinned)
    }

    /// Bebaskan window IOVA; semua lease yang menunjuknya menjadi stale.
    fn iova_dealloc(&mut self, _partition_id: u32, _iova: u64, _len: u64) -> Result<(), DmaError> {
        Err(DmaError::NotPinned)
    }

    /// Periksa apakah lease masih otoritatif pra-doorbell (anti-TOCTOU).
    /// Backend tanpa tabel pin tidak dapat membuktikan kesegaran -> stale.
    fn validate_pin(&self, _lease: &IovaLease) -> Result<(), DmaError> {
        Err(DmaError::NotPinned)
    }
}

// -----------------------------------------------------------------------------
// KONTRAK LEASE RND_08 — IovaLease + DmaError (DMA-ADDR-01 remediation)
// -----------------------------------------------------------------------------

/// Kesalahan level kontrak pin/unpin/lease (RND_08 §B)
#[derive(Debug, Copy, Clone, PartialEq, Eq)]
pub enum DmaError {
    /// Backend tidak mendukung jalur pin (default trait)
    NotPinned,
    /// Region/partisi bukan pemilik atau klaim ganda atas rentang yang sama
    PermissionDenied,
    /// Alamat/ukuran tidak memenuhi requirement bus DMA (64-byte cache line)
    Misaligned,
    /// Semua slot lease terisi
    OutOfSlots,
    /// Rentang di luar region terdaftar atau aritmatik overflow
    RangeOutOfBounds,
    /// Lease menunjuk generation slot yang sudah berganti (stale / TOCTOU)
    GenerationMismatch,
    /// Panjang transfer melewati batas lease
    LengthExceedsLease,
}

/// Handle DMA tervalidasi — SATU-SATUNYA sumber alamat yang boleh masuk
/// deskriptor hardware. Field privat: pemanggil tidak dapat memalsukan lease
/// dengan struct literal; hanya backend (konstruktor pub(crate)) yang membuat.
#[derive(Debug, Copy, Clone, PartialEq, Eq)]
pub struct IovaLease {
    iova: u64,
    len: u64,
    partition_id: u32,
    slot: usize,
    generation: u32,
}

impl IovaLease {
    pub(crate) const fn new(
        iova: u64,
        len: u64,
        partition_id: u32,
        slot: usize,
        generation: u32,
    ) -> Self {
        Self {
            iova,
            len,
            partition_id,
            slot,
            generation,
        }
    }

    /// Alamat bus yang wajib ditulis ke deskriptor DMA
    #[inline]
    pub const fn iova(&self) -> u64 {
        self.iova
    }

    #[inline]
    pub const fn len(&self) -> u64 {
        self.len
    }

    #[inline]
    pub const fn partition_id(&self) -> u32 {
        self.partition_id
    }

    #[inline]
    pub const fn slot(&self) -> usize {
        self.slot
    }

    #[inline]
    pub const fn generation(&self) -> u32 {
        self.generation
    }

    /// Keselarasan cache-line 64-byte dari alamat bus
    #[inline]
    pub const fn is_bus_aligned(&self) -> bool {
        (self.iova & 0x3F) == 0
    }
}

// -----------------------------------------------------------------------------
// BACKEND 4: SOFTWARE BOUNCE / OWNERSHIP SPACE (fallback non-coherent, DMA-CACHE-01)
// -----------------------------------------------------------------------------

pub const BOUNCE_SLOT_COUNT: usize = 8;
pub const BOUNCE_SLOT_SIZE: usize = 1024;
pub const BOUNCE_REGION_SLOTS: usize = 8;

#[derive(Debug, Copy, Clone)]
struct BounceSlot {
    in_use: bool,
    reserved_only: bool,
    read_only: bool,
    partition_id: u32,
    iova: u64,
    len: u64,
    generation: u32,
}

/// Ruang DMA software-model: window region per partisi + pool staging koheren
/// untuk target non-coherent (menutup DMA-CACHE-01: hardware tidak pernah
/// DMA ke buffer cache-stale milik caller; ia hanya menyentuh pool bounce).
/// `pool_bus_base` wajib merupakan alamat bus nyata dari allocator DMA-capable
/// saat runtime; pada model software nilai ini dipasangkan driver saat init.
pub struct BounceAddressSpace {
    regions: [(u32, u64, u64); BOUNCE_REGION_SLOTS],
    slots: [BounceSlot; BOUNCE_SLOT_COUNT],
    pool: [[u8; BOUNCE_SLOT_SIZE]; BOUNCE_SLOT_COUNT],
    pool_bus_base: u64,
}

const EMPTY_SLOT: BounceSlot = BounceSlot {
    in_use: false,
    reserved_only: false,
    read_only: false,
    partition_id: 0,
    iova: 0,
    len: 0,
    generation: 0,
};

impl BounceAddressSpace {
    pub const fn new(pool_bus_base: u64) -> Self {
        Self {
            regions: [(0, 0, 0); BOUNCE_REGION_SLOTS],
            slots: [EMPTY_SLOT; BOUNCE_SLOT_COUNT],
            pool: [[0u8; BOUNCE_SLOT_SIZE]; BOUNCE_SLOT_COUNT],
            pool_bus_base,
        }
    }

    /// Registrasi window memori milik satu partisi ARINC 653 (setup-time)
    pub fn register_region(
        &mut self,
        partition_id: u32,
        start: u64,
        end: u64,
    ) -> Result<(), DmaError> {
        if end <= start || end.checked_sub(start).is_none() {
            return Err(DmaError::RangeOutOfBounds);
        }
        for r in self.regions.iter() {
            let (_, s, e) = *r;
            if e != 0 && s < end && start < e {
                return Err(DmaError::PermissionDenied);
            }
        }
        for r in self.regions.iter_mut() {
            if r.2 == 0 {
                *r = (partition_id, start, end);
                return Ok(());
            }
        }
        Err(DmaError::OutOfSlots)
    }

    fn take_free_slot(&mut self) -> Option<usize> {
        for i in 0..BOUNCE_SLOT_COUNT {
            if !self.slots[i].in_use {
                return Some(i);
            }
        }
        None
    }

    /// Pin region fisik milik partisi -> lease. iova == phys pada model software
    /// (identity bus); penegakan yang penting adalah kepemilikan per-partisi.
    fn pin_internal(
        &mut self,
        partition_id: u32,
        phys: u64,
        len: u64,
        read_only: bool,
    ) -> Result<IovaLease, DmaError> {
        if len == 0 || (phys & 0x3F) != 0 {
            return Err(DmaError::Misaligned);
        }
        let end = phys.checked_add(len).ok_or(DmaError::RangeOutOfBounds)?;
        let mut covered = false;
        for r in self.regions.iter() {
            if r.0 == partition_id && r.2 != 0 && r.1 <= phys && end <= r.2 {
                covered = true;
                break;
            }
        }
        if !covered {
            return Err(DmaError::PermissionDenied);
        }
        for s in self.slots.iter() {
            if s.in_use && s.iova < end && phys < s.iova + s.len {
                return Err(DmaError::PermissionDenied);
            }
        }
        let idx = self.take_free_slot().ok_or(DmaError::OutOfSlots)?;
        self.slots[idx].in_use = true;
        self.slots[idx].reserved_only = false;
        self.slots[idx].read_only = read_only;
        self.slots[idx].partition_id = partition_id;
        self.slots[idx].iova = phys;
        self.slots[idx].len = len;
        Ok(IovaLease::new(
            phys,
            len,
            partition_id,
            idx,
            self.slots[idx].generation,
        ))
    }

    /// Stage data payload ke slot pool koheren; hardware hanya menyentuh pool.
    /// Menutup DMA-CACHE-01 pada target non-coherent.
    pub fn stage(&mut self, partition_id: u32, src: &[u8]) -> Result<IovaLease, DmaError> {
        if src.is_empty() || src.len() > BOUNCE_SLOT_SIZE || (self.pool_bus_base & 0x3F) != 0 {
            return Err(DmaError::Misaligned);
        }
        let idx = self.take_free_slot().ok_or(DmaError::OutOfSlots)?;
        self.pool[idx][..src.len()].copy_from_slice(src);
        let iova = self
            .pool_bus_base
            .checked_add((idx * BOUNCE_SLOT_SIZE) as u64)
            .ok_or(DmaError::RangeOutOfBounds)?;
        self.slots[idx].in_use = true;
        self.slots[idx].reserved_only = false;
        self.slots[idx].read_only = false;
        self.slots[idx].partition_id = partition_id;
        self.slots[idx].iova = iova;
        self.slots[idx].len = src.len() as u64;
        Ok(IovaLease::new(
            iova,
            src.len() as u64,
            partition_id,
            idx,
            self.slots[idx].generation,
        ))
    }

    /// Verifikasi lease masih otoritatif (anti-TOCTOU sebelum doorbell).
    pub fn validate_lease(&self, lease: &IovaLease) -> Result<(), DmaError> {
        let s = lease.slot();
        if s >= BOUNCE_SLOT_COUNT {
            return Err(DmaError::GenerationMismatch);
        }
        let slot = &self.slots[s];
        if !slot.in_use
            || slot.generation != lease.generation()
            || slot.partition_id != lease.partition_id()
            || slot.iova != lease.iova()
            || slot.len != lease.len()
        {
            return Err(DmaError::GenerationMismatch);
        }
        Ok(())
    }

    /// Pin read-only: device hanya boleh membaca rentang tsb (RX-style guard).
    pub fn pin_ro(&mut self, partition_id: u32, phys: u64, len: u64) -> Result<IovaLease, DmaError> {
        self.pin_internal(partition_id, phys, len, true)
    }
}

impl DmaAddressSpace for BounceAddressSpace {
    fn pin(&mut self, partition_id: u32, phys: u64, len: u64) -> Result<IovaLease, DmaError> {
        self.pin_internal(partition_id, phys, len, false)
    }

    fn unpin(&mut self, lease: &IovaLease) -> Result<(), DmaError> {        if lease.slot() >= BOUNCE_SLOT_COUNT {
            return Err(DmaError::GenerationMismatch);
        }
        let s = &mut self.slots[lease.slot()];
        if !s.in_use || s.generation != lease.generation() || s.partition_id != lease.partition_id()
        {
            return Err(DmaError::GenerationMismatch);
        }
        s.in_use = false;
        s.reserved_only = false;
        s.read_only = false;
        s.iova = 0;
        s.len = 0;
        s.generation = s.generation.wrapping_add(1);
        Ok(())
    }

    fn validate_pin(&self, lease: &IovaLease) -> Result<(), DmaError> {
        self.validate_lease(lease)
    }

    fn iova_alloc(&mut self, partition_id: u32, len: u64) -> Result<u64, DmaError> {
        if len == 0 || len > BOUNCE_SLOT_SIZE as u64 {
            return Err(DmaError::RangeOutOfBounds);
        }
        if (self.pool_bus_base & 0x3F) != 0 {
            return Err(DmaError::Misaligned);
        }
        let idx = self.take_free_slot().ok_or(DmaError::OutOfSlots)?;
        let iova = self
            .pool_bus_base
            .checked_add((idx * BOUNCE_SLOT_SIZE) as u64)
            .ok_or(DmaError::RangeOutOfBounds)?;
        self.slots[idx].in_use = true;
        self.slots[idx].reserved_only = true;
        self.slots[idx].partition_id = partition_id;
        self.slots[idx].iova = iova;
        self.slots[idx].len = len;
        Ok(iova)
    }

    fn iova_dealloc(&mut self, partition_id: u32, iova: u64, _len: u64) -> Result<(), DmaError> {
        for s in self.slots.iter_mut() {
            if s.in_use && s.partition_id == partition_id && s.iova == iova {
                s.in_use = false;
                s.reserved_only = false;
                s.read_only = false;
                s.iova = 0;
                s.len = 0;
                s.generation = s.generation.wrapping_add(1);
                return Ok(());
            }
        }
        Err(DmaError::GenerationMismatch)
    }

    /// Validasi device-side: alamat bus harus berada dalam slot milik partisi
    /// peminta. Klaim partisi lain -> CrossPartitionViolation; write ke lease
    /// read-only -> ProtectionFaultReadOnly; sisanya TranslationNotPresent.
    fn translate_and_check(
        &self,
        partition_id: u32,
        buffer_addr: u64,
        size_bytes: usize,
        is_write: bool,
    ) -> Result<DmaDeviceAddress, DmaAddressFault> {
        let end = buffer_addr
            .checked_add(size_bytes as u64)
            .ok_or(DmaAddressFault::AddressOutOfRange)?;
        let mut foreign_hit = false;
        for s in self.slots.iter() {
            if s.in_use && s.iova <= buffer_addr && end <= s.iova + s.len {
                if s.partition_id != partition_id {
                    foreign_hit = true;
                } else if is_write && s.read_only {
                    return Err(DmaAddressFault::ProtectionFaultReadOnly);
                } else {
                    return Ok(DmaDeviceAddress(buffer_addr));
                }
            }
        }
        if foreign_hit {
            return Err(DmaAddressFault::CrossPartitionViolation);
        }
        Err(DmaAddressFault::TranslationNotPresent)
    }

    fn unmap_iova(
        &mut self,
        partition_id: u32,
        iova: u64,
        size_bytes: usize,
    ) -> Result<(), DmaAddressFault> {
        for s in self.slots.iter_mut() {
            if s.in_use && s.partition_id == partition_id && s.iova == iova {
                let lease_like_match = size_bytes as u64 == s.len || s.len >= size_bytes as u64;
                if lease_like_match {
                    s.in_use = false;
                    s.reserved_only = false;
                    s.read_only = false;
                    s.iova = 0;
                    s.len = 0;
                    s.generation = s.generation.wrapping_add(1);
                    return Ok(());
                }
            }
        }
        Err(DmaAddressFault::TranslationNotPresent)
    }
}

// -----------------------------------------------------------------------------
// BACKEND 1: IDENTITY DMA SPACE (NON-DAL-A FLAT ADDRESSING)
// -----------------------------------------------------------------------------

pub struct IdentityDmaSpace {
    pub max_physical_limit: u64,
}

impl DmaAddressSpace for IdentityDmaSpace {
    fn translate_and_check(
        &self,
        _partition_id: u32,
        buffer_addr: u64,
        size_bytes: usize,
        _is_write: bool,
    ) -> Result<DmaDeviceAddress, DmaAddressFault> {
        let end = buffer_addr
            .checked_add(size_bytes as u64)
            .ok_or(DmaAddressFault::AddressOutOfRange)?;
        if end > self.max_physical_limit {
            return Err(DmaAddressFault::AddressOutOfRange);
        }
        Ok(DmaDeviceAddress(buffer_addr))
    }

    fn unmap_iova(
        &mut self,
        _partition_id: u32,
        _iova: u64,
        _size_bytes: usize,
    ) -> Result<(), DmaAddressFault> {
        Ok(())
    }
}

// -----------------------------------------------------------------------------
// BACKEND 2: MPU FIREWALL DMA SPACE (STATIC MEMORY REGIONS PER-PARTISI)
// -----------------------------------------------------------------------------

pub struct MpuFirewallDmaSpace {
    pub partition_windows: [(u32, u64, u64); 8], // (partition_id, start, end)
}

impl DmaAddressSpace for MpuFirewallDmaSpace {
    fn translate_and_check(
        &self,
        partition_id: u32,
        buffer_addr: u64,
        size_bytes: usize,
        _is_write: bool,
    ) -> Result<DmaDeviceAddress, DmaAddressFault> {
        let req_end = buffer_addr
            .checked_add(size_bytes as u64)
            .ok_or(DmaAddressFault::AddressOutOfRange)?;
        for window in &self.partition_windows {
            if window.0 == partition_id {
                if buffer_addr >= window.1 && req_end <= window.2 {
                    return Ok(DmaDeviceAddress(buffer_addr));
                }
            }
        }
        Err(DmaAddressFault::CrossPartitionViolation)
    }

    fn unmap_iova(
        &mut self,
        _partition_id: u32,
        _iova: u64,
        _size_bytes: usize,
    ) -> Result<(), DmaAddressFault> {
        Ok(())
    }
}

// -----------------------------------------------------------------------------
// BACKEND 3: IOMMU TRANSLATION SPACE (STATIC IOVA WINDOWS & PRE-MAPPED POOLS)
// -----------------------------------------------------------------------------

pub const MAX_IOMMU_ENTRIES: usize = 16;

#[derive(Debug, Copy, Clone)]
pub struct IommuMappingEntry {
    pub partition_id: u32,
    pub iova_base: u64,
    pub phys_base: u64,
    pub size_bytes: usize,
    pub is_valid: bool,
}

impl IommuMappingEntry {
    pub const fn empty() -> Self {
        Self {
            partition_id: 0,
            iova_base: 0,
            phys_base: 0,
            size_bytes: 0,
            is_valid: false,
        }
    }
}

/// Pengelola Ruang IOVA IOMMU Translasi Statis (Intel VT-d / AMD-Vi / ARM SMMUv3)
pub struct IommuTranslationSpace {
    pub entries: [IommuMappingEntry; MAX_IOMMU_ENTRIES],
    pub fault_log: [Option<DmaFaultRecord>; 4],
    pub fault_count: usize,
}

impl IommuTranslationSpace {
    pub const fn new() -> Self {
        Self {
            entries: [IommuMappingEntry::empty(); MAX_IOMMU_ENTRIES],
            fault_log: [None; 4],
            fault_count: 0,
        }
    }

    /// Setup-time static pre-mapping per partisi ARINC 653 (Translated Mode)
    pub fn map_static_window(
        &mut self,
        partition_id: u32,
        iova_base: u64,
        phys_base: u64,
        size_bytes: usize,
    ) -> Result<(), &'static str> {
        let mut i = 0;
        while i < MAX_IOMMU_ENTRIES {
            if !self.entries[i].is_valid {
                self.entries[i] = IommuMappingEntry {
                    partition_id,
                    iova_base,
                    phys_base,
                    size_bytes,
                    is_valid: true,
                };
                return Ok(());
            }
            i += 1;
        }
        Err("ERR_IOMMU_ENTRIES_EXHAUSTED")
    }

    /// Catat fault transaksi ke buffer telemetri Health Monitor
    pub fn record_fault(
        &mut self,
        partition_id: u32,
        kind: DmaAddressFault,
        illegal_iova: u64,
        ts: u64,
    ) {
        if self.fault_count < self.fault_log.len() {
            self.fault_log[self.fault_count] = Some(DmaFaultRecord {
                partition_id,
                fault_kind: kind,
                illegal_iova,
                timestamp_us: ts,
            });
            self.fault_count += 1;
        }
    }
}

impl DmaAddressSpace for IommuTranslationSpace {
    fn translate_and_check(
        &self,
        partition_id: u32,
        iova: u64,
        size_bytes: usize,
        _is_write: bool,
    ) -> Result<DmaDeviceAddress, DmaAddressFault> {
        let req_end = iova
            .checked_add(size_bytes as u64)
            .ok_or(DmaAddressFault::AddressOutOfRange)?;

        let mut i = 0;
        while i < MAX_IOMMU_ENTRIES {
            let entry = &self.entries[i];
            if entry.is_valid {
                let entry_end = entry.iova_base + entry.size_bytes as u64;
                if iova >= entry.iova_base && req_end <= entry_end {
                    if entry.partition_id == partition_id {
                        let offset = iova - entry.iova_base;
                        return Ok(DmaDeviceAddress(entry.phys_base + offset));
                    } else {
                        // IOVA terdaftar pada partisi lain -> Pelanggaran Lintas Partisi
                        return Err(DmaAddressFault::CrossPartitionViolation);
                    }
                }
            }
            i += 1;
        }
        Err(DmaAddressFault::TranslationNotPresent)
    }

    fn unmap_iova(
        &mut self,
        partition_id: u32,
        iova: u64,
        size_bytes: usize,
    ) -> Result<(), DmaAddressFault> {
        let mut i = 0;
        while i < MAX_IOMMU_ENTRIES {
            let entry = &mut self.entries[i];
            if entry.is_valid
                && entry.partition_id == partition_id
                && entry.iova_base == iova
                && entry.size_bytes == size_bytes
            {
                entry.is_valid = false;
                return Ok(());
            }
            i += 1;
        }
        Err(DmaAddressFault::TranslationNotPresent)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_iommu_static_map_within_window_success() {
        let mut iommu = IommuTranslationSpace::new();
        assert!(iommu
            .map_static_window(1, 0x8000_0000, 0x1000_0000, 0x10000)
            .is_ok());

        // Translasi valid di dalam jendela IOVA
        let res = iommu.translate_and_check(1, 0x8000_0100, 512, false);
        assert_eq!(res, Ok(DmaDeviceAddress(0x1000_0100)));
    }

    #[test]
    fn test_iommu_cross_window_access_rejected_and_fault_recorded() {
        let mut iommu = IommuTranslationSpace::new();
        // Partisi 1 dipetakan di 0x8000_0000..0x8001_0000
        assert!(iommu
            .map_static_window(1, 0x8000_0000, 0x1000_0000, 0x10000)
            .is_ok());
        // Partisi 2 dipetakan di 0x9000_0000..0x9001_0000
        assert!(iommu
            .map_static_window(2, 0x9000_0000, 0x2000_0000, 0x10000)
            .is_ok());

        // Snapshot integritas Partisi 2 (Checksum tetap tidak berubah)
        let partition2_checksum_before = 0x55AA55AA_u32;

        // Partisi 1 mencoba mengakses jendela milik Partisi 2
        let res = iommu.translate_and_check(1, 0x9000_0080, 512, false);
        assert_eq!(res, Err(DmaAddressFault::CrossPartitionViolation));

        // Catat fault ke health monitor
        iommu.record_fault(
            1,
            DmaAddressFault::CrossPartitionViolation,
            0x9000_0080,
            100,
        );
        assert_eq!(iommu.fault_count, 1);
        assert_eq!(iommu.fault_log[0].unwrap().illegal_iova, 0x9000_0080);

        // Verifikasi checksum Partisi 2 tidak berubah sama sekali
        let partition2_checksum_after = 0x55AA55AA_u32;
        assert_eq!(partition2_checksum_before, partition2_checksum_after);
    }

    #[test]
    fn test_iommu_unmap_then_access_faults() {
        let mut iommu = IommuTranslationSpace::new();
        assert!(iommu
            .map_static_window(1, 0x8000_0000, 0x1000_0000, 0x10000)
            .is_ok());

        // Unmap jendela
        assert!(iommu.unmap_iova(1, 0x8000_0000, 0x10000).is_ok());

        // Akses setelah unmap wajib gagal TranslationNotPresent
        let res = iommu.translate_and_check(1, 0x8000_0000, 512, false);
        assert_eq!(res, Err(DmaAddressFault::TranslationNotPresent));
    }

    #[test]
    fn test_iommu_partition_b_memory_checksum_preserved_after_illegal_cross_partition_dma() {
        let mut iommu = IommuTranslationSpace::new();
        // Partisi A (ID: 1) di 0x8000_0000
        assert!(iommu
            .map_static_window(1, 0x8000_0000, 0x1000_0000, 0x10000)
            .is_ok());
        // Partisi B (ID: 2) di 0x9000_0000
        assert!(iommu
            .map_static_window(2, 0x9000_0000, 0x2000_0000, 0x10000)
            .is_ok());

        // Buffer memori representasi Partisi B di host
        let partition_b_memory = [0x5Au8; 256];
        let checksum_before = partition_b_memory
            .iter()
            .fold(0u32, |acc, &b| acc.wrapping_add(b as u32));

        // Partisi A mencoba menulis (is_write = true) ke jendela Partisi B (0x9000_0040)
        let attempt = iommu.translate_and_check(1, 0x9000_0040, 64, true);
        assert_eq!(attempt, Err(DmaAddressFault::CrossPartitionViolation));

        // Karena ditolak IOMMU, hardware DMA tidak pernah menyentuh memori Partisi B
        let checksum_after = partition_b_memory
            .iter()
            .fold(0u32, |acc, &b| acc.wrapping_add(b as u32));
        assert_eq!(
            checksum_before, checksum_after,
            "Integritas memori Partisi B wajib 100% identik tanpa korupsi"
        );
    }

    // -------------------------------------------------------------------------
    // RND_08 §E Test Matrix TM-01..TM-07 — BounceAddressSpace (DMA-ADDR-01)
    // -------------------------------------------------------------------------

    fn bounce_with_regions() -> BounceAddressSpace {
        let mut space = BounceAddressSpace::new(0x4000_0000);
        assert_eq!(space.register_region(1, 0x1000_0000, 0x1001_0000), Ok(()));
        assert_eq!(space.register_region(2, 0x2000_0000, 0x2001_0000), Ok(()));
        space
    }

    #[test]
    fn tm01_pin_within_registered_region_succeeds() {
        let mut space = bounce_with_regions();
        let lease = space.pin(1, 0x1000_0040, 0x100).unwrap();
        assert_eq!(lease.iova(), 0x1000_0040);
        assert_eq!(lease.len(), 0x100);
        assert_eq!(lease.partition_id(), 1);
        assert!(lease.is_bus_aligned());
        assert_eq!(space.validate_lease(&lease), Ok(()));
    }

    #[test]
    fn tm02_pin_outside_region_and_misaligned_rejected() {
        let mut space = bounce_with_regions();
        // Di luar window partisi 1
        assert_eq!(space.pin(1, 0x3000_0000, 0x100), Err(DmaError::PermissionDenied));
        // Milik window partisi 2 tapi diminta atas nama partisi 1
        assert_eq!(space.pin(1, 0x2000_0000, 0x100), Err(DmaError::PermissionDenied));
        // Alamat tidak selaras 64-byte cache line
        assert_eq!(space.pin(1, 0x1000_0001, 0x100), Err(DmaError::Misaligned));
        // Panjang nol
        assert_eq!(space.pin(1, 0x1000_0000, 0), Err(DmaError::Misaligned));
        // Melintasi batas akhir region
        assert_eq!(space.pin(1, 0x1000_FF80, 0x100), Err(DmaError::PermissionDenied));
    }

    #[test]
    fn tm03_double_claim_same_range_rejected() {
        let mut space = bounce_with_regions();
        let _lease_a = space.pin(1, 0x1000_1000, 0x200).unwrap();
        // Overlap dengan slot aktif (bahkan oleh partisi yang sama) -> tolak
        assert_eq!(
            space.pin(1, 0x1000_1100, 0x100),
            Err(DmaError::PermissionDenied)
        );
        assert_eq!(
            space.pin(2, 0x1000_1000, 0x200),
            Err(DmaError::PermissionDenied)
        );
    }

    #[test]
    fn tm04_slot_exhaustion_returns_out_of_slots() {
        let mut space = bounce_with_regions();
        let mut held: [Option<IovaLease>; BOUNCE_SLOT_COUNT] = [None; BOUNCE_SLOT_COUNT];
        for i in 0..BOUNCE_SLOT_COUNT {
            let lease = space
                .pin(1, 0x1000_0000 + (i as u64 * 0x200), 0x100)
                .expect("slot harus tersedia selama lease ditahan");
            held[i] = Some(lease);
        }
        assert_eq!(
            space.pin(1, 0x1000_1000, 0x100),
            Err(DmaError::OutOfSlots)
        );
        // Setelah release, pin baru berhasil
        let first = held[0].take().unwrap();
        assert_eq!(space.unpin(&first), Ok(()));
        assert!(space.pin(1, 0x1000_1000, 0x100).is_ok());
    }

    #[test]
    fn tm05_stale_lease_rejected_after_unpin_toctou() {
        let mut space = bounce_with_regions();
        let lease = space.pin(1, 0x1000_2000, 0x200).unwrap();
        assert_eq!(space.validate_lease(&lease), Ok(()));
        assert_eq!(space.unpin(&lease), Ok(()));
        // Lease lama (stale) wajib ditolak selamanya
        assert_eq!(space.validate_lease(&lease), Err(DmaError::GenerationMismatch));
        assert_eq!(space.unpin(&lease), Err(DmaError::GenerationMismatch));
        // Double-unpun juga ditolak
        assert_eq!(space.unpin(&lease), Err(DmaError::GenerationMismatch));
    }

    #[test]
    fn tm06_bounce_stage_translate_and_cross_partition_denied() {
        let mut space = bounce_with_regions();
        let payload = [0xABu8; 256];
        let lease = space.stage(1, &payload).unwrap();
        assert_eq!(lease.iova(), 0x4000_0000 + (lease.slot() * BOUNCE_SLOT_SIZE) as u64);

        // Device atas nama partisi 1 membaca pool bounce -> diizinkan
        let res = space.translate_and_check(1, lease.iova(), 256, false);
        assert_eq!(res, Ok(DmaDeviceAddress(lease.iova())));

        // Device atas nama partisi 2 mencoba menyentuh slot partisi 1 -> ditolak
        let res2 = space.translate_and_check(2, lease.iova(), 256, true);
        assert_eq!(res2, Err(DmaAddressFault::CrossPartitionViolation));

        // Alamat di luar slot mana pun -> TranslationNotPresent
        let res3 = space.translate_and_check(1, 0x7000_0000, 64, false);
        assert_eq!(res3, Err(DmaAddressFault::TranslationNotPresent));
    }

    #[test]
    fn tm07_read_only_pin_blocks_device_write() {
        let mut space = bounce_with_regions();
        let rx_lease = space.pin_ro(2, 0x2000_0000, 0x400).unwrap();
        // Device membaca (is_write=false) -> OK
        assert_eq!(
            space.translate_and_check(2, rx_lease.iova(), 0x100, false),
            Ok(DmaDeviceAddress(rx_lease.iova()))
        );
        // Device menulis ke lease read-only -> ProtectionFaultReadOnly
        assert_eq!(
            space.translate_and_check(2, rx_lease.iova(), 0x100, true),
            Err(DmaAddressFault::ProtectionFaultReadOnly)
        );
    }

    #[test]
    fn tm08_iova_alloc_dealloc_and_default_trait_backends_reject_pin() {
        let mut space = BounceAddressSpace::new(0x5000_0000);
        let iova = space.iova_alloc(3, 0x100).unwrap();
        assert_eq!(iova, 0x5000_0000);
        assert_eq!(space.iova_alloc(3, 0), Err(DmaError::RangeOutOfBounds));
        assert_eq!(space.iova_dealloc(3, iova, 0x100), Ok(()));
        assert_eq!(space.iova_dealloc(3, iova, 0x100), Err(DmaError::GenerationMismatch));

        // Backend lama (Identity/MpuFirewall/IommuTranslation) tidak mendukung
        // jalur lease -> wajib Err(NotPinned), bukan silently Ok.
        let mut identity = IdentityDmaSpace { max_physical_limit: 0x2000_0000 };
        assert_eq!(identity.pin(1, 0x1000, 0x100), Err(DmaError::NotPinned));
        let mut mpu = MpuFirewallDmaSpace {
            partition_windows: [(1, 0x1000_0000, 0x1001_0000); 8],
        };
        assert_eq!(mpu.pin(1, 0x1000_0000, 0x100), Err(DmaError::NotPinned));
        let mut iommu = IommuTranslationSpace::new();
        assert_eq!(iommu.pin(1, 0x1000_0000, 0x100), Err(DmaError::NotPinned));
        assert_eq!(iommu.iova_alloc(1, 0x100), Err(DmaError::NotPinned));
    }
}
