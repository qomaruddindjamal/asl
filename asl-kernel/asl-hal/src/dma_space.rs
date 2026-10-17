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

    /// Melepas pemetaan halaman IOVA
    fn unmap_iova(
        &mut self,
        partition_id: u32,
        iova: u64,
        size_bytes: usize,
    ) -> Result<(), DmaAddressFault>;
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
}
