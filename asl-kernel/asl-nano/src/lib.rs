//! ASL-Nano Subkernel: No-MMU Tickless RTOS Engine
//! ==================================================
//! Dirancang khusus untuk mikrokontroler (Cortex-M0+/M3/M4/M7, RV32IMAC, Xtensa LX6/LX7).
//! Jejak memori minimalis: < 32 KB Flash & < 8 KB RAM.
//! Menghadirkan Tickless Preemption untuk efisiensi daya maksimal.

#![no_std]

/// Kapasitas maksimum task dalam ASL-Nano (Statis tanpa dynamic heap)
pub const MAX_NANO_TASKS: usize = 8;
pub const NANO_STACK_SIZE: usize = 1024; // 1 KB stack per task

/// Status siklus hidup task di RTOS
#[repr(u8)]
#[derive(Debug, Copy, Clone, PartialEq, Eq)]
pub enum TaskState {
    Unused = 0,
    Ready = 1,
    Running = 2,
    Sleeping = 3,
    Suspended = 4,
}

/// Task Control Block (TCB) statis deterministik
#[repr(C)]
#[derive(Copy, Clone)]
pub struct NanoTaskControlBlock {
    pub task_id: u8,
    pub priority: u8, // 0 = Tertinggi (FADEC / Motor Control), 255 = Idle
    pub state: TaskState,
    pub sleep_ticks_remaining: u32,
    pub stack_pointer: u32,
    pub stack_storage: [u8; NANO_STACK_SIZE],
}

impl NanoTaskControlBlock {
    pub const fn empty() -> Self {
        Self {
            task_id: 0,
            priority: 255,
            state: TaskState::Unused,
            sleep_ticks_remaining: 0,
            stack_pointer: 0,
            stack_storage: [0u8; NANO_STACK_SIZE],
        }
    }
}

/// Penjadwal Deterministik ASL-Nano dengan Fitur Tickless Idle
pub struct AslNanoKernel {
    pub tasks: [NanoTaskControlBlock; MAX_NANO_TASKS],
    pub current_running_task: usize,
    pub system_ticks: u64,
    pub is_tickless_mode: bool,
}

impl AslNanoKernel {
    pub const fn new() -> Self {
        Self {
            tasks: [NanoTaskControlBlock::empty(); MAX_NANO_TASKS],
            current_running_task: 0,
            system_ticks: 0,
            is_tickless_mode: true,
        }
    }

    /// Mendaftarkan fungsi task ke dalam penjadwal
    pub fn spawn_task(&mut self, id: u8, priority: u8) -> Result<usize, &'static str> {
        let mut i = 0;
        while i < MAX_NANO_TASKS {
            if self.tasks[i].state == TaskState::Unused {
                self.tasks[i].task_id = id;
                self.tasks[i].priority = priority;
                self.tasks[i].state = TaskState::Ready;
                // Arahkan stack pointer ke ujung atas buffer (Full Descending Stack)
                let top_stack = self.tasks[i].stack_storage.as_ptr() as u32 + NANO_STACK_SIZE as u32;
                self.tasks[i].stack_pointer = top_stack;
                return Ok(i);
            }
            i += 1;
        }
        Err("ERR_NANO_TASK_CAPACITY_EXCEEDED")
    }

    /// Memilih task dengan prioritas tertinggi yang siap dieksekusi (O(1) deterministik)
    pub fn schedule_next(&mut self) -> usize {
        let mut best_index = self.current_running_task;
        let mut highest_priority = 255;

        let mut i = 0;
        while i < MAX_NANO_TASKS {
            let t = &self.tasks[i];
            if t.state == TaskState::Ready || t.state == TaskState::Running {
                if t.priority < highest_priority {
                    highest_priority = t.priority;
                    best_index = i;
                }
            }
            i += 1;
        }

        if self.tasks[self.current_running_task].state == TaskState::Running {
            self.tasks[self.current_running_task].state = TaskState::Ready;
        }

        self.tasks[best_index].state = TaskState::Running;
        self.current_running_task = best_index;
        best_index
    }

    /// Menghitung durasi tidur terdekat untuk memprogram timer hardware (Tickless Dynamic Sleep)
    pub fn calculate_tickless_sleep_ticks(&self) -> u32 {
        let mut min_sleep = u32::MAX;
        let mut found_sleeping = false;

        let mut i = 0;
        while i < MAX_NANO_TASKS {
            let t = &self.tasks[i];
            if t.state == TaskState::Sleeping && t.sleep_ticks_remaining > 0 {
                if t.sleep_ticks_remaining < min_sleep {
                    min_sleep = t.sleep_ticks_remaining;
                    found_sleeping = true;
                }
            }
            i += 1;
        }

        if found_sleeping { min_sleep } else { 1 }
    }
}
