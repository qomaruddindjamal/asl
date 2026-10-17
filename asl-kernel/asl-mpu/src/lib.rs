//! ASL MPU: Hardware Memory Protection Unit Model (No-MMU Isolation)
//! =================================================================
//! Abstraksi MPU (Cortex-M PMSAv7/PMSAv8, RISC-V PMP, Infineon TriCore MPU).
//! Menyediakan isolasi thread/task deterministik pada mikrokontroler tanpa paging virtual.

#![no_std]

/// Hak Akses Region MPU Hardware
#[repr(u8)]
#[derive(Debug, Copy, Clone, PartialEq, Eq)]
pub enum MpuAccessPermission {
    NoAccess = 0b000,
    PrivilegedRwUserNone = 0b001,
    PrivilegedRwUserRo = 0b010,
    PrivilegedRwUserRw = 0b011,
    PrivilegedRoUserNone = 0b101,
    PrivilegedRoUserRo = 0b110,
}

/// Konfigurasi 1 Region MPU Silikon (Ukuran berbasis eksponensial 2^N)
#[repr(C)]
#[derive(Debug, Copy, Clone)]
pub struct MpuRegionConfig {
    pub region_number: u8,
    pub base_address: u32,
    pub size_order: u8, // Ukuran region = 2^(size_order + 1)
    pub permission: MpuAccessPermission,
    pub is_executable: bool,
    pub is_enabled: bool,
}

impl MpuRegionConfig {
    pub const fn empty(region_num: u8) -> Self {
        Self {
            region_number: region_num,
            base_address: 0,
            size_order: 0,
            permission: MpuAccessPermission::NoAccess,
            is_executable: false,
            is_enabled: false,
        }
    }
}

pub const MAX_HARDWARE_MPU_REGIONS: usize = 16;

/// Driver Manajer MPU Hardware
pub struct HardwareMpuController {
    pub regions: [MpuRegionConfig; MAX_HARDWARE_MPU_REGIONS],
    pub is_mpu_enabled: bool,
}

impl HardwareMpuController {
    pub const fn new() -> Self {
        Self {
            regions: [
                MpuRegionConfig::empty(0),
                MpuRegionConfig::empty(1),
                MpuRegionConfig::empty(2),
                MpuRegionConfig::empty(3),
                MpuRegionConfig::empty(4),
                MpuRegionConfig::empty(5),
                MpuRegionConfig::empty(6),
                MpuRegionConfig::empty(7),
                MpuRegionConfig::empty(8),
                MpuRegionConfig::empty(9),
                MpuRegionConfig::empty(10),
                MpuRegionConfig::empty(11),
                MpuRegionConfig::empty(12),
                MpuRegionConfig::empty(13),
                MpuRegionConfig::empty(14),
                MpuRegionConfig::empty(15),
            ],
            is_mpu_enabled: false,
        }
    }

    /// Konfigurasi region MPU
    pub fn configure_region(&mut self, region: MpuRegionConfig) -> Result<(), &'static str> {
        let idx = region.region_number as usize;
        if idx >= MAX_HARDWARE_MPU_REGIONS {
            return Err("ERR_INVALID_MPU_REGION_INDEX");
        }
        self.regions[idx] = region;
        Ok(())
    }

    /// Validasi proteksi sebelum eksekusi pergantian task
    pub fn switch_task_mpu_boundary(
        &mut self,
        stack_base: u32,
        stack_size_order: u8,
    ) -> Result<(), &'static str> {
        // Region 0: Flash/ROM Read-Only Executable
        // Region 1: SRAM Global
        // Region 2: Task Stack Pribadi (Hanya task aktif yang dapat menulis)
        self.configure_region(MpuRegionConfig {
            region_number: 2,
            base_address: stack_base,
            size_order: stack_size_order,
            permission: MpuAccessPermission::PrivilegedRwUserRw,
            is_executable: false,
            is_enabled: true,
        })
    }
}
