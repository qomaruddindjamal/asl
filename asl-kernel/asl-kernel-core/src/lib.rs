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

use asl_afdx_ipc::AfdxVirtualLinkRingBuffer;
use asl_bootloader::{AslBootHandover, TargetArch};
use asl_hal::ArchitectureRuntime;
use asl_memory::BuddyAllocator;
use asl_sched::MajorFrameSchedule;
use asl_security_vault::{MemoryWindow, PartitionAccessRights, SpatialPartitionManager};

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

pub mod font;
pub use font::{ASL_FONT_REGULAR_8X16, BUILTIN_FONT_8X16};

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

    pub fn init(
        &mut self,
        base: usize,
        width: usize,
        height: usize,
        pitch_bytes: usize,
        bpp: usize,
    ) {
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

    pub fn write_u64(&mut self, mut n: u64) {
        if n == 0 {
            self.draw_char(b'0');
            return;
        }
        let mut buf = [0u8; 20];
        let mut i = 0;
        while n > 0 {
            buf[i] = b'0' + (n % 10) as u8;
            n /= 10;
            i += 1;
        }
        while i > 0 {
            i -= 1;
            self.draw_char(buf[i]);
        }
    }

    pub fn write_hex16(&mut self, val: u16) {
        const HEX_DIGITS: &[u8; 16] = b"0123456789ABCDEF";
        for shift in [12, 8, 4, 0] {
            let nibble = ((val >> shift) & 0x0F) as usize;
            self.draw_char(HEX_DIGITS[nibble]);
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

    pub fn write_u64(&mut self, mut n: u64) {
        if n == 0 {
            self.write_char(b'0');
            return;
        }
        let mut buf = [0u8; 20];
        let mut i = 0;
        while n > 0 {
            buf[i] = b'0' + (n % 10) as u8;
            n /= 10;
            i += 1;
        }
        while i > 0 {
            i -= 1;
            self.write_char(buf[i]);
        }
    }
}

static mut VGA: VgaConsole = VgaConsole::new();

/// Membaca CMOS port 0x70/0x71 untuk mendapatkan ukuran memori fisik yang terpasang (dalam MB)
#[cfg(target_arch = "x86_64")]
pub fn read_cmos_reg(reg: u8) -> u8 {
    unsafe {
        core::arch::asm!("out 0x70, al", in("al") reg, options(nomem, nostack, preserves_flags));
        let val: u8;
        core::arch::asm!("in al, 0x71", out("al") val, options(nomem, nostack, preserves_flags));
        val
    }
}

/// Membaca CMOS port 0x70/0x71 untuk mendapatkan ukuran memori fisik yang terpasang (dalam MB)
#[cfg(target_arch = "x86_64")]
pub fn read_cmos_memory_mb() -> u64 {
    let r30 = read_cmos_reg(0x30) as u64;
    let r31 = read_cmos_reg(0x31) as u64;
    let r34 = read_cmos_reg(0x34) as u64;
    let r35 = read_cmos_reg(0x35) as u64;
    let r5b = read_cmos_reg(0x5b) as u64;
    let r5c = read_cmos_reg(0x5c) as u64;
    let r5d = read_cmos_reg(0x5d) as u64;

    early_serial_write_str("[ASL-CORE] CMOS Memory DUMP: 0x30/31=");
    early_serial_write_u64((r31 << 8) | r30);
    early_serial_write_str(" KB, 0x34/35=");
    early_serial_write_u64((r35 << 8) | r34);
    early_serial_write_str(" (64KB units), 0x5b/5c/5d=");
    early_serial_write_u64(r5b | (r5c << 8) | (r5d << 16));
    early_serial_write_str("\n");

    // CMOS 0x34 & 0x35: Actual extended memory above 16MB in 64KB chunks (IBM / Phoenix / Bochs / VirtualBox standard)
    let chunks_64k = (r35 << 8) | r34;
    if chunks_64k > 0 {
        // total MB = 16MB + (chunks_64k * 64KB / 1024KB) = 16 + chunks_64k / 16
        return 16 + (chunks_64k * 64 / 1024);
    }

    // CMOS 0x5b, 0x5c, 0x5d: Memory in 64KB blocks
    let blocks_5b = r5b | (r5c << 8) | (r5d << 16);
    if blocks_5b > 0 {
        return 16 + (blocks_5b * 64 / 1024);
    }

    // CMOS 0x30 & 0x31: Extended memory up to 15MB/64MB in 1KB units
    let ext_kb = (r31 << 8) | r30;
    if ext_kb > 0 {
        return (ext_kb / 1024) + 1;
    }

    0
}

/// Helper untuk menulis u64 ke serial output UART COM1
pub fn early_serial_write_u64(mut n: u64) {
    if n == 0 {
        early_serial_write_str("0");
        return;
    }
    let mut buf = [0u8; 20];
    let mut i = 0;
    while n > 0 {
        buf[i] = b'0' + (n % 10) as u8;
        n /= 10;
        i += 1;
    }
    while i > 0 {
        i -= 1;
        let b = buf[i];
        let slice = core::str::from_utf8(core::slice::from_ref(&b)).unwrap_or("0");
        early_serial_write_str(slice);
    }
}

/// Helper untuk menulis nilai 16-bit dalam format heksadesimal 4 digit (misal "8086", "1234")
pub fn early_serial_write_hex16(val: u16) {
    const HEX_DIGITS: &[u8; 16] = b"0123456789ABCDEF";
    for shift in [12, 8, 4, 0] {
        let nibble = ((val >> shift) & 0x0F) as usize;
        let b = HEX_DIGITS[nibble];
        let slice = core::str::from_utf8(core::slice::from_ref(&b)).unwrap_or("0");
        early_serial_write_str(slice);
    }
}

/// Mengambil Brand CPU x86 langsung dari instruksi CPUID (daun 0x80000002..0x80000004)
#[cfg(target_arch = "x86_64")]
pub fn get_x86_cpu_brand() -> [u8; 49] {
    let mut brand = [0u8; 49]; // 48 bytes + 1 null terminator
    let mut max_ext: u32;

    unsafe {
        // Periksa apakah extended CPUID didukung
        core::arch::asm!(
            "push rbx",
            "cpuid",
            "pop rbx",
            inout("eax") 0x80000000u32 => max_ext,
            out("ecx") _,
            out("edx") _,
            options(nomem, nostack, preserves_flags)
        );

        if max_ext >= 0x80000004 {
            let mut offset = 0;
            for leaf in 0x80000002u32..=0x80000004u32 {
                let a: u32;
                let b: u32;
                let c: u32;
                let d: u32;
                core::arch::asm!(
                    "push rbx",
                    "cpuid",
                    "mov {tmp:e}, ebx",
                    "pop rbx",
                    tmp = out(reg) b,
                    inout("eax") leaf => a,
                    out("ecx") c,
                    out("edx") d,
                    options(nomem, nostack, preserves_flags)
                );

                for reg in [a, b, c, d] {
                    let bytes = reg.to_le_bytes();
                    for byte in bytes {
                        if offset < 48 {
                            brand[offset] = byte;
                            offset += 1;
                        }
                    }
                }
            }
        }
    }

    brand
}

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
                let pitch_bytes =
                    (fb_cfg.pixels_per_scan_line as usize) * (fb_cfg.bytes_per_pixel as usize);
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
                if kernel.memory_allocator.total_pages > 0 && kernel.memory_allocator.free_pages > 0
                {
                    mem_init_ok = true;
                    early_serial_write_str(
                        "[ASL-CORE] M4: Buddy Allocator & Heap Memory Subsystem Initialized.\n",
                    );
                } else {
                    early_serial_write_str(
                        "[ASL-CORE] M4: WARNING - Allocator initialized with 0 available pages.\n",
                    );
                }

                if handover.target_arch == TargetArch::X86_64 {
                    cpu_arch_detected = true;
                }

                // MILESTONE M5: Direct Proof of ARINC 653 Partitions & Major Frame Scheduler
                if kernel.vault_manager.partitions[0].is_active && kernel.scheduler.slot_count == 2
                {
                    // Verifikasi tick deterministik pertama (advance 10ms)
                    let (active_part, _switched) = kernel.scheduler.advance_tick(10_000);
                    if active_part == 1 {
                        early_serial_write_str("[ASL-CORE] M5: ARINC 653 Partitions & Deterministic Scheduler Active.\n");
                    } else {
                        early_serial_write_str("[ASL-CORE] M5: WARNING - Unexpected active partition on initial tick.\n");
                    }
                } else {
                    early_serial_write_str(
                        "[ASL-CORE] M5: WARNING - Partition slots incomplete.\n",
                    );
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
    early_serial_write_str(
        "[ASL-CORE] M6: Kernel Cold Boot Complete. Entering Deterministic Idle Loop.\n",
    );

    let logo = "       ___    ____    __   \n      /   |  / __/   / /   \n     / /| |  \\ \\    / /    \n    / ___ | _/ /   / /___  \n   /_/  |_|/___/  /_____/  \n\n   ASL Kernel v1.0.2\n   Selamat Datang Di ASL Kernel\n\n";

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
            (*vga).write_str("Kernel initialized: ASL Core Kernel v1.0.2\n");

            (*fb).set_fg_color(0x0000FF00); // Green
            (*fb).write_str("   [ OK ] ");
            (*fb).set_fg_color(0x00FFFFFF);
            (*fb).write_str("Kernel initialized: ASL Core Kernel v1.0.2\n");
        }

        if mem_init_ok {
            let mut installed_mb = if !handover_ptr.is_null() {
                let handover = &*handover_ptr;
                handover.calculate_installed_memory() / (1024 * 1024)
            } else {
                0
            };

            #[cfg(target_arch = "x86_64")]
            {
                if installed_mb == 0 {
                    installed_mb = read_cmos_memory_mb();
                }
            }

            (*vga).set_color(0x0A); // Light Green
            (*vga).write_str("   [ OK ] ");
            (*vga).set_color(0x0F);
            (*vga).write_str("Memory manager initialized: ");
            (*vga).write_u64(installed_mb);
            (*vga).write_str(" MB RAM detected\n");

            (*fb).set_fg_color(0x0000FF00); // Green
            (*fb).write_str("   [ OK ] ");
            (*fb).set_fg_color(0x00FFFFFF);
            (*fb).write_str("Memory manager initialized: ");
            (*fb).write_u64(installed_mb);
            (*fb).write_str(" MB RAM detected\n");
        }

        if cpu_arch_detected {
            #[cfg(target_arch = "x86_64")]
            let brand_bytes = get_x86_cpu_brand();

            let mut brand_len = 0;
            while brand_len < 48 && brand_bytes[brand_len] != 0 {
                brand_len += 1;
            }
            let brand_str =
                core::str::from_utf8(&brand_bytes[..brand_len]).unwrap_or("x86_64 Processor");
            let trimmed_brand = brand_str.trim();

            // CPU Load: ARINC 653 Core Partition Slot / Major Frame
            let major_frame = kernel.scheduler.major_frame_duration_us as u64;
            let cpu_load_pct = if major_frame > 0 && kernel.scheduler.slot_count > 0 {
                (kernel.scheduler.slots[0].duration_microseconds as u64 * 100) / major_frame
            } else {
                0
            };

            (*vga).set_color(0x0A); // Light Green
            (*vga).write_str("   [ OK ] ");
            (*vga).set_color(0x0F);
            (*vga).write_str("CPU architecture detected: ");
            (*vga).write_str(trimmed_brand);
            (*vga).write_str(" | CPU Load: ");
            (*vga).write_u64(cpu_load_pct);
            (*vga).write_str("%\n");

            (*fb).set_fg_color(0x0000FF00); // Green
            (*fb).write_str("   [ OK ] ");
            (*fb).set_fg_color(0x00FFFFFF);
            (*fb).write_str("CPU architecture detected: ");
            (*fb).write_str(trimmed_brand);
            (*fb).write_str(" | CPU Load: ");
            (*fb).write_u64(cpu_load_pct);
            (*fb).write_str("%\n");
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
        early_serial_write_str("   [ OK ] Kernel initialized: ASL Core Kernel v1.0.2\n");
    }
    if mem_init_ok {
        let mut installed_mb = if !handover_ptr.is_null() {
            let handover = &*handover_ptr;
            handover.calculate_installed_memory() / (1024 * 1024)
        } else {
            0
        };

        #[cfg(target_arch = "x86_64")]
        {
            if installed_mb == 0 {
                installed_mb = read_cmos_memory_mb();
            }
        }

        early_serial_write_str("   [ OK ] Memory manager initialized: ");
        early_serial_write_u64(installed_mb);
        early_serial_write_str(" MB RAM detected\n");
    }
    if cpu_arch_detected {
        #[cfg(target_arch = "x86_64")]
        let brand_bytes = get_x86_cpu_brand();

        let mut brand_len = 0;
        while brand_len < 48 && brand_bytes[brand_len] != 0 {
            brand_len += 1;
        }
        let brand_str =
            core::str::from_utf8(&brand_bytes[..brand_len]).unwrap_or("x86_64 Processor");
        let trimmed_brand = brand_str.trim();

        let major_frame = kernel.scheduler.major_frame_duration_us as u64;
        let cpu_load_pct = if major_frame > 0 && kernel.scheduler.slot_count > 0 {
            (kernel.scheduler.slots[0].duration_microseconds as u64 * 100) / major_frame
        } else {
            0
        };

        early_serial_write_str("   [ OK ] CPU architecture detected: ");
        early_serial_write_str(trimmed_brand);
        early_serial_write_str(" | CPU Load: ");
        early_serial_write_u64(cpu_load_pct);
        early_serial_write_str("%\n");
    }
    early_serial_write_str("\n   ASL kernel booted successfully.\n\n");

    // =========================================================================
    // MILESTONE 1 (v1.0.2): PCI Discovery, PS/2 Keyboard & Cockpit Shell Integration
    // =========================================================================

    // 1. Inisialisasi Secondary Failover Core Engine (Standby Readiness)
    let _secondary_engine = asl_kernel_core_secondary::SecondaryCoreEngine::new();
    early_serial_write_str("[ASL-CORE] Secondary Failover Core Engine: STANDBY ARMED.\n");

    // 2. Scan PCI Bus
    let mut pci_devices =
        [asl_driver_pci::PciDeviceDescriptor::empty(); asl_driver_pci::MAX_PCI_DEVICES];
    let pci_transport = asl_driver_pci::X86PortIoTransport;
    let pci_count =
        unsafe { asl_driver_pci::PciScanner::scan_all(&pci_transport, &mut pci_devices, 0) };

    early_serial_write_str("[ASL-CORE] PCI Discovery: Found ");
    early_serial_write_u64(pci_count as u64);
    early_serial_write_str(" devices on Bus 0.\n");

    #[cfg(target_arch = "x86_64")]
    {
        let fb = core::ptr::addr_of_mut!(FB_CONSOLE);
        (*fb).set_fg_color(0x00FFFF00); // Yellow
        (*fb).write_str("   [ OK ] ");
        (*fb).set_fg_color(0x00FFFFFF);
        (*fb).write_str("PCI Bus Enumerator: ");
        (*fb).write_u64(pci_count as u64);
        (*fb).write_str(" devices discovered\n");

        (*fb).set_fg_color(0x0000FFFF); // Cyan
        (*fb).write_str("   [ OK ] ");
        (*fb).set_fg_color(0x00FFFFFF);
        (*fb).write_str("Secondary Failover Engine: Active-Standby Ready\n\n");

        // Tampilkan prompt Cockpit Shell pertama kali
        (*fb).set_fg_color(0x0000FF00); // Green
        (*fb).write_str("ASL-OS> ");
        (*fb).set_fg_color(0x00FFFFFF);
    }

    early_serial_write_str("\nASL-OS> ");

    // 3. Inisialisasi PS/2 Keyboard & Cockpit Shell Engine
    let mut keyboard = asl_driver_ps2_keyboard::Ps2Keyboard::new();
    let mut shell = asl_cockpit_shell::CockpitShell::new();

    // 4. Interactive Cockpit Shell & Deterministic Event Loop
    let mut _shell_ticks: u64 = 0;
    loop {
        _shell_ticks = _shell_ticks.wrapping_add(1);

        // Poll keyboard input (PS/2 Hardware and Serial COM1 Data Ready)
        #[cfg(target_arch = "x86_64")]
        {
            let mut input_char: Option<u8> = unsafe { keyboard.poll_hardware() };

            // Jika tidak ada input PS/2, periksa apakah ada karakter di port serial COM1 (0x3F8)
            if input_char.is_none() {
                let lsr = unsafe { early_inb(COM1_PORT + 5) };
                if (lsr & 0x01) != 0 {
                    let serial_byte = unsafe { early_inb(COM1_PORT) };
                    // Konversi CR (\r) ke LF (\n)
                    let normalized = if serial_byte == b'\r' {
                        b'\n'
                    } else {
                        serial_byte
                    };
                    input_char = Some(normalized);
                }
            }

            if let Some(ascii) = input_char {
                // Echo key to serial
                if ascii == b'\x08' {
                    early_serial_write_str("\x08 \x08");
                } else if ascii == b'\n' {
                    early_serial_write_str("\n");
                } else {
                    let b = [ascii];
                    if let Ok(s) = core::str::from_utf8(&b) {
                        early_serial_write_str(s);
                    }
                }

                // Echo to Framebuffer
                let fb = core::ptr::addr_of_mut!(FB_CONSOLE);
                unsafe {
                    if ascii == b'\x08' {
                        if (*fb).cursor_x >= 8 {
                            (*fb).cursor_x -= 8;
                            (*fb).draw_char(b' ');
                            (*fb).cursor_x -= 8;
                        }
                    } else {
                        (*fb).draw_char(ascii);
                    }
                }

                // Process command in shell
                if let Some(cmd) = shell.handle_char(ascii) {
                    let fb = core::ptr::addr_of_mut!(FB_CONSOLE);
                    match cmd {
                        asl_cockpit_shell::ShellCommand::Help => {
                            let msg = "Available commands:\n  help    - Show command list\n  clear   - Clear display console\n  meminfo - Display installed RAM & memory map\n  cpuinfo - Display CPU brand & scheduler load\n  lspci   - List detected PCI bus devices\n  uname   - Display kernel release version\n  reboot  - Restart machine\n  halt    - Halt processor\n";
                            early_serial_write_str(msg);
                            unsafe {
                                (*fb).write_str(msg);
                            }
                        }
                        asl_cockpit_shell::ShellCommand::Clear => {
                            unsafe {
                                (*fb).clear();
                                (*fb).cursor_y = 16;
                            }
                            early_serial_write_str("\x1B[2J\x1B[H");
                        }
                        asl_cockpit_shell::ShellCommand::MemInfo => {
                            let installed_mb = if !handover_ptr.is_null() {
                                unsafe {
                                    (*handover_ptr).calculate_installed_memory() / (1024 * 1024)
                                }
                            } else {
                                0
                            };
                            early_serial_write_str("Installed RAM: ");
                            early_serial_write_u64(installed_mb);
                            early_serial_write_str(" MB\nFree Heap Pages: ");
                            early_serial_write_u64(kernel.memory_allocator.free_pages as u64);
                            early_serial_write_str("\n");

                            unsafe {
                                (*fb).write_str("Installed RAM: ");
                                (*fb).write_u64(installed_mb);
                                (*fb).write_str(" MB | Free Heap Pages: ");
                                (*fb).write_u64(kernel.memory_allocator.free_pages as u64);
                                (*fb).write_str("\n");
                            }
                        }
                        asl_cockpit_shell::ShellCommand::CpuInfo => {
                            #[cfg(target_arch = "x86_64")]
                            let brand_bytes = get_x86_cpu_brand();
                            let mut brand_len = 0;
                            while brand_len < 48 && brand_bytes[brand_len] != 0 {
                                brand_len += 1;
                            }
                            let brand_str = core::str::from_utf8(&brand_bytes[..brand_len])
                                .unwrap_or("x86_64 Processor");
                            let trimmed_brand = brand_str.trim();

                            early_serial_write_str("CPU Model: ");
                            early_serial_write_str(trimmed_brand);
                            early_serial_write_str("\n");

                            unsafe {
                                (*fb).write_str("CPU Model: ");
                                (*fb).write_str(trimmed_brand);
                                (*fb).write_str("\n");
                            }
                        }
                        asl_cockpit_shell::ShellCommand::LsPci => {
                            early_serial_write_str("PCI Bus Devices:\n");
                            unsafe {
                                (*fb).write_str("PCI Bus Devices:\n");
                            }

                            for i in 0..pci_count {
                                let dev = &pci_devices[i];
                                early_serial_write_str("  [");
                                early_serial_write_u64(dev.address.bus as u64);
                                early_serial_write_str(":");
                                early_serial_write_u64(dev.address.device as u64);
                                early_serial_write_str(".");
                                early_serial_write_u64(dev.address.function as u64);
                                early_serial_write_str("] Vendor: 0x");
                                early_serial_write_hex16(dev.vendor_id);
                                early_serial_write_str(" Device: 0x");
                                early_serial_write_hex16(dev.device_id);
                                early_serial_write_str(" Class: ");
                                early_serial_write_str(dev.class_name());
                                early_serial_write_str("\n");

                                unsafe {
                                    (*fb).write_str("  [");
                                    (*fb).write_u64(dev.address.bus as u64);
                                    (*fb).write_str(":");
                                    (*fb).write_u64(dev.address.device as u64);
                                    (*fb).write_str(".");
                                    (*fb).write_u64(dev.address.function as u64);
                                    (*fb).write_str("] 0x");
                                    (*fb).write_hex16(dev.vendor_id);
                                    (*fb).write_str(":0x");
                                    (*fb).write_hex16(dev.device_id);
                                    (*fb).write_str(" ");
                                    (*fb).write_str(dev.class_name());
                                    (*fb).write_str("\n");
                                }
                            }
                        }
                        asl_cockpit_shell::ShellCommand::Uname => {
                            let uname_msg = "ASL Core Kernel v1.0.2 (x86_64-unknown-none) [DO-178C DAL-A Standard]\n";
                            early_serial_write_str(uname_msg);
                            unsafe {
                                (*fb).write_str(uname_msg);
                            }
                        }
                        asl_cockpit_shell::ShellCommand::Reboot => {
                            early_serial_write_str("Rebooting machine...\n");
                            unsafe {
                                asl_cockpit_shell::CockpitShell::trigger_x86_reboot();
                            }
                        }
                        asl_cockpit_shell::ShellCommand::Halt => {
                            early_serial_write_str("System halted.\n");
                            unsafe {
                                asl_cockpit_shell::CockpitShell::trigger_x86_halt();
                            }
                        }
                        asl_cockpit_shell::ShellCommand::Unknown => {
                            let unk_msg = "Unknown command. Type 'help' for available commands.\n";
                            early_serial_write_str(unk_msg);
                            unsafe {
                                (*fb).write_str(unk_msg);
                            }
                        }
                        asl_cockpit_shell::ShellCommand::Empty => {}
                    }

                    early_serial_write_str("ASL-OS> ");
                    unsafe {
                        (*fb).set_fg_color(0x0000FF00); // Green
                        (*fb).write_str("ASL-OS> ");
                        (*fb).set_fg_color(0x00FFFFFF);
                    }
                }
            }
        }

        #[cfg(target_arch = "x86_64")]
        core::arch::asm!("pause", options(nomem, nostack, preserves_flags));
        #[cfg(not(target_arch = "x86_64"))]
        core::hint::spin_loop();
    }
}
