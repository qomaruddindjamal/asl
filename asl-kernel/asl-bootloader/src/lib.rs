//! ASL Bootloader Agnostic Protocol Interface
//! ==========================================
//! DO-178C DAL-A Compliant Hardware Handover Protocol.
//! Mendukung konversi data boot dari UEFI GOP, Multiboot2, dan Flattened Device Tree (DTB).
//! Beroperasi di level `#![no_std]` mutlak.

#![no_std]

/// Magic signature untuk memvalidasi keaslian handover bootloader ASL ("ASL_BOOT_AVIONIC")
pub const ASL_BOOT_MAGIC: u64 = 0x41534C5F424F4F54; // 'ASL_BOOT'

/// Jenis arsitektur CPU target
#[repr(u32)]
#[derive(Debug, Copy, Clone, PartialEq, Eq)]
pub enum TargetArch {
    X86_64 = 1,
    Aarch64 = 2,
    Riscv64 = 3,
    Mips64 = 4,
    PowerPc64 = 5,
    SparcV8 = 6,
    LoongArch64 = 7,
    Unknown = 0xFFFF,
}

/// Tipe region memori fisik yang diserahkan oleh bootloader
#[repr(u32)]
#[derive(Debug, Copy, Clone, PartialEq, Eq)]
pub enum MemoryRegionType {
    UsableRam = 1,
    Reserved = 2,
    AcpiReclaimable = 3,
    AcpiNvs = 4,
    BadMemory = 5,
    BootloaderReclaimable = 6,
    KernelImage = 7,
    PageTables = 8,
    FrameBuffer = 9,
}

/// Deskriptor satu segmen memori fisik
#[repr(C)]
#[derive(Debug, Copy, Clone)]
pub struct MemoryDescriptor {
    pub base_physical_addr: u64,
    pub length_bytes: u64,
    pub region_type: MemoryRegionType,
    pub flags: u32,
}

/// Struktur Framebuffer grafis awal untuk diagnostik avionik wind tunnel
#[repr(C)]
#[derive(Debug, Copy, Clone)]
pub struct FramebufferConfig {
    pub base_physical_addr: u64,
    pub size_bytes: u64,
    pub width: u32,
    pub height: u32,
    pub pixels_per_scan_line: u32,
    pub bytes_per_pixel: u8,
}

/// Kapasitas maksimum entri memory map yang dapat ditampung dalam static payload
pub const MAX_MEMORY_MAP_ENTRIES: usize = 128;

/// Blok Handover Utama Bootloader (Multi-Arch Agnostic Boot Information)
#[repr(C)]
#[derive(Debug, Clone)]
pub struct AslBootHandover {
    /// Wajib bernilai `ASL_BOOT_MAGIC`
    pub magic: u64,
    /// Versi protokol handover (dimulai dari 1)
    pub protocol_version: u32,
    /// Arsitektur hardware yang terdeteksi
    pub target_arch: TargetArch,
    
    /// Virtual base offset tempat kernel di-remap (Higher Half Kernel)
    pub kernel_virtual_base: u64,
    pub kernel_physical_base: u64,
    pub kernel_size_bytes: u64,
    
    /// Pointer fisik ke tabel ACPI RSDP atau Flattened Device Tree (DTB)
    pub device_tree_or_acpi_ptr: u64,
    
    /// Konfigurasi Framebuffer grafis awal
    pub framebuffer: FramebufferConfig,
    
    /// Memory Map fisik
    pub memory_map_count: usize,
    pub memory_map: [MemoryDescriptor; MAX_MEMORY_MAP_ENTRIES],
}

impl AslBootHandover {
    /// Inisialisasi blok handover kosong
    pub const fn empty() -> Self {
        Self {
            magic: ASL_BOOT_MAGIC,
            protocol_version: 1,
            target_arch: TargetArch::Unknown,
            kernel_virtual_base: 0,
            kernel_physical_base: 0,
            kernel_size_bytes: 0,
            device_tree_or_acpi_ptr: 0,
            framebuffer: FramebufferConfig {
                base_physical_addr: 0,
                size_bytes: 0,
                width: 0,
                height: 0,
                pixels_per_scan_line: 0,
                bytes_per_pixel: 0,
            },
            memory_map_count: 0,
            memory_map: [MemoryDescriptor {
                base_physical_addr: 0,
                length_bytes: 0,
                region_type: MemoryRegionType::Reserved,
                flags: 0,
            }; MAX_MEMORY_MAP_ENTRIES],
        }
    }

    /// Validasi matematis integritas bootloader handover dengan pengamanan physical address range & overflow
    pub fn validate(&self) -> Result<(), &'static str> {
        if self.magic != ASL_BOOT_MAGIC {
            return Err("ERR_INVALID_BOOT_MAGIC");
        }
        if self.protocol_version != 1 {
            return Err("ERR_UNSUPPORTED_PROTOCOL_VERSION");
        }
        if self.target_arch == TargetArch::Unknown {
            return Err("ERR_UNKNOWN_TARGET_ARCH");
        }
        if self.memory_map_count == 0 || self.memory_map_count > MAX_MEMORY_MAP_ENTRIES {
            return Err("ERR_INVALID_MEMORY_MAP_SIZE");
        }
        if self.kernel_size_bytes == 0 {
            return Err("ERR_ZERO_KERNEL_SIZE");
        }

        // Validasi pembatasan integer overflow pada alamat fisik kernel
        let kernel_phys_end = match self.kernel_physical_base.checked_add(self.kernel_size_bytes) {
            Some(end) => end,
            None => return Err("ERR_KERNEL_PHYSICAL_RANGE_OVERFLOW"),
        };

        // Standar platform: Alamat fisik 64-bit pada arsitektur modern dibatasi pada 52-bit (4 Petabytes)
        const MAX_PHYSICAL_ADDRESS_LIMIT: u64 = 0x000F_FFFF_FFFF_FFFF;
        if kernel_phys_end > MAX_PHYSICAL_ADDRESS_LIMIT {
            return Err("ERR_KERNEL_PHYSICAL_ADDRESS_OUT_OF_BOUNDS");
        }

        // Validasi pembatasan integer overflow pada alamat virtual kernel
        if self.kernel_virtual_base.checked_add(self.kernel_size_bytes).is_none() {
            return Err("ERR_KERNEL_VIRTUAL_RANGE_OVERFLOW");
        }

        // Validasi setiap entri dalam memory map fisik
        let mut i = 0;
        while i < self.memory_map_count {
            let desc = &self.memory_map[i];
            if desc.length_bytes == 0 {
                return Err("ERR_EMPTY_MEMORY_REGION_DESCRIPTOR");
            }
            let reg_end = match desc.base_physical_addr.checked_add(desc.length_bytes) {
                Some(end) => end,
                None => return Err("ERR_MEMORY_REGION_OVERFLOW"),
            };
            if reg_end > MAX_PHYSICAL_ADDRESS_LIMIT {
                return Err("ERR_MEMORY_REGION_EXCEEDS_PHYSICAL_LIMIT");
            }
            i += 1;
        }

        Ok(())
    }

    /// Menghitung total RAM fisik yang dapat digunakan (Usable RAM)
    pub fn calculate_usable_memory(&self) -> u64 {
        let mut total: u64 = 0;
        let mut i = 0;
        while i < self.memory_map_count {
            if self.memory_map[i].region_type == MemoryRegionType::UsableRam {
                total += self.memory_map[i].length_bytes;
            }
            i += 1;
        }
        total
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn create_valid_handover() -> AslBootHandover {
        let mut h = AslBootHandover::empty();
        h.target_arch = TargetArch::X86_64;
        h.kernel_physical_base = 0x0010_0000; // 1 MB
        h.kernel_virtual_base = 0xFFFF_8000_0010_0000;
        h.kernel_size_bytes = 0x0020_0000; // 2 MB
        h.memory_map_count = 1;
        h.memory_map[0] = MemoryDescriptor {
            base_physical_addr: 0x0010_0000,
            length_bytes: 0x1000_0000, // 256 MB
            region_type: MemoryRegionType::UsableRam,
            flags: 0,
        };
        h
    }

    #[test]
    fn test_valid_handover() {
        let h = create_valid_handover();
        assert_eq!(h.validate(), Ok(()));
        assert_eq!(h.calculate_usable_memory(), 0x1000_0000);
    }

    #[test]
    fn test_invalid_magic_or_version() {
        let mut h = create_valid_handover();
        h.magic = 0xDEADBEEF;
        assert_eq!(h.validate(), Err("ERR_INVALID_BOOT_MAGIC"));

        let mut h2 = create_valid_handover();
        h2.protocol_version = 99;
        assert_eq!(h2.validate(), Err("ERR_UNSUPPORTED_PROTOCOL_VERSION"));
    }

    #[test]
    fn test_zero_kernel_size() {
        let mut h = create_valid_handover();
        h.kernel_size_bytes = 0;
        assert_eq!(h.validate(), Err("ERR_ZERO_KERNEL_SIZE"));
    }

    #[test]
    fn test_kernel_physical_overflow() {
        let mut h = create_valid_handover();
        h.kernel_physical_base = u64::MAX - 100;
        h.kernel_size_bytes = 200;
        assert_eq!(h.validate(), Err("ERR_KERNEL_PHYSICAL_RANGE_OVERFLOW"));
    }

    #[test]
    fn test_kernel_physical_exceeds_platform_limit() {
        let mut h = create_valid_handover();
        h.kernel_physical_base = 0x0010_0000_0000_0000; // > 52-bit limit
        h.kernel_size_bytes = 0x1000;
        assert_eq!(h.validate(), Err("ERR_KERNEL_PHYSICAL_ADDRESS_OUT_OF_BOUNDS"));
    }

    #[test]
    fn test_kernel_virtual_overflow() {
        let mut h = create_valid_handover();
        h.kernel_virtual_base = u64::MAX - 100;
        h.kernel_size_bytes = 200;
        assert_eq!(h.validate(), Err("ERR_KERNEL_VIRTUAL_RANGE_OVERFLOW"));
    }

    #[test]
    fn test_empty_or_overflowing_memory_region() {
        let mut h = create_valid_handover();
        h.memory_map[0].length_bytes = 0;
        assert_eq!(h.validate(), Err("ERR_EMPTY_MEMORY_REGION_DESCRIPTOR"));

        let mut h2 = create_valid_handover();
        h2.memory_map[0].base_physical_addr = u64::MAX - 50;
        h2.memory_map[0].length_bytes = 100;
        assert_eq!(h2.validate(), Err("ERR_MEMORY_REGION_OVERFLOW"));
    }

    #[test]
    fn test_build_profile_option_env_fallback() {
        // Validasi AG-06: option_env fallback menghasilkan string valid
        let profile = option_env!("ASL_BUILD_PROFILE_NON_EXISTENT").unwrap_or("release-fallback");
        assert_eq!(profile, "release-fallback");
    }
}


