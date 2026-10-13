//! ASL Kernel Launcher & Limine Handover Adapter
//! ==============================================
//! Mengorkestrasi Stage-1 Limine -> ASL Boot Handover -> ASL Microkernel Core.

#![no_std]
#![no_main]

mod serial;
use serial::SerialPort;

use asl_bootloader::{
    AslBootHandover, TargetArch, MemoryDescriptor, MemoryRegionType, FramebufferConfig,
    MAX_MEMORY_MAP_ENTRIES, ASL_BOOT_MAGIC,
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

// -----------------------------------------------------------------------------
// LIMINE REQUEST ANCHORS (Ditempatkan di section .limine_requests)
// -----------------------------------------------------------------------------

#[used]
#[link_section = ".limine_requests"]
static BASE_REVISION: LimineBaseRevision = LimineBaseRevision {
    id: [0xf9562b2d3616ae62, 0x831252811a66e404],
    revision: 2,
};

#[used]
#[link_section = ".limine_requests"]
static MEMMAP_REQUEST: LimineMemmapRequest = LimineMemmapRequest {
    id: [0xc7b1dd30df4c8b88, 0x0a82e883a194f07b, 0x67cf3d9d378a15e6, 0x8d5004882da0f324],
    revision: 0,
    response: core::ptr::null(),
};

#[used]
#[link_section = ".limine_requests"]
static KERNEL_ADDR_REQUEST: LimineKernelAddressRequest = LimineKernelAddressRequest {
    id: [0xc7b1dd30df4c8b88, 0x0a82e883a194f07b, 0x71ba76863cc55f63, 0xb2644a48c516a487],
    revision: 0,
    response: core::ptr::null(),
};

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
    "_asl_stack_bottom:",
    ".skip 524288",
    "_asl_stack_top:"
);

// -----------------------------------------------------------------------------
// ASL LAUNCHER MAIN & HANDOVER ADAPTER
// -----------------------------------------------------------------------------

#[no_mangle]
pub extern "C" fn asl_launcher_main() -> ! {
    SERIAL.init();
    SERIAL.write_str("\n[ASL-LAUNCHER] M1: Entered Limine Entry Stub via Assembly Trampoline.\n");

    let memmap_resp = MEMMAP_REQUEST.response;
    let kaddr_resp = KERNEL_ADDR_REQUEST.response;

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

    SERIAL.write_str("[ASL-LAUNCHER] M2: Handover Adapted & Memory Map Sanitized.\n");
    SERIAL.write_str("[ASL-LAUNCHER] M3: Transferring Control to ASL Microkernel Core Entry.\n");

    let handover_ptr = core::ptr::addr_of!(ASL_HANDOVER_STORAGE);

    // Lompat ke Entry Point Resmi ASL Core Microkernel
    unsafe {
        asl_kernel_core::_asl_kernel_entry(handover_ptr);
    }
}

// Panic handler disediakan oleh crate dependency `asl-kernel-core`
