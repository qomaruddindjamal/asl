//! ASL Security Vault: ARINC 653 Spatial Partitioning & Memory Armor
//! ================================================================
//! Mengimplementasikan doktrin The Impenetrable Vault:
//! 1. Isolasi Kedap Udara Antar Partisi (Air-Gapped Virtual Partitions)
//! 2. Proteksi Akses Memori Deterministik (Spatial Boundaries)
//! 3. Verifikasi Logika Keamanan Formal (SeL4-Class Microkernel Integrity)

#![no_std]

/// Tipe Hak Akses Partisi Memori
#[repr(u8)]
#[derive(Debug, Copy, Clone, PartialEq, Eq)]
pub enum PartitionAccessRights {
    None = 0,
    ReadOnly = 1,
    ReadWrite = 2,
    ExecuteOnly = 3,
    ReadExecute = 4,
}

/// Deskriptor Batas Ruang (Spatial Boundary Window)
#[derive(Debug, Copy, Clone)]
pub struct MemoryWindow {
    pub start_address: u64,
    pub length_bytes: u64,
    pub rights: PartitionAccessRights,
}

impl MemoryWindow {
    pub const fn empty() -> Self {
        Self {
            start_address: 0,
            length_bytes: 0,
            rights: PartitionAccessRights::None,
        }
    }

    /// Cek apakah alamat berada di dalam jendela memori ini dengan proteksi checked arithmetic anti-overflow
    #[inline(always)]
    pub fn contains(&self, address: u64, size: u64) -> bool {
        if self.rights == PartitionAccessRights::None || self.length_bytes == 0 || size == 0 {
            return false;
        }
        if address < self.start_address {
            return false;
        }
        let req_end = match address.checked_add(size) {
            Some(end) => end,
            None => return false, // Overflow pada requested range
        };
        let win_end = match self.start_address.checked_add(self.length_bytes) {
            Some(end) => end,
            None => return false, // Overflow pada window descriptor
        };
        req_end <= win_end
    }
}

pub const MAX_WINDOWS_PER_PARTITION: usize = 8;
pub const MAX_PARTITIONS: usize = 32;

/// Partisi Ruang Udara ARINC 653 (Air-Gapped Partition)
#[derive(Copy, Clone)]
pub struct PartitionVault {
    pub partition_id: u32,
    pub is_active: bool,
    pub is_critical_avionics: bool,
    pub windows: [MemoryWindow; MAX_WINDOWS_PER_PARTITION],
}

impl PartitionVault {
    pub const fn new(id: u32, is_critical: bool) -> Self {
        Self {
            partition_id: id,
            is_active: false,
            is_critical_avionics: is_critical,
            windows: [MemoryWindow::empty(); MAX_WINDOWS_PER_PARTITION],
        }
    }

    /// Mendaftarkan segmen memori resmi ke dalam partisi
    pub fn register_window(&mut self, window: MemoryWindow) -> Result<(), &'static str> {
        let mut i = 0;
        while i < MAX_WINDOWS_PER_PARTITION {
            if self.windows[i].rights == PartitionAccessRights::None {
                self.windows[i] = window;
                return Ok(());
            }
            i += 1;
        }
        Err("ERR_PARTITION_WINDOWS_EXHAUSTED")
    }

    /// Validasi matematis apakah upaya akses memori legal
    pub fn verify_access(&self, address: u64, size: u64, requested_rights: PartitionAccessRights) -> bool {
        let mut i = 0;
        while i < MAX_WINDOWS_PER_PARTITION {
            let win = &self.windows[i];
            if win.contains(address, size) {
                match (win.rights, requested_rights) {
                    (PartitionAccessRights::ReadWrite, PartitionAccessRights::ReadOnly) => return true,
                    (PartitionAccessRights::ReadWrite, PartitionAccessRights::ReadWrite) => return true,
                    (PartitionAccessRights::ReadOnly, PartitionAccessRights::ReadOnly) => return true,
                    (PartitionAccessRights::ReadExecute, PartitionAccessRights::ReadOnly) => return true,
                    (PartitionAccessRights::ReadExecute, PartitionAccessRights::ExecuteOnly) => return true,
                    (PartitionAccessRights::ReadExecute, PartitionAccessRights::ReadExecute) => return true,
                    _ => return false,
                }
            }
            i += 1;
        }
        false // Akses di luar batas memicu Hardware Trap / Immediate Fault Isolation
    }
}

/// Pengelola Ruang Partisi Global (Vault Partition Hypervisor)
pub struct SpatialPartitionManager {
    pub partitions: [PartitionVault; MAX_PARTITIONS],
    pub active_partition_index: usize,
}

impl SpatialPartitionManager {
    pub const fn new() -> Self {
        Self {
            partitions: [PartitionVault::new(0, false); MAX_PARTITIONS],
            active_partition_index: 0,
        }
    }

    /// Daftarkan partisi baru
    pub fn create_partition(&mut self, id: u32, is_critical: bool) -> Result<usize, &'static str> {
        let mut i = 0;
        while i < MAX_PARTITIONS {
            if !self.partitions[i].is_active {
                self.partitions[i] = PartitionVault::new(id, is_critical);
                self.partitions[i].is_active = true;
                return Ok(i);
            }
            i += 1;
        }
        Err("ERR_MAX_PARTITIONS_REACHED")
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_memory_window_normal_range() {
        let win = MemoryWindow {
            start_address: 0x1000,
            length_bytes: 0x1000, // Range: [0x1000, 0x2000]
            rights: PartitionAccessRights::ReadWrite,
        };
        // Inside boundary
        assert!(win.contains(0x1000, 0x1000));
        assert!(win.contains(0x1000, 0x100));
        assert!(win.contains(0x1F00, 0x100));
        // Outside boundary
        assert!(!win.contains(0x0FFF, 0x10));
        assert!(!win.contains(0x1000, 0x1001));
        assert!(!win.contains(0x2000, 0x1));
    }

    #[test]
    fn test_memory_window_zero_size() {
        let win = MemoryWindow {
            start_address: 0x1000,
            length_bytes: 0x1000,
            rights: PartitionAccessRights::ReadWrite,
        };
        // Zero size request must return false
        assert!(!win.contains(0x1000, 0));
    }

    #[test]
    fn test_memory_window_overflow_request() {
        let win = MemoryWindow {
            start_address: 0x1000,
            length_bytes: 0x1000,
            rights: PartitionAccessRights::ReadWrite,
        };
        // Address + size overflows u64::MAX
        assert!(!win.contains(u64::MAX - 10, 20));
        assert!(!win.contains(0x1000, u64::MAX));
        assert!(!win.contains(u64::MAX, 1));
    }

    #[test]
    fn test_memory_window_overflow_descriptor() {
        let win = MemoryWindow {
            start_address: u64::MAX - 100,
            length_bytes: 200, // start + length overflows u64
            rights: PartitionAccessRights::ReadWrite,
        };
        // Malformed window descriptor must not accept any access
        assert!(!win.contains(u64::MAX - 50, 10));
    }

    #[test]
    fn test_partition_verify_access_matrix() {
        let mut vault = PartitionVault::new(1, true);
        let win = MemoryWindow {
            start_address: 0x1000,
            length_bytes: 0x1000,
            rights: PartitionAccessRights::ReadOnly,
        };
        assert!(vault.register_window(win).is_ok());

        assert!(vault.verify_access(0x1000, 0x100, PartitionAccessRights::ReadOnly));
        assert!(!vault.verify_access(0x1000, 0x100, PartitionAccessRights::ReadWrite));
        assert!(!vault.verify_access(u64::MAX - 10, 20, PartitionAccessRights::ReadOnly));
    }
}

