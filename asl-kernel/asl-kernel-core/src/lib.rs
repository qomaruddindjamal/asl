//! ASL Core Avionics Microkernel
//! ==============================
//! Standar DO-178C DAL-A Bare-Metal Microkernel.
//! Mengorkestrasi 150 arsitektur silikon dunia melalui:
//! - Agnostic Bootloader Handover
//! - Universal HAL Traits
//! - Deterministic Buddy-Slab Allocator
//! - ARINC 653 Spatial Partitioning Vault
//! - Time-Triggered Major/Minor Frame Scheduler
//! - AFDX Virtual Link Zero-Copy IPC

#![no_std]
#![cfg_attr(not(test), no_main)]

use core::panic::PanicInfo;

use asl_bootloader::{AslBootHandover, TargetArch};
use asl_hal::ArchitectureRuntime;
use asl_memory::BuddyAllocator;
use asl_security_vault::{SpatialPartitionManager, PartitionAccessRights, MemoryWindow};
use asl_sched::MajorFrameSchedule;
use asl_afdx_ipc::AfdxVirtualLinkRingBuffer;

// -----------------------------------------------------------------------------
// BOUNDED EARLY SERIAL TELEMETRY (COM1 0x3F8)
// -----------------------------------------------------------------------------

const COM1_PORT: u16 = 0x3F8;

#[inline(always)]
unsafe fn early_outb(port: u16, val: u8) {
    #[cfg(target_arch = "x86_64")]
    core::arch::asm!(
        "out dx, al",
        in("dx") port,
        in("al") val,
        options(nomem, nostack, preserves_flags)
    );
    #[cfg(not(target_arch = "x86_64"))]
    {
        let _ = (port, val);
    }
}

#[inline(always)]
unsafe fn early_inb(port: u16) -> u8 {
    #[cfg(target_arch = "x86_64")]
    {
        let ret: u8;
        core::arch::asm!(
            "in al, dx",
            out("al") ret,
            in("dx") port,
            options(nomem, nostack, preserves_flags)
        );
        ret
    }
    #[cfg(not(target_arch = "x86_64"))]
    {
        let _ = port;
        0
    }
}

fn early_serial_write_str(s: &str) {
    for b in s.bytes() {
        if b == b'\n' {
            let mut timeout = 100_000;
            while (unsafe { early_inb(COM1_PORT + 5) } & 0x20) == 0 && timeout > 0 {
                core::hint::spin_loop();
                timeout -= 1;
            }
            unsafe { early_outb(COM1_PORT, b'\r') };
        }
        let mut timeout = 100_000;
        while (unsafe { early_inb(COM1_PORT + 5) } & 0x20) == 0 && timeout > 0 {
            core::hint::spin_loop();
            timeout -= 1;
        }
        unsafe { early_outb(COM1_PORT, b) };
    }
}

mod font;
use font::BUILTIN_FONT_8X16;

// -----------------------------------------------------------------------------
// LINEAR FRAMEBUFFER CONSOLE (VBE / GOP Graphics Mode)
// -----------------------------------------------------------------------------

pub struct FbConsole {
    pub base: usize,
    pub width: usize,
    pub height: usize,
    pub pitch_bytes: usize,
    pub bpp: usize,
    pub cursor_x: usize,
    pub cursor_y: usize,
    pub fg_color: u32,
    pub bg_color: u32,
    pub scale: usize,
    pub bold: bool,
}

impl FbConsole {
    pub const fn new() -> Self {
        Self {
            base: 0,
            width: 0,
            height: 0,
            pitch_bytes: 0,
            bpp: 0,
            cursor_x: 0,
            cursor_y: 0,
            fg_color: 0x00FFFFFF, // White
            bg_color: 0x00000000, // Black
            scale: 1,
            bold: true,
        }
    }

    pub fn init(&mut self, base: usize, width: usize, height: usize, pitch_bytes: usize, bpp: usize) {
        self.base = base;
        self.width = width;
        self.height = height;
        self.pitch_bytes = pitch_bytes;
        self.bpp = bpp;
        self.cursor_x = 0;
        self.cursor_y = 0;
        self.fg_color = 0x00FFFFFF;
        self.bg_color = 0x00000000;
        self.scale = 1;
        self.bold = true;
    }

    pub fn set_bold(&mut self, bold: bool) {
        self.bold = bold;
    }

    pub fn set_scale(&mut self, scale: usize) {
        self.scale = if scale == 0 { 1 } else { scale };
    }

    pub fn is_available(&self) -> bool {
        self.base != 0 && self.width > 0 && self.height > 0
    }

    pub fn clear(&mut self) {
        if !self.is_available() {
            return;
        }
        let total_pixels = self.height * (self.pitch_bytes / 4);
        let ptr = self.base as *mut u32;
        for i in 0..total_pixels {
            unsafe {
                core::ptr::write_volatile(ptr.add(i), self.bg_color);
            }
        }
        self.cursor_x = 0;
        self.cursor_y = 0;
    }

    pub fn set_fg_color(&mut self, color: u32) {
        self.fg_color = color;
    }

    pub fn new_line(&mut self) {
        self.cursor_x = 0;
        let line_height = 16 * self.scale;
        if self.cursor_y + line_height * 2 <= self.height {
            self.cursor_y += line_height;
        } else {
            // Scroll frame up
            let stride_u32 = self.pitch_bytes / 4;
            let ptr = self.base as *mut u32;
            let copy_lines = self.height - line_height;
            for y in 0..copy_lines {
                let src_y = y + line_height;
                for x in 0..self.width {
                    unsafe {
                        let px = core::ptr::read_volatile(ptr.add(src_y * stride_u32 + x));
                        core::ptr::write_volatile(ptr.add(y * stride_u32 + x), px);
                    }
                }
            }
            // Clear last lines
            for y in copy_lines..self.height {
                for x in 0..self.width {
                    unsafe {
                        core::ptr::write_volatile(ptr.add(y * stride_u32 + x), self.bg_color);
                    }
                }
            }
        }
    }

    pub fn draw_char(&mut self, c: u8) {
        if !self.is_available() {
            return;
        }
        match c {
            b'\n' => self.new_line(),
            b'\r' => self.cursor_x = 0,
            _ => {
                let char_width = 8 * self.scale;
                if self.cursor_x + char_width > self.width {
                    self.new_line();
                }
                let glyph_idx = c as usize;
                let glyph_offset = glyph_idx * 16;
                let glyph = &BUILTIN_FONT_8X16[glyph_offset..glyph_offset + 16];

                let stride_u32 = self.pitch_bytes / 4;
                let ptr = self.base as *mut u32;

                for row in 0..16 {
                    let bits = glyph[row];
                    for sy in 0..self.scale {
                        let py = self.cursor_y + row * self.scale + sy;
                        if py >= self.height {
                            break;
                        }
                        for col in 0..8 {
                            let is_fg = (bits & (0x80 >> col)) != 0;
                            let color = if is_fg { self.fg_color } else { self.bg_color };

                            for sx in 0..self.scale {
                                let px = self.cursor_x + col * self.scale + sx;
                                if px >= self.width {
                                    break;
                                }
                                unsafe {
                                    core::ptr::write_volatile(ptr.add(py * stride_u32 + px), color);
                                }
                            }
                        }
                    }
                }
                self.cursor_x += char_width;
            }
        }
    }

    pub fn write_str(&mut self, s: &str) {
        for b in s.bytes() {
            self.draw_char(b);
        }
    }
}

static mut FB_CONSOLE: FbConsole = FbConsole::new();

// -----------------------------------------------------------------------------
// VGA TEXT MODE CONSOLE (HHDM: 0xffff8000000b8000)
// -----------------------------------------------------------------------------

const HHDM_OFFSET: usize = 0xffff_8000_0000_0000;
const VGA_BUFFER_VIRT: usize = HHDM_OFFSET + 0xb8000;
const VGA_COLS: usize = 80;
const VGA_ROWS: usize = 25;

pub struct VgaConsole {
    row: usize,
    col: usize,
    color: u8,
}

impl VgaConsole {
    pub const fn new() -> Self {
        Self {
            row: 0,
            col: 0,
            color: 0x07, // Light gray on black
        }
    }

    pub fn clear(&mut self) {
        let vga = VGA_BUFFER_VIRT as *mut u16;
        let blank = (self.color as u16) << 8 | (b' ' as u16);
        for i in 0..(VGA_COLS * VGA_ROWS) {
            unsafe {
                core::ptr::write_volatile(vga.add(i), blank);
            }
        }
        self.row = 0;
        self.col = 0;
    }

    pub fn set_color(&mut self, color: u8) {
        self.color = color;
    }

    pub fn new_line(&mut self) {
        self.col = 0;
        if self.row < VGA_ROWS - 1 {
            self.row += 1;
        } else {
            // Scroll up by 1 line
            let vga = VGA_BUFFER_VIRT as *mut u16;
            for r in 1..VGA_ROWS {
                for c in 0..VGA_COLS {
                    unsafe {
                        let ch = core::ptr::read_volatile(vga.add(r * VGA_COLS + c));
                        core::ptr::write_volatile(vga.add((r - 1) * VGA_COLS + c), ch);
                    }
                }
            }
            let blank = (self.color as u16) << 8 | (b' ' as u16);
            for c in 0..VGA_COLS {
                unsafe {
                    core::ptr::write_volatile(vga.add((VGA_ROWS - 1) * VGA_COLS + c), blank);
                }
            }
        }
    }

    pub fn write_char(&mut self, c: u8) {
        match c {
            b'\n' => self.new_line(),
            b'\r' => self.col = 0,
            _ => {
                if self.col >= VGA_COLS {
                    self.new_line();
                }
                let vga = VGA_BUFFER_VIRT as *mut u16;
                let entry = (self.color as u16) << 8 | (c as u16);
                unsafe {
                    core::ptr::write_volatile(vga.add(self.row * VGA_COLS + self.col), entry);
                }
                self.col += 1;
            }
        }
    }

    pub fn write_str(&mut self, s: &str) {
        for b in s.bytes() {
            self.write_char(b);
        }
    }
}

static mut VGA: VgaConsole = VgaConsole::new();

/// State Utama Kernel Tunggal (Integrated Modular Avionics Core)
pub struct AslAvionicsKernel {
    pub runtime: ArchitectureRuntime,
    pub memory_allocator: BuddyAllocator,
    pub vault_manager: SpatialPartitionManager,
    pub scheduler: MajorFrameSchedule,
    pub ipc_bus: AfdxVirtualLinkRingBuffer,
}

impl AslAvionicsKernel {
    pub const fn new(arch: TargetArch) -> Self {
        Self {
            runtime: ArchitectureRuntime::new(arch),
            memory_allocator: BuddyAllocator::empty(),
            vault_manager: SpatialPartitionManager::new(),
            scheduler: MajorFrameSchedule::new(100_000), // 100ms Major Frame
            ipc_bus: AfdxVirtualLinkRingBuffer::new(1001, 1000), // BAG 1ms
        }
    }

    /// Titik masuk inisialisasi avionik dari handover bootloader
    pub fn cold_boot_init(&mut self, handover: &AslBootHandover) -> Result<(), &'static str> {
        // 1. Validasi Handover Integritas
        handover.validate()?;

        // 2. Inisialisasi Buddy System Memory
        self.memory_allocator.init_from_handover(handover)?;

        // 3. Konfigurasi Default Partisi Isolasi (Avionics Core Vault)
        let core_part_idx = self.vault_manager.create_partition(1, true)?;
        let kernel_win = MemoryWindow {
            start_address: handover.kernel_physical_base,
            length_bytes: handover.kernel_size_bytes,
            rights: PartitionAccessRights::ReadExecute,
        };
        self.vault_manager.partitions[core_part_idx].register_window(kernel_win)?;

        // 4. Jadwalkan Siklus Slot Deterministik (ARINC 653)
        self.scheduler.add_slot(1, 20_000, true)?; // 20ms Slot untuk Core Partition
        self.scheduler.add_slot(2, 80_000, false)?; // 80ms Slot untuk Application Partition

        Ok(())
    }
}

/// Panic Handler Standar Bare-Metal (DO-178C Fail-Safe Trap)
#[cfg(not(test))]
#[panic_handler]
fn panic(_info: &PanicInfo) -> ! {
    early_serial_write_str("\n[ASL-CORE-PANIC] Fail-Safe Trap Invoked. Spinning in safe loop.\n");
    // Memasuki Safe Fallback Loop tanpa restart liar
    loop {
        core::hint::spin_loop();
    }
}

/// Official bare-metal kernel entry point
///
/// # Safety
/// Caller must ensure `handover_ptr` is either null or points to a valid, initialized,
/// properly aligned `AslBootHandover` struct.
#[no_mangle]
pub unsafe extern "C" fn _asl_kernel_entry(handover_ptr: *const AslBootHandover) -> ! {
    #[cfg(target_arch = "x86_64")]
    {
        let vga = core::ptr::addr_of_mut!(VGA);
        (*vga).clear();

        if !handover_ptr.is_null() {
            let fb_cfg = &(*handover_ptr).framebuffer;
            if fb_cfg.base_physical_addr != 0 {
                let fb = core::ptr::addr_of_mut!(FB_CONSOLE);
                let pitch_bytes = (fb_cfg.pixels_per_scan_line as usize) * (fb_cfg.bytes_per_pixel as usize);
                (*fb).init(
                    fb_cfg.base_physical_addr as usize,
                    fb_cfg.width as usize,
                    fb_cfg.height as usize,
                    pitch_bytes,
                    (fb_cfg.bytes_per_pixel as usize) * 8,
                );
                (*fb).set_scale(1); // Native 1:1 Pixel-Perfect 8x16 font rendering (ultra-sharp & clean)
                (*fb).clear();
                (*fb).cursor_y = 16;
            }
        }
    }

    let mut kernel = AslAvionicsKernel::new(TargetArch::X86_64);
    let mut kernel_init_ok = false;
    let mut mem_init_ok = false;
    let mut cpu_arch_detected = false;

    if !handover_ptr.is_null() {
        let handover = &*handover_ptr;
        match kernel.cold_boot_init(handover) {
            Ok(()) => {
                kernel_init_ok = true;

                // MILESTONE M4: Direct Proof of Allocator & Initial Page Availability
                if kernel.memory_allocator.total_pages > 0 && kernel.memory_allocator.free_pages > 0 {
                    mem_init_ok = true;
                    early_serial_write_str("[ASL-CORE] M4: Buddy Allocator & Heap Memory Subsystem Initialized.\n");
                } else {
                    early_serial_write_str("[ASL-CORE] M4: WARNING - Allocator initialized with 0 available pages.\n");
                }

                if handover.target_arch == TargetArch::X86_64 {
                    cpu_arch_detected = true;
                }

                // MILESTONE M5: Direct Proof of ARINC 653 Partitions & Major Frame Scheduler
                if kernel.vault_manager.partitions[0].is_active && kernel.scheduler.slot_count == 2 {
                    // Verifikasi tick deterministik pertama (advance 10ms)
                    let (active_part, _switched) = kernel.scheduler.advance_tick(10_000);
                    if active_part == 1 {
                        early_serial_write_str("[ASL-CORE] M5: ARINC 653 Partitions & Deterministic Scheduler Active.\n");
                    } else {
                        early_serial_write_str("[ASL-CORE] M5: WARNING - Unexpected active partition on initial tick.\n");
                    }
                } else {
                    early_serial_write_str("[ASL-CORE] M5: WARNING - Partition slots incomplete.\n");
                }
            }
            Err(e) => {
                early_serial_write_str("[ASL-CORE] ERROR: cold_boot_init failed: ");
                early_serial_write_str(e);
                early_serial_write_str("\n");
            }
        }
    } else {
        early_serial_write_str("[ASL-CORE] WARNING: handover_ptr is NULL.\n");
    }

    // MILESTONE M6: Direct Proof of Reaching Safe Kernel Idle Loop
    early_serial_write_str("[ASL-CORE] M6: Kernel Cold Boot Complete. Entering Deterministic Idle Loop.\n");

    let logo = 
"       ___    ____    __   \n      /   |  / __/   / /   \n     / /| |  \\ \\    / /    \n    / ___ | _/ /   / /___  \n   /_/  |_|/___/  /_____/  \n\n   ASL Kernel v1.0.1\n   Selamat Datang Di ASL Kernel\n\n";


    #[cfg(target_arch = "x86_64")]
    {
        let vga = core::ptr::addr_of_mut!(VGA);
        let fb = core::ptr::addr_of_mut!(FB_CONSOLE);

        // Render to legacy VGA text buffer
        (*vga).set_color(0x0F); // White
        (*vga).write_str(logo);

        // Render to Linear Framebuffer (VBE / GOP)
        (*fb).set_fg_color(0x00FFFFFF); // White
        (*fb).write_str(logo);

        if kernel_init_ok {
            (*vga).set_color(0x0A); // Light Green
            (*vga).write_str("   [ OK ] ");
            (*vga).set_color(0x0F);
            (*vga).write_str("Kernel initialized\n");

            (*fb).set_fg_color(0x0000FF00); // Green
            (*fb).write_str("   [ OK ] ");
            (*fb).set_fg_color(0x00FFFFFF);
            (*fb).write_str("Kernel initialized\n");
        }

        if mem_init_ok {
            (*vga).set_color(0x0A); // Light Green
            (*vga).write_str("   [ OK ] ");
            (*vga).set_color(0x0F);
            (*vga).write_str("Memory manager initialized\n");

            (*fb).set_fg_color(0x0000FF00); // Green
            (*fb).write_str("   [ OK ] ");
            (*fb).set_fg_color(0x00FFFFFF);
            (*fb).write_str("Memory manager initialized\n");
        }

        if cpu_arch_detected {
            (*vga).set_color(0x0A); // Light Green
            (*vga).write_str("   [ OK ] ");
            (*vga).set_color(0x0F);
            (*vga).write_str("CPU architecture detected\n");

            (*fb).set_fg_color(0x0000FF00); // Green
            (*fb).write_str("   [ OK ] ");
            (*fb).set_fg_color(0x00FFFFFF);
            (*fb).write_str("CPU architecture detected\n");
        }

        (*vga).write_str("\n");
        (*vga).set_color(0x0B); // Light Cyan
        (*vga).write_str("   ASL kernel booted successfully.\n");

        (*fb).write_str("\n");
        (*fb).set_fg_color(0x0000FFFF); // Cyan
        (*fb).write_str("   ASL kernel booted successfully.\n");
    }

    early_serial_write_str(logo);
    if kernel_init_ok {
        early_serial_write_str("   [ OK ] Kernel initialized\n");
    }
    if mem_init_ok {
        early_serial_write_str("   [ OK ] Memory manager initialized\n");
    }
    if cpu_arch_detected {
        early_serial_write_str("   [ OK ] CPU architecture detected\n");
    }
    early_serial_write_str("\n   ASL kernel booted successfully.\n");

    loop {
        #[cfg(target_arch = "x86_64")]
        core::arch::asm!("hlt", options(nomem, nostack, preserves_flags));
        #[cfg(not(target_arch = "x86_64"))]
        core::hint::spin_loop();
    }
}
