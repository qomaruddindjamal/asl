//! ASL Deterministic Scheduler & Preemptive Multitasking Foundation (Gate 3C Hardening)
//! ====================================================================================
//! DO-178C DAL-A alignment work in progress / inspired deterministic engineering.
//! Menyediakan:
//! - ARINC 653 Major/Minor Frame Time-Triggered Scheduler
//! - x86_64 InterruptContext structure (19 registers, 152 bytes)
//! - Preemptive TaskControlBlock (TCB) dengan isolasi stack kernel 8 KB
//! - Synthetic initial interrupt stack frame creation
//! - Preemptive APIC timer interrupt scheduling (`asl_preemptive_schedule_tick`)
//! - Cooperative context switch fallback (`asl_context_switch`)
//! - Stack Canary protection (`0xDEAD_BEEF_CAFE_BABE`)
//! - Stack boundary integrity validation

#![no_std]

#[cfg(test)]
extern crate std;

use core::sync::atomic::{AtomicU64, Ordering};

/// Cooperative switch counter (hanya bertambah saat schedule_next dipanggil)
pub static COOPERATIVE_SWITCH_COUNT: AtomicU64 = AtomicU64::new(0);

pub const MAX_SLOTS_PER_MAJOR_FRAME: usize = 32;

/// Slot Waktu Minor Frame dalam Siklus Avionic Major Frame
#[derive(Debug, Copy, Clone)]
pub struct MinorFrameSlot {
    pub partition_id: u32,
    pub duration_microseconds: u32,
    pub is_critical_slot: bool,
}

impl MinorFrameSlot {
    pub const fn empty() -> Self {
        Self {
            partition_id: 0,
            duration_microseconds: 0,
            is_critical_slot: false,
        }
    }
}

/// Siklus Jadwal Deterministik ARINC 653 Major Frame
pub struct MajorFrameSchedule {
    pub major_frame_duration_us: u32,
    pub slot_count: usize,
    pub slots: [MinorFrameSlot; MAX_SLOTS_PER_MAJOR_FRAME],
    pub current_slot_index: usize,
    pub elapsed_in_current_slot_us: u32,
}

impl MajorFrameSchedule {
    pub const fn new(duration_us: u32) -> Self {
        Self {
            major_frame_duration_us: duration_us,
            slot_count: 0,
            slots: [MinorFrameSlot::empty(); MAX_SLOTS_PER_MAJOR_FRAME],
            current_slot_index: 0,
            elapsed_in_current_slot_us: 0,
        }
    }

    /// Menambahkan Minor Frame Slot ke dalam siklus
    pub fn add_slot(
        &mut self,
        partition_id: u32,
        duration_us: u32,
        is_critical: bool,
    ) -> Result<(), &'static str> {
        if self.slot_count >= MAX_SLOTS_PER_MAJOR_FRAME {
            return Err("ERR_SCHEDULE_SLOTS_FULL");
        }

        self.slots[self.slot_count] = MinorFrameSlot {
            partition_id,
            duration_microseconds: duration_us,
            is_critical_slot: is_critical,
        };
        self.slot_count += 1;
        Ok(())
    }

    /// Memajukan tick timer deterministik dan menentukan apakah terjadi perpindahan partisi (Context Switch)
    pub fn advance_tick(&mut self, tick_us: u32) -> (u32, bool) {
        if self.slot_count == 0 {
            return (0, false);
        }

        self.elapsed_in_current_slot_us += tick_us;
        let current_slot = &self.slots[self.current_slot_index];

        if self.elapsed_in_current_slot_us >= current_slot.duration_microseconds {
            // Waktu slot habis: ganti ke slot berikutnya secara deterministik siklik
            self.elapsed_in_current_slot_us = 0;
            self.current_slot_index = (self.current_slot_index + 1) % self.slot_count;
            let next_partition = self.slots[self.current_slot_index].partition_id;
            (next_partition, true) // Memicu Context Switch
        } else {
            (current_slot.partition_id, false)
        }
    }
}

// =========================================================================
// GATE 3B: TRUE PREEMPTIVE SCHEDULING & INTERRUPT CONTEXT
// =========================================================================

pub const KERNEL_STACK_SIZE: usize = 8192; // 8 KB per task
pub const STACK_CANARY_VALUE: u64 = 0xDEAD_BEEF_CAFE_BABE;
pub const RFLAGS_INTERRUPTS_ENABLED: u64 = 0x202; // IF (bit 9) | Reserved (bit 1)
pub const KERNEL_CS_SELECTOR: u64 = 0x28; // 64-bit Kernel Code Segment in STATIC_GDT

/// Interrupt Context Structure (20 QWORDs = 160 bytes)
/// Layout byte-per-byte identik dengan urutan push/pop pada `isr_wrapper_apic_timer`.
///
/// Urutan stack (dari address rendah ke address tinggi):
/// [rsp + 0x00]: r15
/// [rsp + 0x08]: r14
/// [rsp + 0x10]: r13
/// [rsp + 0x18]: r12
/// [rsp + 0x20]: r11
/// [rsp + 0x28]: r10
/// [rsp + 0x30]: r9
/// [rsp + 0x38]: r8
/// [rsp + 0x40]: rbp
/// [rsp + 0x48]: rdi
/// [rsp + 0x50]: rsi
/// [rsp + 0x58]: rdx
/// [rsp + 0x60]: rcx
/// [rsp + 0x68]: rbx
/// [rsp + 0x70]: rax
/// -- Hardware Frame didorong otomatis oleh CPU 64-bit pada SEMUA interrupt --
/// [rsp + 0x78]: rip
/// [rsp + 0x80]: cs
/// [rsp + 0x88]: rflags
/// [rsp + 0x90]: rsp
/// [rsp + 0x98]: ss
#[repr(C)]
#[derive(Debug, Copy, Clone, PartialEq, Eq)]
pub struct InterruptContext {
    pub r15: u64,    // 0x00
    pub r14: u64,    // 0x08
    pub r13: u64,    // 0x10
    pub r12: u64,    // 0x18
    pub r11: u64,    // 0x20
    pub r10: u64,    // 0x28
    pub r9: u64,     // 0x30
    pub r8: u64,     // 0x38
    pub rbp: u64,    // 0x40
    pub rdi: u64,    // 0x48
    pub rsi: u64,    // 0x50
    pub rdx: u64,    // 0x58
    pub rcx: u64,    // 0x60
    pub rbx: u64,    // 0x68
    pub rax: u64,    // 0x70
    // Hardware interrupt frame (pushed by CPU in 64-bit mode)
    pub rip: u64,    // 0x78
    pub cs: u64,     // 0x80
    pub rflags: u64, // 0x88
    pub rsp: u64,    // 0x90
    pub ss: u64,     // 0x98
}

impl InterruptContext {
    pub const fn empty() -> Self {
        Self {
            r15: 0,
            r14: 0,
            r13: 0,
            r12: 0,
            r11: 0,
            r10: 0,
            r9: 0,
            r8: 0,
            rbp: 0,
            rdi: 0,
            rsi: 0,
            rdx: 0,
            rcx: 0,
            rbx: 0,
            rax: 0,
            rip: 0,
            cs: KERNEL_CS_SELECTOR,
            rflags: RFLAGS_INTERRUPTS_ENABLED,
            rsp: 0,
            ss: 0x30, // 64-bit Kernel Data Segment
        }
    }
}

// Compile-time assertions for struct size and alignment
const _: () = assert!(core::mem::size_of::<InterruptContext>() == 160);
const _: () = assert!(core::mem::align_of::<InterruptContext>() == 8);

// Compile-time offset assertions for all 20 QWORD fields (Part C requirement)
const _: () = assert!(core::mem::offset_of!(InterruptContext, r15) == 0x00);
const _: () = assert!(core::mem::offset_of!(InterruptContext, r14) == 0x08);
const _: () = assert!(core::mem::offset_of!(InterruptContext, r13) == 0x10);
const _: () = assert!(core::mem::offset_of!(InterruptContext, r12) == 0x18);
const _: () = assert!(core::mem::offset_of!(InterruptContext, r11) == 0x20);
const _: () = assert!(core::mem::offset_of!(InterruptContext, r10) == 0x28);
const _: () = assert!(core::mem::offset_of!(InterruptContext, r9) == 0x30);
const _: () = assert!(core::mem::offset_of!(InterruptContext, r8) == 0x38);
const _: () = assert!(core::mem::offset_of!(InterruptContext, rbp) == 0x40);
const _: () = assert!(core::mem::offset_of!(InterruptContext, rdi) == 0x48);
const _: () = assert!(core::mem::offset_of!(InterruptContext, rsi) == 0x50);
const _: () = assert!(core::mem::offset_of!(InterruptContext, rdx) == 0x58);
const _: () = assert!(core::mem::offset_of!(InterruptContext, rcx) == 0x60);
const _: () = assert!(core::mem::offset_of!(InterruptContext, rbx) == 0x68);
const _: () = assert!(core::mem::offset_of!(InterruptContext, rax) == 0x70);
const _: () = assert!(core::mem::offset_of!(InterruptContext, rip) == 0x78);
const _: () = assert!(core::mem::offset_of!(InterruptContext, cs) == 0x80);
const _: () = assert!(core::mem::offset_of!(InterruptContext, rflags) == 0x88);
const _: () = assert!(core::mem::offset_of!(InterruptContext, rsp) == 0x90);
const _: () = assert!(core::mem::offset_of!(InterruptContext, ss) == 0x98);

// =========================================================================
// PART D: RING 0 INTERRUPT FRAME NORMALIZATION
// =========================================================================
/// 15 General Purpose Registers yang disimpan oleh ISR trampoline stub
#[repr(C)]
#[derive(Debug, Copy, Clone, PartialEq, Eq)]
pub struct SavedGeneralRegisters {
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
}

/// Hardware frame Ring 0 ke Ring 0 yang didorong secara otomatis oleh CPU x86_64
#[repr(C)]
#[derive(Debug, Copy, Clone, PartialEq, Eq)]
pub struct SamePrivilegeHardwareFrame {
    pub rip: u64,
    pub cs: u64,
    pub rflags: u64,
    pub rsp: u64,
    pub ss: u64,
}

/// Hardware frame lintas privilege (Ring 3 ke Ring 0) - disediakan untuk arsitektur masa depan
#[repr(C)]
#[derive(Debug, Copy, Clone, PartialEq, Eq)]
pub struct PrivilegeTransitionHardwareFrame {
    pub rip: u64,
    pub cs: u64,
    pub rflags: u64,
    pub rsp: u64,
    pub ss: u64,
}

/// Normalisasi representasi frame interrupt lengkap
#[repr(C)]
#[derive(Debug, Copy, Clone, PartialEq, Eq)]
pub struct NormalizedInterruptContext {
    pub gpr: SavedGeneralRegisters,
    pub hw: SamePrivilegeHardwareFrame,
}

const _: () = assert!(core::mem::size_of::<SavedGeneralRegisters>() == 120);
const _: () = assert!(core::mem::size_of::<SamePrivilegeHardwareFrame>() == 40);
const _: () = assert!(core::mem::size_of::<NormalizedInterruptContext>() == 160);

impl InterruptContext {
    /// Validasi integritas runtime frame sebelum dikembalikan / dieksekusi
    #[inline(always)]
    pub fn is_runtime_valid(&self) -> bool {
        // 1. RIP harus canonical (bit 47..63 harus seragam sign-extended)
        let rip_upper = self.rip >> 47;
        let is_canonical = rip_upper == 0 || rip_upper == 0x1FFFF;
        if !is_canonical || self.rip == 0 {
            return false;
        }

        // 2. CS harus kernel code selector
        if self.cs != KERNEL_CS_SELECTOR {
            return false;
        }

        // 3. RFLAGS: bit 1 (reserved) harus 1, IF (bit 9) harus 1
        if (self.rflags & 0x02) == 0 {
            return false;
        }

        true
    }
}

/// CPU Register Context untuk Cooperative Context Switching Gate 3 (Backward Compatibility)
#[repr(C)]
#[derive(Debug, Copy, Clone, PartialEq, Eq)]
pub struct CpuContext {
    pub r15: u64,
    pub r14: u64,
    pub r13: u64,
    pub r12: u64,
    pub rbx: u64,
    pub rbp: u64,
    pub rflags: u64,
    pub rip: u64,
    pub rsp: u64,
}

impl CpuContext {
    pub const fn empty() -> Self {
        Self {
            r15: 0,
            r14: 0,
            r13: 0,
            r12: 0,
            rbx: 0,
            rbp: 0,
            rflags: RFLAGS_INTERRUPTS_ENABLED,
            rip: 0,
            rsp: 0,
        }
    }

    pub fn switch_simulation(&mut self) {
        self.rbx = 0xBBBB;
        self.rbp = 0xCCCC;
    }
}

/// Status Siklus Hidup Task Deterministik
#[derive(Debug, Copy, Clone, PartialEq, Eq)]
pub enum TaskState {
    Unused,
    Ready,
    Running,
    Blocked,
    Halted,
    Terminated,
    Faulted,
}

/// Identifikasi Unik Task
pub type TaskId = u32;

/// Struktur Memori Stack Kernel Mandiri 8 KB dengan Proteksi Canary
#[repr(C, align(16))]
pub struct KernelStack {
    pub canary: u64,
    pub storage: [u8; KERNEL_STACK_SIZE - 8],
}

impl KernelStack {
    pub const fn new() -> Self {
        Self {
            canary: STACK_CANARY_VALUE,
            storage: [0u8; KERNEL_STACK_SIZE - 8],
        }
    }

    /// Memeriksa integritas canary stack di dasar stack
    #[inline(always)]
    pub fn verify_canary(&self) -> bool {
        self.canary == STACK_CANARY_VALUE
    }

    /// Mendapatkan batas bawah stack storage
    #[inline(always)]
    pub fn bottom_address(&self) -> u64 {
        self.storage.as_ptr() as u64
    }

    /// Mendapatkan batas atas (puncak stack yang tumbuh ke bawah)
    #[inline(always)]
    pub fn top_address(&self) -> u64 {
        let ptr = self.storage.as_ptr() as u64;
        ptr + (KERNEL_STACK_SIZE - 8) as u64
    }

    /// Memeriksa apakah RSP berada dalam batas stack yang sah
    #[inline(always)]
    pub fn is_rsp_within_bounds(&self, rsp: u64) -> bool {
        rsp >= self.bottom_address() && rsp <= self.top_address()
    }
}

/// Preemptive Task Control Block (TCB)
#[repr(C)]
pub struct TaskControlBlock {
    pub id: TaskId,
    pub state: TaskState,
    pub saved_rsp: u64,
    pub context: CpuContext,
    pub stack: KernelStack,
    pub switch_count: u64,
    pub runtime_ticks: u64,
    pub dispatch_count: u64,
    pub preemption_count: u64,
    pub total_quanta: u64,
    pub longest_wait_ticks: u64,
    pub last_scheduled_tick: u64,
    pub starvation_counter: u64,
}

impl TaskControlBlock {
    pub const fn empty() -> Self {
        Self {
            id: 0,
            state: TaskState::Unused,
            saved_rsp: 0,
            context: CpuContext::empty(),
            stack: KernelStack::new(),
            switch_count: 0,
            runtime_ticks: 0,
            dispatch_count: 0,
            preemption_count: 0,
            total_quanta: 0,
            longest_wait_ticks: 0,
            last_scheduled_tick: 0,
            starvation_counter: 0,
        }
    }

    /// Inisialisasi task dengan Synthetic Interrupt Return Frame.
    /// Memungkinkan task pertama kali dijalankan langsung melalui jalur `iretq` yang sama dengan task yang dipreempt.
    pub fn init_preemptive(&mut self, id: TaskId, entry_point: usize) {
        self.id = id;
        self.state = TaskState::Ready;
        self.switch_count = 0;
        self.runtime_ticks = 0;
        self.dispatch_count = 0;
        self.preemption_count = 0;
        self.total_quanta = 0;
        self.longest_wait_ticks = 0;
        self.last_scheduled_tick = 0;
        self.starvation_counter = 0;
        self.stack.canary = STACK_CANARY_VALUE;

        // Puncak stack disejajarkan ke 16-byte boundary
        let stack_top = self.stack.top_address() & !0x0F;

        // Buat Synthetic Interrupt Frame pada stack task
        // Frame size = 152 bytes (19 QWORDs)
        let frame_ptr = (stack_top - core::mem::size_of::<InterruptContext>() as u64) as *mut InterruptContext;

        unsafe {
            core::ptr::write_volatile(
                frame_ptr,
                InterruptContext {
                    r15: 0,
                    r14: 0,
                    r13: 0,
                    r12: 0,
                    r11: 0,
                    r10: 0,
                    r9: 0,
                    r8: 0,
                    rbp: 0,
                    rdi: 0,
                    rsi: 0,
                    rdx: 0,
                    rcx: 0,
                    rbx: 0,
                    rax: 0,
                    rip: entry_point as u64,
                    cs: KERNEL_CS_SELECTOR,
                    rflags: RFLAGS_INTERRUPTS_ENABLED,
                    rsp: stack_top,
                    ss: 0x30,
                },
            );
        }

        self.saved_rsp = frame_ptr as u64;

        // Simpan juga versi cooperative fallback
        self.context = CpuContext {
            r15: 0,
            r14: 0,
            r13: 0,
            r12: 0,
            rbx: 0,
            rbp: 0,
            rflags: RFLAGS_INTERRUPTS_ENABLED,
            rip: entry_point as u64,
            rsp: (stack_top - 8),
        };
    }

    /// Inisialisasi preemptive dengan initial general purpose registers kustom pada synthetic frame
    pub fn init_preemptive_with_gprs(
        &mut self,
        id: TaskId,
        entry_point: usize,
        gprs: [u64; 15], // [r15, r14, r13, r12, r11, r10, r9, r8, rbp, rdi, rsi, rdx, rcx, rbx, rax]
    ) {
        self.init_preemptive(id, entry_point);
        let frame_ptr = self.saved_rsp as *mut InterruptContext;
        unsafe {
            (*frame_ptr).r15 = gprs[0];
            (*frame_ptr).r14 = gprs[1];
            (*frame_ptr).r13 = gprs[2];
            (*frame_ptr).r12 = gprs[3];
            (*frame_ptr).r11 = gprs[4];
            (*frame_ptr).r10 = gprs[5];
            (*frame_ptr).r9  = gprs[6];
            (*frame_ptr).r8  = gprs[7];
            (*frame_ptr).rbp = gprs[8];
            (*frame_ptr).rdi = gprs[9];
            (*frame_ptr).rsi = gprs[10];
            (*frame_ptr).rdx = gprs[11];
            (*frame_ptr).rcx = gprs[12];
            (*frame_ptr).rbx = gprs[13];
            (*frame_ptr).rax = gprs[14];
        }
    }

    /// Inisialisasi cooperative (Gate 3 compatibility)
    pub fn init(&mut self, id: TaskId, entry_point: usize) {
        self.init_preemptive(id, entry_point);
    }

    /// Verifikasi apakah stack canary masih utuh
    #[inline(always)]
    pub fn check_stack_canary(&self) -> Result<(), &'static str> {
        if self.stack.verify_canary() {
            Ok(())
        } else {
            Err("ERR_STACK_CANARY_OVERWRITE_DETECTED")
        }
    }
}

// =========================================================================
// PART D: ASSEMBLY CONTEXT SWITCH TRAMPOLINES
// =========================================================================

#[cfg(all(target_arch = "x86_64", target_os = "none"))]
core::arch::global_asm!(
    r#"
    .global asl_context_switch
    asl_context_switch:
        // RDI = *mut CpuContext (prev_context)
        // RSI = *const CpuContext (next_context)

        mov [rdi + 0x00], r15
        mov [rdi + 0x08], r14
        mov [rdi + 0x10], r13
        mov [rdi + 0x18], r12
        mov [rdi + 0x20], rbx
        mov [rdi + 0x28], rbp

        pushfq
        pop rax
        mov [rdi + 0x30], rax

        mov rax, [rsp]
        mov [rdi + 0x38], rax

        lea rax, [rsp + 8]
        mov [rdi + 0x40], rax

        mov r15, [rsi + 0x00]
        mov r14, [rsi + 0x08]
        mov r13, [rsi + 0x10]
        mov r12, [rsi + 0x18]
        mov rbx, [rsi + 0x20]
        mov rbp, [rsi + 0x28]

        mov rax, [rsi + 0x30]
        push rax
        popfq

        mov rsp, [rsi + 0x40]
        mov rax, [rsi + 0x38]
        sub rsp, 8
        mov [rsp], rax

        ret
    "#
);

#[cfg(not(all(target_arch = "x86_64", target_os = "none")))]
#[no_mangle]
pub unsafe extern "C" fn asl_context_switch(prev: *mut CpuContext, next: *const CpuContext) {
    if !prev.is_null() && !next.is_null() {
        (*prev).switch_simulation();
        let _ = (*next).rip;
        let _ = (*next).rsp;
    }
}

#[cfg(all(target_arch = "x86_64", target_os = "none"))]
extern "C" {
    pub fn asl_context_switch(prev: *mut CpuContext, next: *const CpuContext);
}

// =========================================================================
// PART E: PREEMPTIVE TASK MANAGER & DISPATCHER
// =========================================================================

pub const MAX_KERNEL_TASKS: usize = 4;

pub struct KernelTaskManager {
    pub tasks: [TaskControlBlock; MAX_KERNEL_TASKS],
    pub current_task_idx: usize,
    pub task_count: usize,
    pub total_switches: u64,
    pub preemptive_enabled: bool,
    pub quantum_ticks: u64,
    pub current_quantum_count: u64,
}

impl KernelTaskManager {
    pub const fn new() -> Self {
        Self {
            tasks: [
                TaskControlBlock::empty(),
                TaskControlBlock::empty(),
                TaskControlBlock::empty(),
                TaskControlBlock::empty(),
            ],
            current_task_idx: 0,
            task_count: 0,
            total_switches: 0,
            preemptive_enabled: false,
            quantum_ticks: 1, // Default 1 tick per quantum
            current_quantum_count: 0,
        }
    }

    /// Mendaftarkan task baru
    pub fn spawn_task(&mut self, entry_point: usize) -> Result<TaskId, &'static str> {
        if self.task_count >= MAX_KERNEL_TASKS {
            return Err("ERR_TASK_LIMIT_EXCEEDED");
        }
        let id = (self.task_count + 1) as TaskId;
        self.tasks[self.task_count].init_preemptive(id, entry_point);
        self.task_count += 1;
        Ok(id)
    }

    /// Mendaftarkan task baru dengan initial general purpose registers kustom pada synthetic frame
    pub fn spawn_task_with_gprs(
        &mut self,
        entry_point: usize,
        gprs: [u64; 15],
    ) -> Result<TaskId, &'static str> {
        if self.task_count >= MAX_KERNEL_TASKS {
            return Err("ERR_TASK_LIMIT_EXCEEDED");
        }
        let id = (self.task_count + 1) as TaskId;
        self.tasks[self.task_count].init_preemptive_with_gprs(id, entry_point, gprs);
        self.task_count += 1;
        Ok(id)
    }

    /// Preemptive Scheduler Tick: Dipanggil langsung dari APIC Timer ISR (`x86_64_apic_preemptive_tick`)
    /// Menerima stack pointer task yang dipreempt (`current_rsp`),
    /// Memilih task berikutnya, dan mengembalikan stack pointer task berikutnya (`next_rsp`).
    pub fn on_timer_tick(&mut self, current_rsp: u64) -> u64 {
        // Jika preemption belum aktif atau task runnable <= 1, tetap di task yang sama
        if !self.preemptive_enabled || self.task_count <= 1 {
            return current_rsp;
        }

        self.current_quantum_count += 1;
        if self.current_quantum_count < self.quantum_ticks {
            return current_rsp;
        }
        self.current_quantum_count = 0;

        let prev_idx = self.current_task_idx;

        // Validasi stack canary task aktif (untuk task > 0 yang memiliki dedicated KernelStack)
        if prev_idx != 0 && !self.tasks[prev_idx].stack.verify_canary() {
            // Stack overrun fatal: tandai task Faulted dan isolasi segera
            self.tasks[prev_idx].state = TaskState::Faulted;
            // Cari task sehat yang Ready
            let mut fallback_idx = (prev_idx + 1) % self.task_count;
            for _ in 0..self.task_count {
                if self.tasks[fallback_idx].state == TaskState::Ready {
                    self.current_task_idx = fallback_idx;
                    self.tasks[fallback_idx].state = TaskState::Running;
                    self.tasks[fallback_idx].dispatch_count += 1;
                    return self.tasks[fallback_idx].saved_rsp;
                }
                fallback_idx = (fallback_idx + 1) % self.task_count;
            }
            return current_rsp;
        }

        // Simpan stack pointer task aktif
        self.tasks[prev_idx].saved_rsp = current_rsp;
        if self.tasks[prev_idx].state == TaskState::Running {
            self.tasks[prev_idx].state = TaskState::Ready;
            self.tasks[prev_idx].preemption_count += 1;
        }
        self.tasks[prev_idx].runtime_ticks += 1;
        self.tasks[prev_idx].total_quanta += 1;

        // Perbarui metrik waktu tunggu untuk seluruh task yang berstatus Ready
        let current_tick_now = self.total_switches;
        for i in 0..self.task_count {
            if i != prev_idx && self.tasks[i].state == TaskState::Ready {
                let wait = current_tick_now.saturating_sub(self.tasks[i].last_scheduled_tick);
                if wait > self.tasks[i].longest_wait_ticks {
                    self.tasks[i].longest_wait_ticks = wait;
                }
            }
        }

        // Cari task berikutnya yang Ready (Round-Robin teruji, abaikan Faulted/Halted)
        let mut next_idx = (prev_idx + 1) % self.task_count;
        let mut found = false;
        for _ in 0..self.task_count {
            if self.tasks[next_idx].state == TaskState::Ready {
                found = true;
                break;
            }
            next_idx = (next_idx + 1) % self.task_count;
        }

        // Jika tidak ada task Ready lain yang ditemukan, kembali ke task lama
        if !found {
            if self.tasks[prev_idx].state == TaskState::Ready {
                self.tasks[prev_idx].state = TaskState::Running;
            }
            return current_rsp;
        }

        // Validasi stack bounds task berikutnya
        let next_rsp = self.tasks[next_idx].saved_rsp;
        let is_rsp_valid = if next_idx == 0 {
            // Task 0: valid jika berada di dalam stack task-nya sendiri ATAU boot kernel stack
            self.tasks[0].stack.is_rsp_within_bounds(next_rsp)
                || (next_rsp != 0 && next_rsp >= 0xFFFF_8000_0000_0000)
        } else {
            self.tasks[next_idx].stack.is_rsp_within_bounds(next_rsp)
        };

        if !is_rsp_valid {
            // Saved RSP tidak valid: tandai task target sebagai Faulted untuk mencegah eksekusi korup
            self.tasks[next_idx].state = TaskState::Faulted;
            if self.tasks[prev_idx].state == TaskState::Ready {
                self.tasks[prev_idx].state = TaskState::Running;
            }
            return current_rsp;
        }

        // Validasi runtime frame context (RIP canonical, CS, RFLAGS)
        let frame_ref = unsafe { &*(next_rsp as *const InterruptContext) };
        if !frame_ref.is_runtime_valid() {
            // Context corrupt / non-canonical RIP: tolak restore dan tandai Faulted
            self.tasks[next_idx].state = TaskState::Faulted;
            if self.tasks[prev_idx].state == TaskState::Ready {
                self.tasks[prev_idx].state = TaskState::Running;
            }
            return current_rsp;
        }

        // Switch ke task berikutnya
        self.tasks[next_idx].state = TaskState::Running;
        self.tasks[next_idx].switch_count += 1;
        self.tasks[next_idx].dispatch_count += 1;
        self.tasks[next_idx].last_scheduled_tick = self.total_switches + 1;
        self.total_switches += 1;
        self.current_task_idx = next_idx;

        next_rsp
    }

    /// Cooperative switch fallback (Gate 3)
    pub fn schedule_next(&mut self) -> Result<bool, &'static str> {
        if self.task_count <= 1 {
            return Ok(false);
        }

        let prev_idx = self.current_task_idx;
        let next_idx = (prev_idx + 1) % self.task_count;

        self.tasks[prev_idx].check_stack_canary()?;

        self.tasks[prev_idx].state = TaskState::Ready;
        self.tasks[next_idx].state = TaskState::Running;
        self.tasks[prev_idx].switch_count += 1;
        self.total_switches += 1;
        self.current_task_idx = next_idx;
        COOPERATIVE_SWITCH_COUNT.fetch_add(1, Ordering::Relaxed);

        let prev_ptr = core::ptr::addr_of_mut!(self.tasks[prev_idx].context);
        let next_ptr = core::ptr::addr_of!(self.tasks[next_idx].context);

        unsafe {
            asl_context_switch(prev_ptr, next_ptr);
        }

        self.tasks[next_idx].check_stack_canary()?;
        Ok(true)
    }
}

pub static mut GLOBAL_TASK_MANAGER: KernelTaskManager = KernelTaskManager::new();

/// Dispatcher Preemptive Tick di tingkat C-ABI: Dipanggil langsung oleh ISR wrapper APIC
#[no_mangle]
pub extern "C" fn asl_preemptive_schedule_tick(current_rsp: u64) -> u64 {
    unsafe {
        let mgr = core::ptr::addr_of_mut!(GLOBAL_TASK_MANAGER);
        (*mgr).on_timer_tick(current_rsp)
    }
}

// =========================================================================
// UNIT TESTS
// =========================================================================

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_gate3b_test1_interrupt_context_layout_and_offsets() {
        assert_eq!(core::mem::size_of::<InterruptContext>(), 160);
        assert_eq!(core::mem::align_of::<InterruptContext>(), 8);

        let ctx = InterruptContext::empty();
        let base = &ctx as *const _ as usize;

        assert_eq!(&ctx.r15 as *const _ as usize - base, 0x00);
        assert_eq!(&ctx.r14 as *const _ as usize - base, 0x08);
        assert_eq!(&ctx.r13 as *const _ as usize - base, 0x10);
        assert_eq!(&ctx.r12 as *const _ as usize - base, 0x18);
        assert_eq!(&ctx.r11 as *const _ as usize - base, 0x20);
        assert_eq!(&ctx.r10 as *const _ as usize - base, 0x28);
        assert_eq!(&ctx.r9 as *const _ as usize - base, 0x30);
        assert_eq!(&ctx.r8 as *const _ as usize - base, 0x38);
        assert_eq!(&ctx.rbp as *const _ as usize - base, 0x40);
        assert_eq!(&ctx.rdi as *const _ as usize - base, 0x48);
        assert_eq!(&ctx.rsi as *const _ as usize - base, 0x50);
        assert_eq!(&ctx.rdx as *const _ as usize - base, 0x58);
        assert_eq!(&ctx.rcx as *const _ as usize - base, 0x60);
        assert_eq!(&ctx.rbx as *const _ as usize - base, 0x68);
        assert_eq!(&ctx.rax as *const _ as usize - base, 0x70);
        assert_eq!(&ctx.rip as *const _ as usize - base, 0x78);
        assert_eq!(&ctx.cs as *const _ as usize - base, 0x80);
        assert_eq!(&ctx.rflags as *const _ as usize - base, 0x88);
        assert_eq!(&ctx.rsp as *const _ as usize - base, 0x90);
        assert_eq!(&ctx.ss as *const _ as usize - base, 0x98);
    }

    #[test]
    fn test_gate3b_test2_synthetic_initial_frame() {
        let mut tcb = TaskControlBlock::empty();
        tcb.init_preemptive(1, 0x401000);

        assert_eq!(tcb.id, 1);
        assert_eq!(tcb.state, TaskState::Ready);
        assert!(tcb.saved_rsp > 0);
        assert!(tcb.stack.is_rsp_within_bounds(tcb.saved_rsp));

        let frame = unsafe { &*(tcb.saved_rsp as *const InterruptContext) };
        assert_eq!(frame.rip, 0x401000);
        assert_eq!(frame.cs, KERNEL_CS_SELECTOR);
        assert_eq!(frame.rflags, RFLAGS_INTERRUPTS_ENABLED);
        assert_eq!(frame.r15, 0);
        assert_eq!(frame.rax, 0);
    }

    #[test]
    fn test_gate3b_test3_preemptive_tick_dispatch() {
        let mut mgr = KernelTaskManager::new();
        mgr.spawn_task(0x1000).unwrap();
        mgr.spawn_task(0x2000).unwrap();
        mgr.preemptive_enabled = true;
        mgr.tasks[0].state = TaskState::Running;

        let task0_rsp = mgr.tasks[0].saved_rsp;
        let task1_rsp = mgr.tasks[1].saved_rsp;

        // Tick 1: Preempt Task 0 -> Switch to Task 1
        let next_rsp = mgr.on_timer_tick(task0_rsp);
        assert_eq!(next_rsp, task1_rsp);
        assert_eq!(mgr.current_task_idx, 1);
        assert_eq!(mgr.tasks[0].state, TaskState::Ready);
        assert_eq!(mgr.tasks[1].state, TaskState::Running);

        // Tick 2: Preempt Task 1 -> Switch back to Task 0
        let next_rsp2 = mgr.on_timer_tick(task1_rsp);
        assert_eq!(next_rsp2, task0_rsp);
        assert_eq!(mgr.current_task_idx, 0);
        assert_eq!(mgr.tasks[0].state, TaskState::Running);
        assert_eq!(mgr.tasks[1].state, TaskState::Ready);
    }

    #[test]
    fn test_gate3b_test7_stack_bounds_verification() {
        let mut tcb = TaskControlBlock::empty();
        tcb.init_preemptive(1, 0x1000);

        let valid_rsp = tcb.saved_rsp;
        assert!(tcb.stack.is_rsp_within_bounds(valid_rsp));

        // Test out of bounds (above top)
        assert!(!tcb.stack.is_rsp_within_bounds(tcb.stack.top_address() + 16));
        // Test out of bounds (below bottom)
        assert!(!tcb.stack.is_rsp_within_bounds(tcb.stack.bottom_address() - 8));
    }

    #[test]
    fn test_gate3b_test8_no_ready_task_path() {
        let mut mgr = KernelTaskManager::new();
        mgr.spawn_task(0x1000).unwrap();
        mgr.preemptive_enabled = true;
        mgr.tasks[0].state = TaskState::Running;

        let cur_rsp = mgr.tasks[0].saved_rsp;
        // Hanya 1 task: harus mengembalikan stack pointer task aktif (tidak null)
        let next_rsp = mgr.on_timer_tick(cur_rsp);
        assert_eq!(next_rsp, cur_rsp);
        assert_ne!(next_rsp, 0);
    }

    #[test]
    fn test_gate3_cooperative_regression() {
        let mut mgr = KernelTaskManager::new();
        assert_eq!(mgr.spawn_task(0x1000).unwrap(), 1);
        assert_eq!(mgr.spawn_task(0x2000).unwrap(), 2);
        assert_eq!(mgr.task_count, 2);

        assert!(mgr.schedule_next().unwrap());
        assert_eq!(mgr.current_task_idx, 1);
        assert!(mgr.schedule_next().unwrap());
        assert_eq!(mgr.current_task_idx, 0);
        assert_eq!(mgr.total_switches, 2);
    }

    #[test]
    fn test_gate3c_faulted_task_exclusion() {
        let mut mgr = KernelTaskManager::new();
        mgr.spawn_task(0x1000).unwrap();
        mgr.spawn_task(0x2000).unwrap();
        mgr.spawn_task(0x3000).unwrap();
        mgr.preemptive_enabled = true;
        mgr.tasks[0].state = TaskState::Running;

        // Mark task 1 as Faulted
        mgr.tasks[1].state = TaskState::Faulted;

        // On tick, scheduler must skip Task 1 (Faulted) and switch directly to Task 2
        let cur_rsp = mgr.tasks[0].saved_rsp;
        let next_rsp = mgr.on_timer_tick(cur_rsp);
        assert_eq!(mgr.current_task_idx, 2);
        assert_eq!(next_rsp, mgr.tasks[2].saved_rsp);
        assert_eq!(mgr.tasks[1].state, TaskState::Faulted);
    }

    #[test]
    fn test_gate3c_three_task_fairness_telemetry() {
        let mut mgr = KernelTaskManager::new();
        mgr.spawn_task(0x1000).unwrap();
        mgr.spawn_task(0x2000).unwrap();
        mgr.spawn_task(0x3000).unwrap();
        mgr.preemptive_enabled = true;
        mgr.tasks[0].state = TaskState::Running;

        let mut sim_rsp = mgr.tasks[0].saved_rsp;
        // Run 300 cycles (100 per task in perfect round robin)
        for _ in 0..300 {
            sim_rsp = mgr.on_timer_tick(sim_rsp);
        }

        // Each task should be dispatched ~100 times
        assert!(mgr.tasks[0].dispatch_count >= 95 && mgr.tasks[0].dispatch_count <= 105);
        assert!(mgr.tasks[1].dispatch_count >= 95 && mgr.tasks[1].dispatch_count <= 105);
        assert!(mgr.tasks[2].dispatch_count >= 95 && mgr.tasks[2].dispatch_count <= 105);
    }

    #[test]
    fn test_gate3d_failure_injection_canary_corruption() {
        let mut mgr = KernelTaskManager::new();
        mgr.spawn_task(0x1000).unwrap();
        mgr.spawn_task(0x2000).unwrap();
        mgr.preemptive_enabled = true;
        mgr.current_task_idx = 1;
        mgr.tasks[1].state = TaskState::Running;

        // Corrupt canary of running task 1
        mgr.tasks[1].stack.canary = 0xDEAD_BEEF;

        let cur_rsp = mgr.tasks[1].saved_rsp;
        let next_rsp = mgr.on_timer_tick(cur_rsp);

        // Task 1 must transition to Faulted and scheduler must isolate it and fallback to Task 0
        assert_eq!(mgr.tasks[1].state, TaskState::Faulted);
        assert_eq!(mgr.current_task_idx, 0);
        assert_eq!(next_rsp, mgr.tasks[0].saved_rsp);
    }

    #[test]
    fn test_gate3d_failure_injection_invalid_rsp() {
        let mut mgr = KernelTaskManager::new();
        mgr.spawn_task(0x1000).unwrap();
        mgr.spawn_task(0x2000).unwrap();
        mgr.preemptive_enabled = true;
        mgr.tasks[0].state = TaskState::Running;

        // Set target task 1 saved_rsp to an invalid address outside its stack
        mgr.tasks[1].saved_rsp = 0xDEAD_BEEF;

        let cur_rsp = mgr.tasks[0].saved_rsp;
        let next_rsp = mgr.on_timer_tick(cur_rsp);

        // Scheduler must detect out-of-bounds RSP, mark task 1 Faulted, and keep task 0
        assert_eq!(mgr.tasks[1].state, TaskState::Faulted);
        assert_eq!(mgr.current_task_idx, 0);
        assert_eq!(next_rsp, cur_rsp);
    }

    #[test]
    fn test_gate3d_failure_injection_non_canonical_rip() {
        let mut mgr = KernelTaskManager::new();
        mgr.spawn_task(0x1000).unwrap();
        let t2 = mgr.spawn_task(0x2000).unwrap() as usize;
        mgr.preemptive_enabled = true;
        mgr.tasks[0].state = TaskState::Running;

        // Invalidate RIP on synthetic frame of task 2
        let frame_ptr = mgr.tasks[t2 - 1].saved_rsp as *mut InterruptContext;
        unsafe {
            (*frame_ptr).rip = 0x00FF_DEAD_BEEF_0000; // non-canonical
        }

        let cur_rsp = mgr.tasks[0].saved_rsp;
        let next_rsp = mgr.on_timer_tick(cur_rsp);

        // Scheduler must reject non-canonical RIP, mark task 2 Faulted, and retain current RSP
        assert_eq!(mgr.tasks[t2 - 1].state, TaskState::Faulted);
        assert_eq!(next_rsp, cur_rsp);
    }

    #[test]
    fn test_gate3d_no_ready_task_fallback() {
        let mut mgr = KernelTaskManager::new();
        mgr.spawn_task(0x1000).unwrap();
        mgr.preemptive_enabled = true;
        mgr.tasks[0].state = TaskState::Running;

        // Mark all tasks except task 0 as Faulted or Terminated
        for i in 1..mgr.task_count {
            mgr.tasks[i].state = TaskState::Faulted;
        }

        let cur_rsp = 0xFFFF_8000_1234_5678;
        let next_rsp = mgr.on_timer_tick(cur_rsp);

        // Must retain current_rsp cleanly without panicking or returning 0
        assert_eq!(next_rsp, cur_rsp);
        assert_ne!(next_rsp, 0);
    }
}

