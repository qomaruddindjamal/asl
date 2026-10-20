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
        let kernel_phys_end = match self
            .kernel_physical_base
            .checked_add(self.kernel_size_bytes)
        {
            Some(end) => end,
            None => return Err("ERR_KERNEL_PHYSICAL_RANGE_OVERFLOW"),
        };

        // Standar platform: Alamat fisik 64-bit pada arsitektur modern dibatasi pada 52-bit (4 Petabytes)
        const MAX_PHYSICAL_ADDRESS_LIMIT: u64 = 0x000F_FFFF_FFFF_FFFF;
        if kernel_phys_end > MAX_PHYSICAL_ADDRESS_LIMIT {
            return Err("ERR_KERNEL_PHYSICAL_ADDRESS_OUT_OF_BOUNDS");
        }

        // Validasi pembatasan integer overflow pada alamat virtual kernel
        if self
            .kernel_virtual_base
            .checked_add(self.kernel_size_bytes)
            .is_none()
        {
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

        // Validasi Framebuffer Config jika diaktifkan (base_physical_addr != 0)
        let fb = &self.framebuffer;
        if fb.base_physical_addr != 0 {
            if fb.base_physical_addr < 0x1000 {
                return Err("ERR_FRAMEBUFFER_BASE_BELOW_MINIMUM");
            }
            if fb.base_physical_addr % 4 != 0 {
                return Err("ERR_FRAMEBUFFER_BASE_UNALIGNED");
            }
            if fb.width == 0
                || fb.height == 0
                || fb.pixels_per_scan_line == 0
                || fb.bytes_per_pixel == 0
            {
                return Err("ERR_INVALID_FRAMEBUFFER_DIMENSIONS");
            }
            if fb.bytes_per_pixel != 2 && fb.bytes_per_pixel != 3 && fb.bytes_per_pixel != 4 {
                return Err("ERR_UNSUPPORTED_FRAMEBUFFER_BPP");
            }
            if fb.pixels_per_scan_line < fb.width {
                return Err("ERR_FRAMEBUFFER_STRIDE_LESS_THAN_WIDTH");
            }
            let pitch =
                match (fb.pixels_per_scan_line as u64).checked_mul(fb.bytes_per_pixel as u64) {
                    Some(p) => p,
                    None => return Err("ERR_FRAMEBUFFER_PITCH_OVERFLOW"),
                };
            let required_bytes = match pitch.checked_mul(fb.height as u64) {
                Some(bytes) => bytes,
                None => return Err("ERR_FRAMEBUFFER_SIZE_OVERFLOW"),
            };
            let fb_end = match fb.base_physical_addr.checked_add(required_bytes) {
                Some(end) => end,
                None => return Err("ERR_FRAMEBUFFER_PHYSICAL_RANGE_OVERFLOW"),
            };
            if fb_end > MAX_PHYSICAL_ADDRESS_LIMIT {
                return Err("ERR_FRAMEBUFFER_EXCEEDS_PHYSICAL_LIMIT");
            }

            // Mencegah collision/overlap antara framebuffer memory range dan kernel physical image
            let k_start = self.kernel_physical_base;
            let k_end = kernel_phys_end;
            if !(fb_end <= k_start || fb.base_physical_addr >= k_end) {
                return Err("ERR_FRAMEBUFFER_OVERLAPS_KERNEL_MEMORY");
            }
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

    /// Menghitung total RAM fisik yang terpasang (Installed RAM: Usable + Reserved + ACPI + Kernel + Bootloader)
    /// Mengabaikan MMIO / PCI BAR / Framebuffer address space di address tinggi (> 0x8000_0000).
    pub fn calculate_installed_memory(&self) -> u64 {
        let mut total: u64 = 0;
        let mut i = 0;
        while i < self.memory_map_count {
            let desc = &self.memory_map[i];
            // Wilayah MMIO/PCI/Framebuffer video memory dipetakan di alamat tinggi (> 2GB / 0x8000_0000)
            if desc.base_physical_addr < 0x8000_0000 {
                match desc.region_type {
                    MemoryRegionType::BadMemory | MemoryRegionType::FrameBuffer => {}
                    _ => {
                        total += desc.length_bytes;
                    }
                }
            }
            i += 1;
        }
        // Round to nearest megabyte (+ 512KB) to accurately reflect physical DIMM / motherboard base RAM
        // (accounting for standard EBDA / BIOS reservation below 1MB)
        if total > 0 {
            let mb_half = 512 * 1024;
            ((total + mb_half) / (1024 * 1024)) * 1024 * 1024
        } else {
            0
        }
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
    fn test_valid_handover_and_usable_memory_total() {
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
        assert_eq!(
            h.validate(),
            Err("ERR_KERNEL_PHYSICAL_ADDRESS_OUT_OF_BOUNDS")
        );
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

    #[test]
    fn test_framebuffer_config_validation_positive_and_negative() {
        let mut h = create_valid_handover();
        // Positive: Valid Framebuffer
        h.framebuffer = FramebufferConfig {
            base_physical_addr: 0xE000_0000,
            size_bytes: 1024 * 768 * 4,
            width: 1024,
            height: 768,
            pixels_per_scan_line: 1024,
            bytes_per_pixel: 4,
        };
        assert_eq!(h.validate(), Ok(()));

        // Negative: Zero width
        let mut h_zero_w = h.clone();
        h_zero_w.framebuffer.width = 0;
        assert_eq!(
            h_zero_w.validate(),
            Err("ERR_INVALID_FRAMEBUFFER_DIMENSIONS")
        );

        // Negative: Unsupported BPP (e.g. 1 byte per pixel)
        let mut h_bad_bpp = h.clone();
        h_bad_bpp.framebuffer.bytes_per_pixel = 1;
        assert_eq!(h_bad_bpp.validate(), Err("ERR_UNSUPPORTED_FRAMEBUFFER_BPP"));

        // Negative: Stride less than width
        let mut h_bad_stride = h.clone();
        h_bad_stride.framebuffer.pixels_per_scan_line = 800; // < 1024 width
        assert_eq!(
            h_bad_stride.validate(),
            Err("ERR_FRAMEBUFFER_STRIDE_LESS_THAN_WIDTH")
        );

        // Negative: Framebuffer physical address overflow
        let mut h_overflow = h.clone();
        h_overflow.framebuffer.base_physical_addr = u64::MAX - 3; // 4-byte aligned (ends in 0x...FFFC)
        assert_eq!(
            h_overflow.validate(),
            Err("ERR_FRAMEBUFFER_PHYSICAL_RANGE_OVERFLOW")
        );

        // Negative: Exceeds 52-bit physical limit
        let mut h_limit = h.clone();
        h_limit.framebuffer.base_physical_addr = 0x0010_0000_0000_0000;
        assert_eq!(
            h_limit.validate(),
            Err("ERR_FRAMEBUFFER_EXCEEDS_PHYSICAL_LIMIT")
        );

        // Negative: Address below 0x1000 minimum
        let mut h_low = h.clone();
        h_low.framebuffer.base_physical_addr = 0x800;
        assert_eq!(h_low.validate(), Err("ERR_FRAMEBUFFER_BASE_BELOW_MINIMUM"));

        // Negative: Unaligned base address (not 4-byte aligned)
        let mut h_unaligned = h.clone();
        h_unaligned.framebuffer.base_physical_addr = 0xE000_0003;
        assert_eq!(
            h_unaligned.validate(),
            Err("ERR_FRAMEBUFFER_BASE_UNALIGNED")
        );

        // Negative: Overlaps kernel image memory
        let mut h_overlap = h.clone();
        h_overlap.framebuffer.base_physical_addr = h_overlap.kernel_physical_base;
        assert_eq!(
            h_overlap.validate(),
            Err("ERR_FRAMEBUFFER_OVERLAPS_KERNEL_MEMORY")
        );

        // Negative: Framebuffer size calculation overflow
        let mut h_size_overflow = h.clone();
        h_size_overflow.framebuffer.height = u32::MAX;
        h_size_overflow.framebuffer.pixels_per_scan_line = 0x8000_0000;
        h_size_overflow.framebuffer.width = 100;
        h_size_overflow.framebuffer.bytes_per_pixel = 4;
        assert_eq!(
            h_size_overflow.validate(),
            Err("ERR_FRAMEBUFFER_SIZE_OVERFLOW")
        );

        // Positive: Framebuffer base 0 (disabled) is explicitly allowed and skips framebuffer checks
        let mut h_no_fb = h.clone();
        h_no_fb.framebuffer.base_physical_addr = 0;
        assert_eq!(h_no_fb.validate(), Ok(()));
    }
}
