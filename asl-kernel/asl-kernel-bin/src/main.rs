//! ASL Kernel Launcher & Limine Handover Adapter
//! ==============================================
//! Mengorkestrasi Stage-1 Limine -> ASL Boot Handover -> ASL Microkernel Core.

#![no_std]
#![no_main]

mod serial;
use serial::SerialPort;

use asl_bootloader::{
    AslBootHandover, FramebufferConfig, MemoryDescriptor, MemoryRegionType, TargetArch,
    ASL_BOOT_MAGIC, MAX_MEMORY_MAP_ENTRIES,
};

static SERIAL: SerialPort = SerialPort::new();

// -----------------------------------------------------------------------------
// LIMINE BOOT PROTOCOL STRUCTURES (LIMINE BASE PROTOCOL REVISION 2)
// -----------------------------------------------------------------------------

#[repr(C)]
pub struct LimineBaseRevision {
    pub id: [u64; 2],
    pub revision: u64,
}

#[repr(C)]
pub struct LimineMemmapEntry {
    pub base: u64,
    pub length: u64,
    pub typ: u64,
}

pub const LIMINE_MEMMAP_USABLE: u64 = 0;
pub const LIMINE_MEMMAP_RESERVED: u64 = 1;
pub const LIMINE_MEMMAP_ACPI_RECLAIMABLE: u64 = 2;
pub const LIMINE_MEMMAP_ACPI_NVS: u64 = 3;
pub const LIMINE_MEMMAP_BAD_MEMORY: u64 = 4;
pub const LIMINE_MEMMAP_BOOTLOADER_RECLAIMABLE: u64 = 5;
pub const LIMINE_MEMMAP_KERNEL_AND_MODULES: u64 = 6;
pub const LIMINE_MEMMAP_FRAMEBUFFER: u64 = 7;

#[repr(C)]
pub struct LimineMemmapResponse {
    pub revision: u64,
    pub entry_count: u64,
    pub entries: *const *const LimineMemmapEntry,
}

#[repr(C)]
pub struct LimineMemmapRequest {
    pub id: [u64; 4],
    pub revision: u64,
    pub response: *const LimineMemmapResponse,
}
unsafe impl Sync for LimineMemmapRequest {}

#[repr(C)]
pub struct LimineKernelAddressResponse {
    pub revision: u64,
    pub physical_base: u64,
    pub virtual_base: u64,
}

#[repr(C)]
pub struct LimineKernelAddressRequest {
    pub id: [u64; 4],
    pub revision: u64,
    pub response: *const LimineKernelAddressResponse,
}
unsafe impl Sync for LimineKernelAddressRequest {}

#[repr(C)]
pub struct LimineFramebuffer {
    pub address: *mut u8,
    pub width: u64,
    pub height: u64,
    pub pitch: u64,
    pub bpp: u16,
    pub memory_model: u8,
    pub red_mask_size: u8,
    pub red_mask_shift: u8,
    pub green_mask_size: u8,
    pub green_mask_shift: u8,
    pub blue_mask_size: u8,
    pub blue_mask_shift: u8,
    pub unused: [u8; 7],
    pub edid_size: u64,
    pub edid: *const u8,
}

#[repr(C)]
pub struct LimineFramebufferResponse {
    pub revision: u64,
    pub framebuffer_count: u64,
    pub framebuffers: *const *const LimineFramebuffer,
}

#[repr(C)]
pub struct LimineFramebufferRequest {
    pub id: [u64; 4],
    pub revision: u64,
    pub response: *const LimineFramebufferResponse,
}
unsafe impl Sync for LimineFramebufferRequest {}

#[repr(C)]
pub struct LimineHHDMResponse {
    pub revision: u64,
    pub offset: u64,
}

#[repr(C)]
pub struct LimineHHDMRequest {
    pub id: [u64; 4],
    pub revision: u64,
    pub response: *const LimineHHDMResponse,
}
unsafe impl Sync for LimineHHDMRequest {}

// -----------------------------------------------------------------------------
// LIMINE REQUEST ANCHORS (Ditempatkan di section .limine_requests)
// -----------------------------------------------------------------------------

#[used]
#[link_section = ".limine_requests_start"]
static REQUESTS_START_MARKER: [u64; 4] = [
    0xf6b8f4b39de7d1ae,
    0xfab91a6940fcb9cf,
    0x785c6ed015d3e316,
    0x181e920a7852b9d9,
];

#[used]
#[link_section = ".limine_requests"]
static mut BASE_REVISION: [u64; 3] = [0xf9562b2d5c95a6c8, 0x6a7b384944536bdc, 2];

#[used]
#[link_section = ".limine_requests"]
static mut MEMMAP_REQUEST: LimineMemmapRequest = LimineMemmapRequest {
    id: [
        0xc7b1dd30df4c8b88,
        0x0a82e883a194f07b,
        0x67cf3d9d378a806f,
        0xe304acdfc50c3c62,
    ],
    revision: 0,
    response: core::ptr::null(),
};

#[used]
#[link_section = ".limine_requests"]
static mut KERNEL_ADDR_REQUEST: LimineKernelAddressRequest = LimineKernelAddressRequest {
    id: [
        0xc7b1dd30df4c8b88,
        0x0a82e883a194f07b,
        0x71ba76863cc55f63,
        0xb2644a48c516a487,
    ],
    revision: 0,
    response: core::ptr::null(),
};

#[used]
#[link_section = ".limine_requests"]
static mut FRAMEBUFFER_REQUEST: LimineFramebufferRequest = LimineFramebufferRequest {
    id: [
        0xc7b1dd30df4c8b88,
        0x0a82e883a194f07b,
        0x9d5827dcd881dd75,
        0xa3148604f6fab11b,
    ],
    revision: 0,
    response: core::ptr::null(),
};

#[used]
#[link_section = ".limine_requests"]
static mut HHDM_REQUEST: LimineHHDMRequest = LimineHHDMRequest {
    id: [
        0xc7b1dd30df4c8b88,
        0x0a82e883a194f07b,
        0x48dcf1cb8ad2b852,
        0x63984e959a98244b,
    ],
    revision: 0,
    response: core::ptr::null(),
};

#[used]
#[link_section = ".limine_requests_end"]
static REQUESTS_END_MARKER: [u64; 2] = [0xadc0e0531bb10d03, 0x9572709f31764c62];

// -----------------------------------------------------------------------------
// STATIC HANDOVER STORAGE (BSS)
// -----------------------------------------------------------------------------

static mut ASL_HANDOVER_STORAGE: AslBootHandover = AslBootHandover {
    magic: ASL_BOOT_MAGIC,
    protocol_version: 1,
    target_arch: TargetArch::X86_64,
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
};

// -----------------------------------------------------------------------------
// DEDICATED ASSEMBLY ENTRY STUB
// -----------------------------------------------------------------------------

#[cfg(target_arch = "x86_64")]
core::arch::global_asm!(
    ".section .text.entry, \"ax\"",
    ".global _start",
    "_start:",
    "cli",
    "lea rsp, [rip + _asl_stack_top]",
    "and rsp, -16",
    "call asl_launcher_main",
    "2:",
    "hlt",
    "jmp 2b",
    ".section .bss",
    ".align 4096",
    // Guard pad 4KB tepat di bawah stack: PTE-nya di-clear saat boot (stack guard page)
    ".global _asl_guard_page",
    "_asl_guard_page:",
    ".skip 4096",
    ".align 4096",
    ".global _asl_stack_bottom",
    "_asl_stack_bottom:",
    ".skip 524288",
    ".global _asl_stack_top",
    "_asl_stack_top:"
);

#[cfg(target_arch = "aarch64")]
core::arch::global_asm!(
    ".section .text.entry, \"ax\"",
    ".global _start",
    "_start:",
    "adrp x0, _asl_stack_top",
    "add  sp, x0, :lo12:_asl_stack_top",
    "bl   asl_launcher_aarch64",
    "1:",
    "wfe",
    "b    1b",
    ".section .bss",
    ".align 16",
    ".global _asl_guard_page",
    "_asl_guard_page:",
    ".skip 4096",
    ".align 16",
    ".global _asl_stack_bottom",
    "_asl_stack_bottom:",
    ".skip 524288",
    ".global _asl_stack_top",
    "_asl_stack_top:"
);

// -----------------------------------------------------------------------------
// ASL LAUNCHER MAIN & HANDOVER ADAPTER
// -----------------------------------------------------------------------------

#[cfg(target_arch = "aarch64")]
#[no_mangle]
pub extern "C" fn asl_launcher_aarch64() -> ! {
    SERIAL.init();
    SERIAL.write_str("\n[ASL-LAUNCHER-AARCH64] Entered AArch64 Native Entry Stub.\n");
    SERIAL.write_str("[ASL-LAUNCHER-AARCH64] Target: ARMv8-A Cortex-A57 (QEMU virt bare-metal).\n");

    // Persiapkan Handover untuk QEMU virt platform
    // QEMU virt: RAM 0x4000_0000..0x4400_0000 (64MB)
    // Kernel base: 0x4008_0000 (size 2MB)
    unsafe {
        ASL_HANDOVER_STORAGE.magic = ASL_BOOT_MAGIC;
        ASL_HANDOVER_STORAGE.protocol_version = 1;
        ASL_HANDOVER_STORAGE.target_arch = TargetArch::Aarch64;
        ASL_HANDOVER_STORAGE.kernel_physical_base = 0x4008_0000;
        ASL_HANDOVER_STORAGE.kernel_virtual_base = 0x4008_0000;
        ASL_HANDOVER_STORAGE.kernel_size_bytes = 0x200000;
        ASL_HANDOVER_STORAGE.device_tree_or_acpi_ptr = 0x4000_0000; // QEMU puts DTB at start of RAM

        // Konfigurasi Memory Map
        ASL_HANDOVER_STORAGE.memory_map_count = 2;
        ASL_HANDOVER_STORAGE.memory_map[0] = MemoryDescriptor {
            base_physical_addr: 0x4008_0000,
            length_bytes: 0x200000,
            region_type: MemoryRegionType::KernelImage,
            flags: 0,
        };
        ASL_HANDOVER_STORAGE.memory_map[1] = MemoryDescriptor {
            base_physical_addr: 0x4028_0000,
            length_bytes: 60 * 1024 * 1024, // 60MB usable RAM
            region_type: MemoryRegionType::UsableRam,
            flags: 0,
        };
    }

    SERIAL
        .write_str("[ASL-LAUNCHER-AARCH64] Handover populated. Jumping to _asl_kernel_entry...\n");

    let handover_ptr = core::ptr::addr_of!(ASL_HANDOVER_STORAGE);
    unsafe {
        asl_kernel_core::_asl_kernel_entry(handover_ptr);
    }
}

#[cfg(target_arch = "x86_64")]
#[no_mangle]
pub extern "C" fn asl_launcher_main() -> ! {
    SERIAL.init();
    SERIAL.write_str("\n[ASL-LAUNCHER] M1: Entered Limine Entry Stub (profile=");
    SERIAL.write_str(option_env!("ASL_BUILD_PROFILE").unwrap_or("unknown"));
    SERIAL.write_str(") via Assembly Trampoline.\n");

    let memmap_resp =
        unsafe { core::ptr::read_volatile(core::ptr::addr_of!(MEMMAP_REQUEST.response)) };
    let kaddr_resp =
        unsafe { core::ptr::read_volatile(core::ptr::addr_of!(KERNEL_ADDR_REQUEST.response)) };

    let (kphys, kvirt) = if !kaddr_resp.is_null() {
        let resp = unsafe { &*kaddr_resp };
        (resp.physical_base, resp.virtual_base)
    } else {
        (0x100000, 0xFFFF_8000_0010_0000)
    };

    let ksize = 0x200000; // 2MB kernel reservation window

    unsafe {
        ASL_HANDOVER_STORAGE.magic = ASL_BOOT_MAGIC;
        ASL_HANDOVER_STORAGE.protocol_version = 1;
        ASL_HANDOVER_STORAGE.target_arch = TargetArch::X86_64;
        ASL_HANDOVER_STORAGE.kernel_physical_base = kphys;
        ASL_HANDOVER_STORAGE.kernel_virtual_base = kvirt;
        ASL_HANDOVER_STORAGE.kernel_size_bytes = ksize;
        ASL_HANDOVER_STORAGE.memory_map_count = 0;
    }

    // Adaptasi Limine Memmap -> AslBootHandover
    if !memmap_resp.is_null() {
        let resp = unsafe { &*memmap_resp };
        let entries_ptr = resp.entries;
        let count = resp.entry_count as usize;

        let mut out_idx = 0;
        let mut i = 0;

        while i < count && out_idx < MAX_MEMORY_MAP_ENTRIES {
            let entry_ptr = unsafe { *entries_ptr.add(i) };
            if !entry_ptr.is_null() {
                let entry = unsafe { &*entry_ptr };
                SERIAL.write_str("  [MEMMAP] base: ");
                SERIAL.write_u64(entry.base);
                SERIAL.write_str(" len: ");
                SERIAL.write_u64(entry.length);
                SERIAL.write_str(" typ: ");
                SERIAL.write_u64(entry.typ);
                SERIAL.write_str("\n");

                let reg_type = match entry.typ {
                    LIMINE_MEMMAP_USABLE => MemoryRegionType::UsableRam,
                    LIMINE_MEMMAP_RESERVED => MemoryRegionType::Reserved,
                    LIMINE_MEMMAP_ACPI_RECLAIMABLE => MemoryRegionType::AcpiReclaimable,
                    LIMINE_MEMMAP_ACPI_NVS => MemoryRegionType::AcpiNvs,
                    LIMINE_MEMMAP_BAD_MEMORY => MemoryRegionType::BadMemory,
                    LIMINE_MEMMAP_BOOTLOADER_RECLAIMABLE => MemoryRegionType::BootloaderReclaimable,
                    LIMINE_MEMMAP_KERNEL_AND_MODULES => MemoryRegionType::KernelImage,
                    LIMINE_MEMMAP_FRAMEBUFFER => MemoryRegionType::FrameBuffer,
                    _ => MemoryRegionType::Reserved,
                };

                // Pengecualian Region Kernel dari Usable RAM (Anti Self-Overwrite)
                if reg_type == MemoryRegionType::UsableRam
                    && entry.base <= kphys
                    && (entry.base + entry.length) >= (kphys + ksize)
                {
                    // Memecah region usable menjadi segmen sebelum dan sesudah kernel
                    if kphys > entry.base {
                        unsafe {
                            ASL_HANDOVER_STORAGE.memory_map[out_idx] = MemoryDescriptor {
                                base_physical_addr: entry.base,
                                length_bytes: kphys - entry.base,
                                region_type: MemoryRegionType::UsableRam,
                                flags: 0,
                            };
                            out_idx += 1;
                        }
                    }

                    if out_idx < MAX_MEMORY_MAP_ENTRIES {
                        unsafe {
                            ASL_HANDOVER_STORAGE.memory_map[out_idx] = MemoryDescriptor {
                                base_physical_addr: kphys,
                                length_bytes: ksize,
                                region_type: MemoryRegionType::KernelImage,
                                flags: 0,
                            };
                            out_idx += 1;
                        }
                    }

                    let after_start = kphys + ksize;
                    let after_end = entry.base + entry.length;
                    if after_end > after_start && out_idx < MAX_MEMORY_MAP_ENTRIES {
                        unsafe {
                            ASL_HANDOVER_STORAGE.memory_map[out_idx] = MemoryDescriptor {
                                base_physical_addr: after_start,
                                length_bytes: after_end - after_start,
                                region_type: MemoryRegionType::UsableRam,
                                flags: 0,
                            };
                            out_idx += 1;
                        }
                    }
                } else {
                    unsafe {
                        ASL_HANDOVER_STORAGE.memory_map[out_idx] = MemoryDescriptor {
                            base_physical_addr: entry.base,
                            length_bytes: entry.length,
                            region_type: reg_type,
                            flags: 0,
                        };
                        out_idx += 1;
                    }
                }
            }
            i += 1;
        }

        unsafe {
            ASL_HANDOVER_STORAGE.memory_map_count = out_idx;
        }
    } else {
        // Fallback default memory map untuk simulasi/bare host bila memmap response null
        unsafe {
            ASL_HANDOVER_STORAGE.memory_map_count = 2;
            ASL_HANDOVER_STORAGE.memory_map[0] = MemoryDescriptor {
                base_physical_addr: kphys,
                length_bytes: ksize,
                region_type: MemoryRegionType::KernelImage,
                flags: 0,
            };
            ASL_HANDOVER_STORAGE.memory_map[1] = MemoryDescriptor {
                base_physical_addr: 0x1000000,
                length_bytes: 32 * 1024 * 1024, // 32MB Usable RAM
                region_type: MemoryRegionType::UsableRam,
                flags: 0,
            };
        }
    }

    SERIAL.write_str("[ASL-LAUNCHER] Inspecting Limine Requests:\n");
    let base_rev_supported =
        unsafe { core::ptr::read_volatile(core::ptr::addr_of!(BASE_REVISION[2])) };
    if base_rev_supported == 0 {
        SERIAL.write_str("  -> Base Revision supported.\n");
    } else {
        SERIAL.write_str("  -> Base Revision unsupported or unchanged.\n");
    }

    if !memmap_resp.is_null() {
        SERIAL.write_str("  -> Memmap response received.\n");
    } else {
        SERIAL.write_str("  -> Memmap response NULL.\n");
    }

    let hhdm_resp = unsafe { core::ptr::read_volatile(core::ptr::addr_of!(HHDM_REQUEST.response)) };
    let _hhdm_offset = if !hhdm_resp.is_null() {
        SERIAL.write_str("  -> HHDM response received.\n");
        unsafe { (*hhdm_resp).offset }
    } else {
        SERIAL.write_str("  -> HHDM response NULL.\n");
        0
    };

    let fb_resp =
        unsafe { core::ptr::read_volatile(core::ptr::addr_of!(FRAMEBUFFER_REQUEST.response)) };
    if !fb_resp.is_null() {
        SERIAL.write_str("  -> Framebuffer response received.\n");
        let resp = unsafe { &*fb_resp };
        if resp.framebuffer_count > 0 && !resp.framebuffers.is_null() {
            let first_fb_ptr = unsafe { *resp.framebuffers };
            if !first_fb_ptr.is_null() {
                let fb = unsafe { &*first_fb_ptr };
                let bytes_pp = (fb.bpp / 8) as u8;
                let ppsl = if bytes_pp > 0 {
                    (fb.pitch / bytes_pp as u64) as u32
                } else {
                    fb.width as u32
                };
                let total_size = fb.pitch * fb.height;

                let hhdm = if !hhdm_resp.is_null() {
                    unsafe { (*hhdm_resp).offset }
                } else {
                    0xFFFF_8000_0000_0000
                };
                let raw_addr = fb.address as u64;
                // Limine returns virtual address in higher-half direct map (HHDM)
                let phys_addr = if raw_addr >= hhdm {
                    raw_addr - hhdm
                } else {
                    raw_addr
                };

                unsafe {
                    ASL_HANDOVER_STORAGE.framebuffer = FramebufferConfig {
                        base_physical_addr: phys_addr,
                        size_bytes: total_size,
                        width: fb.width as u32,
                        height: fb.height as u32,
                        pixels_per_scan_line: ppsl,
                        bytes_per_pixel: bytes_pp,
                    };
                }
                SERIAL.write_str("  -> Framebuffer configured successfully.\n");
            }
        }
    } else {
        SERIAL.write_str("  -> Framebuffer response NULL.\n");
    }

    SERIAL.write_str("[ASL-LAUNCHER] M2: Handover Adapted & Memory Map Sanitized.\n");
    SERIAL.write_str("[ASL-LAUNCHER] M3: Transferring Control to ASL Microkernel Core Entry.\n");

    let handover_ptr = core::ptr::addr_of!(ASL_HANDOVER_STORAGE);

    // Lompat ke Entry Point Resmi ASL Core Microkernel
    unsafe {
        asl_kernel_core::_asl_kernel_entry(handover_ptr);
    }
}

// Panic handler disediakan oleh crate dependency `asl-kernel-core`
