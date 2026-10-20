//! x86_64 Interrupt Descriptor Table (IDT), Task State Segment (TSS) & Minimal Fault Infrastructure
//! =================================================================================================
//! DO-178C DAL-A alignment work in progress / inspired deterministic design tanpa alokasi heap.
//! Menyediakan:
//! - IDT 256-entri statis
//! - TSS statis dengan IST1 (Interrupt Stack Table 1) untuk Double Fault (#DF) protection
//! - Dedicated Fault Handler untuk #DE (0), #DF (8), #GP (13), #PF (14)
//! - Exception Frame Capture & Serial Diagnostic Register Dump

/// 64-bit Interrupt Gate Descriptor (16 bytes)
#[repr(C, packed)]
#[derive(Copy, Clone)]
pub struct IdtEntry {
    pub offset_low: u16,
    pub selector: u16,
    pub ist: u8,
    pub type_attr: u8,
    pub offset_mid: u16,
    pub offset_high: u32,
    pub reserved: u32,
}

impl IdtEntry {
    pub const fn missing() -> Self {
        Self {
            offset_low: 0,
            selector: 0,
            ist: 0,
            type_attr: 0,
            offset_mid: 0,
            offset_high: 0,
            reserved: 0,
        }
    }

    /// Membuat 64-bit Interrupt Gate Descriptor
    /// type_attr: 0x8E = Present (1) | DPL 0 (00) | Interrupt Gate 64-bit (0xE)
    pub fn new(handler: usize, cs_selector: u16, ist: u8) -> Self {
        Self {
            offset_low: (handler & 0xFFFF) as u16,
            selector: cs_selector,
            ist: ist & 0x07,
            type_attr: 0x8E, // Present | Ring 0 | 64-bit Interrupt Gate
            offset_mid: ((handler >> 16) & 0xFFFF) as u16,
            offset_high: ((handler >> 32) & 0xFFFF_FFFF) as u32,
            reserved: 0,
        }
    }
}

/// Pointer struktur untuk instruksi `lidt`
#[repr(C, packed)]
pub struct IdtPointer {
    pub limit: u16,
    pub base: u64,
}

/// Task State Segment (TSS) x86_64 (104 bytes)
#[repr(C, packed)]
pub struct TaskStateSegment {
    pub reserved0: u32,
    pub rsp0: u64,
    pub rsp1: u64,
    pub rsp2: u64,
    pub reserved1: u64,
    pub ist1: u64,
    pub ist2: u64,
    pub ist3: u64,
    pub ist4: u64,
    pub ist5: u64,
    pub ist6: u64,
    pub ist7: u64,
    pub reserved2: u64,
    pub reserved3: u16,
    pub iomap_base: u16,
}

impl TaskStateSegment {
    pub const fn empty() -> Self {
        Self {
            reserved0: 0,
            rsp0: 0,
            rsp1: 0,
            rsp2: 0,
            reserved1: 0,
            ist1: 0,
            ist2: 0,
            ist3: 0,
            ist4: 0,
            ist5: 0,
            ist6: 0,
            ist7: 0,
            reserved2: 0,
            reserved3: 0,
            iomap_base: 104, // Tidak ada I/O permission bitmap
        }
    }
}

/// Ukuran stack dedicated IST1 untuk Double Fault (#DF) handler (16 KB)
pub const DOUBLE_FAULT_IST_STACK_SIZE: usize = 16384;
pub static mut DOUBLE_FAULT_IST_STACK: [u8; DOUBLE_FAULT_IST_STACK_SIZE] =
    [0u8; DOUBLE_FAULT_IST_STACK_SIZE];

/// Static TSS Instance
pub static mut STATIC_TSS: TaskStateSegment = TaskStateSegment::empty();

/// Static GDT Table (termasuk TSS Descriptor 16-byte)
/// Entri:
/// Static GDT Table
/// Entri sesuai konvensi segment selector:
/// 0 (0x00): Null Descriptor
/// 1 (0x08): 16-bit Code
/// 2 (0x10): 16-bit Data
/// 3 (0x18): 32-bit Code
/// 4 (0x20): 32-bit Data
/// 5 (0x28): 64-bit Kernel Code (DPL 0, L=1, P=1, readable)
/// 6 (0x30): 64-bit Kernel Data (DPL 0, W=1, P=1)
/// 7 (0x38): 64-bit TSS Low (16-byte System Descriptor)
/// 8 (0x40): 64-bit TSS High
#[repr(C, align(16))]
pub struct GdtTable {
    pub entries: [u64; 9],
}

pub static mut STATIC_GDT: GdtTable = GdtTable {
    entries: [
        0,                                   // 0x00: Null
        0x0000_9A00_0000_FFFF,               // 0x08: 16-bit Code
        0x0000_9200_0000_FFFF,               // 0x10: 16-bit Data
        0x00CF_9A00_0000_FFFF,               // 0x18: 32-bit Code
        0x00CF_9200_0000_FFFF,               // 0x20: 32-bit Data
        0x0020_9A00_0000_0000,               // 0x28: 64-bit Kernel Code
        0x0000_9200_0000_0000,               // 0x30: 64-bit Kernel Data
        0,                                   // 0x38: TSS Low (diisi saat inisialisasi)
        0,                                   // 0x40: TSS High (diisi saat inisialisasi)
    ],
};

#[repr(C, packed)]
pub struct GdtPointer {
    pub limit: u16,
    pub base: u64,
}

/// Static IDT Table 256 entries
pub static mut STATIC_IDT: [IdtEntry; 256] = [IdtEntry::missing(); 256];

/// Snapshot Register & Exception Frame yang ditangkap saat fault terjadi
#[repr(C)]
#[derive(Debug, Copy, Clone)]
pub struct ExceptionContext {
    // CR2 (Page Fault Address) - lowest address on stack (offset 0)
    pub cr2: u64,
    // General Purpose Registers (GPR) - pushed in reverse order: r15 through rax
    // [RSP+8..RSP+128]
    pub r15: u64,
    pub r14: u64,
    pub r13: u64,
    pub r12: u64,
    pub r11: u64,
    pub r10: u64,
    pub r9: u64,
    pub r8: u64,
    pub rbp: u64,
    pub rdi: u64,
    pub rsi: u64,
    pub rdx: u64,
    pub rcx: u64,
    pub rbx: u64,
    pub rax: u64,
    // Vector & error code: vector was pushed last (after error code), so vector is at lower address
    // [RSP+128]: vector
    pub vector: u64,
    // [RSP+136]: error_code
    pub error_code: u64,
    // Hardware interrupt stack frame - pushed by CPU:
    // [RSP+144]: rip
    pub rip: u64,
    // [RSP+152]: cs
    pub cs: u64,
    // [RSP+160]: rflags
    pub rflags: u64,
    // [RSP+168]: rsp
    pub rsp: u64,
    // [RSP+176]: ss
    pub ss: u64,
}

/// Status pemicu trap terakhir untuk verifikasi pengujian
#[derive(Copy, Clone, Debug, PartialEq, Eq)]
pub struct LastTrapRecord {
    pub vector: u64,
    pub error_code: u64,
    pub cr2: u64,
    pub rip: u64,
}

pub static mut LAST_TRAP: LastTrapRecord = LastTrapRecord {
    vector: 0xFFFF,
    error_code: 0,
    cr2: 0,
    rip: 0,
};

pub static mut STATIC_GDTR: GdtPointer = GdtPointer { limit: 0, base: 0 };

/// Inisialisasi GDT & TSS dengan IST1 untuk penanganan Double Fault
#[cfg(target_arch = "x86_64")]
pub unsafe fn init_gdt_and_tss() {
    let tss_ptr = core::ptr::addr_of_mut!(STATIC_TSS);
    let ist1_top = core::ptr::addr_of_mut!(DOUBLE_FAULT_IST_STACK) as u64
        + DOUBLE_FAULT_IST_STACK_SIZE as u64;
    (*tss_ptr).ist1 = ist1_top;

    let tss_addr = tss_ptr as u64;
    let tss_size = core::mem::size_of::<TaskStateSegment>() as u64 - 1;

    let gdt_ptr = core::ptr::addr_of_mut!(STATIC_GDT);
    // TSS Descriptor:
    // Limit 15:0, Base 15:0
    // Base 23:16, Type (0x9 = 64-bit TSS Available), P=1, Limit 19:16
    // Base 31:24
    // Base 63:32
    let limit_low = (tss_size & 0xFFFF) as u64;
    let base_low = (tss_addr & 0xFFFF) as u64;
    let base_mid = ((tss_addr >> 16) & 0xFF) as u64;
    let base_high = ((tss_addr >> 24) & 0xFF) as u64;
    let base_upper = (tss_addr >> 32) as u64;

    let tss_low = limit_low
        | (base_low << 16)
        | (base_mid << 32)
        | (0x89u64 << 40) // Present, TSS 64-bit available (0x89)
        | (((tss_size >> 16) & 0x0F) << 48)
        | (base_high << 56);
    let tss_high = base_upper;

    (*gdt_ptr).entries[7] = tss_low;
    (*gdt_ptr).entries[8] = tss_high;

    STATIC_GDTR = GdtPointer {
        limit: (core::mem::size_of::<GdtTable>() - 1) as u16,
        base: gdt_ptr as u64,
    };

    core::arch::asm!(
        "lgdt [{}]",
        in(reg) core::ptr::addr_of!(STATIC_GDTR),
        options(readonly, nostack)
    );

    // Load Task Register (TR = 0x38, pointing to TSS descriptor in GDT)
    core::arch::asm!(
        "ltr {0:x}",
        in(reg) 0x38u16,
        options(nomem, nostack)
    );
}

/// Mendaftarkan entry gate ke STATIC_IDT (Kernel CS Selector = 0x28)
pub fn register_interrupt_handler(vector: usize, handler: usize, ist: u8) {
    if vector < 256 {
        unsafe {
            STATIC_IDT[vector] = IdtEntry::new(handler, 0x28, ist);
        }
    }
}

pub static mut STATIC_IDTR: IdtPointer = IdtPointer { limit: 0, base: 0 };

/// Memuat tabel IDT ke prosesor dengan instruksi `lidt`
#[cfg(target_arch = "x86_64")]
pub unsafe fn load_idt() {
    STATIC_IDTR = IdtPointer {
        limit: (core::mem::size_of::<[IdtEntry; 256]>() - 1) as u16,
        base: core::ptr::addr_of!(STATIC_IDT) as u64,
    };

    core::arch::asm!(
        "lidt [{}]",
        in(reg) core::ptr::addr_of!(STATIC_IDTR),
        options(readonly, nostack)
    );
}

// Assembly Stubs & ISR Entry Points
#[cfg(target_arch = "x86_64")]
core::arch::global_asm!(
    r#"
    .global isr_wrapper_divide_error
    .global isr_wrapper_double_fault
    .global isr_wrapper_general_protection
    .global isr_wrapper_page_fault
    .global isr_wrapper_breakpoint

    // Vector 0: #DE (No error code pushed by CPU)
    isr_wrapper_divide_error:
        push 0                  // Dummy error code
        push 0                  // Vector 0
        jmp common_isr_stub

    // Vector 3: #BP / INT3 (No error code pushed by CPU)
    isr_wrapper_breakpoint:
        push 0                  // Dummy error code
        push 3                  // Vector 3
        jmp common_isr_stub

    // Vector 8: #DF (CPU hardware pushes error code 0, but software INT 8 does not push error code)
    // To ensure identical stack frame alignment:
    isr_wrapper_double_fault:
        push 0                  // Error code (dummy for INT 8 / normalizer)
        push 8                  // Vector 8
        jmp common_isr_stub

    // Vector 13: #GP (CPU pushes error code)
    isr_wrapper_general_protection:
        // CPU pushes error code
        push 13                 // Vector 13
        jmp common_isr_stub

    // Vector 14: #PF (CPU pushes error code)
    isr_wrapper_page_fault:
        // CPU pushes error code
        push 14                 // Vector 14
        jmp common_isr_stub

    common_isr_stub:
        // Save General Purpose Registers (GPR)
        push rax
        push rbx
        push rcx
        push rdx
        push rsi
        push rdi
        push rbp
        push r8
        push r9
        push r10
        push r11
        push r12
        push r13
        push r14
        push r15

        // Read CR2
        mov rax, cr2
        push rax

        // First argument (RDI in System V AMD64 ABI) = Pointer to ExceptionContext
        mov rdi, rsp

        // Panggil Rust Exception Dispatcher
        cld
        call x86_64_fault_dispatcher

        // Pop CR2
        add rsp, 8

        // Restore GPR
        pop r15
        pop r14
        pop r13
        pop r12
        pop r11
        pop r10
        pop r9
        pop r8
        pop rbp
        pop rdi
        pop rsi
        pop rdx
        pop rcx
        pop rbx
        pop rax

        // Drop vector & error code
        add rsp, 16

        iretq

    // Vector 32 (0x20): APIC Timer Interrupt - Preemptive Scheduling Trampoline
    .global isr_wrapper_apic_timer
    isr_wrapper_apic_timer:
        // Save full 15 GPR registers to construct InterruptContext
        push rax
        push rbx
        push rcx
        push rdx
        push rsi
        push rdi
        push rbp
        push r8
        push r9
        push r10
        push r11
        push r12
        push r13
        push r14
        push r15

        // Pass current stack pointer (pointing to InterruptContext) to Rust dispatcher
        mov rdi, rsp
        cld
        call x86_64_apic_preemptive_handler

        // Switch to the target task's stack pointer returned by scheduler (in RAX)
        mov rsp, rax

        // Restore full 15 GPR registers
        pop r15
        pop r14
        pop r13
        pop r12
        pop r11
        pop r10
        pop r9
        pop r8
        pop rbp
        pop rdi
        pop rsi
        pop rdx
        pop rcx
        pop rbx
        pop rax

        // Hardware frame (RIP, CS, RFLAGS, RSP) is restored by iretq
        iretq
    "#
);

#[cfg(target_arch = "x86_64")]
extern "C" {
    pub fn isr_wrapper_divide_error();
    pub fn isr_wrapper_breakpoint();
    pub fn isr_wrapper_double_fault();
    pub fn isr_wrapper_general_protection();
    pub fn isr_wrapper_page_fault();
    pub fn isr_wrapper_apic_timer();
}

/// Callback tipe untuk notifikasi kernel core & secondary failover
pub type FaultNotifierFn = fn(&ExceptionContext);
pub static mut FAULT_NOTIFIER: Option<FaultNotifierFn> = None;

pub fn set_fault_notifier(notifier: FaultNotifierFn) {
    unsafe {
        FAULT_NOTIFIER = Some(notifier);
    }
}

/// Rust Exception Dispatcher: Dipanggil langsung dari `common_isr_stub`
#[no_mangle]
pub extern "C" fn x86_64_fault_dispatcher(ctx: &ExceptionContext) {
    unsafe {
        LAST_TRAP = LastTrapRecord {
            vector: ctx.vector,
            error_code: ctx.error_code,
            cr2: ctx.cr2,
            rip: ctx.rip,
        };
    }

    // Panggil notifikasi hook jika terpasang (misal secondary core notify & register dump)
    if let Some(notifier) = unsafe { FAULT_NOTIFIER } {
        notifier(ctx);
    }

    // Untuk INT3 (Breakpoint), kita return (resume eksekusi)
    if ctx.vector == 3 {
        return;
    }

    // Untuk fatal faults (#DE, #DF, #GP, #PF), jalankan controlled safe halt
    loop {
        #[cfg(target_arch = "x86_64")]
        unsafe {
            core::arch::asm!("cli; hlt", options(nomem, nostack, preserves_flags));
        }
        #[cfg(not(target_arch = "x86_64"))]
        core::hint::spin_loop();
    }
}

/// Inisialisasi lengkap IDT untuk arsitektur x86_64
#[cfg(target_arch = "x86_64")]
pub unsafe fn init_x86_64_fault_infrastructure() {
    init_gdt_and_tss();

    // Register Vector 0 (#DE)
    register_interrupt_handler(0, isr_wrapper_divide_error as *const () as usize, 0);
    // Register Vector 3 (#BP / INT3)
    register_interrupt_handler(3, isr_wrapper_breakpoint as *const () as usize, 0);
    // Register Vector 8 (#DF) dengan IST 1 (Interrupt Stack Table 1 - Dedicated Double Fault Stack)
    register_interrupt_handler(8, isr_wrapper_double_fault as *const () as usize, 1);
    // Register Vector 13 (#GP)
    register_interrupt_handler(13, isr_wrapper_general_protection as *const () as usize, 0);
    // Register Vector 14 (#PF) dengan IST 1: pada stack overflow murni, frame
    // exception tidak bisa ditulis ke stack yang sedang rusak (akan berubah
    // menjadi #DF dan menghapus vektor/CR2 penyebab). IST1 memaksa switch ke
    // stack dedicated sehingga #PF asli terlaporkan dengan benar.
    register_interrupt_handler(14, isr_wrapper_page_fault as *const () as usize, 1);
    // Register Vector 32 (APIC Timer IRQ)
    register_interrupt_handler(32, isr_wrapper_apic_timer as *const () as usize, 0);

    load_idt();
}
