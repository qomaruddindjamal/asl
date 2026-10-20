//! Local APIC & APIC Timer Subsystem (x86_64)
//! ============================================
//! DO-178C DAL-A alignment work in progress / inspired deterministic engineering.
//! Menyediakan:
//! - Local APIC Discovery via IA32_APIC_BASE MSR (0x1B)
//! - Spurious Interrupt Vector Register (SVR) configuration (Vector 0xFF, Software Enable)
//! - APIC Timer Periodic Mode (Vector 32 / 0x20)
//! - End of Interrupt (EOI) register writes
//! - Deterministic atomic global tick counter

use core::sync::atomic::{AtomicU64, Ordering};

pub const IA32_APIC_BASE_MSR: u32 = 0x1B;
pub const IA32_APIC_BASE_ENABLE: u64 = 1 << 11;
pub const DEFAULT_APIC_PHYS_BASE: u64 = 0xFEE0_0000;

// Register Offset Local APIC MMIO (4-byte registers spaced by 16 bytes)
pub const APIC_REG_ID: usize = 0x020;
pub const APIC_REG_VERSION: usize = 0x030;
pub const APIC_REG_TPR: usize = 0x080;
pub const APIC_REG_EOI: usize = 0x0B0;
pub const APIC_REG_SVR: usize = 0x0F0;
pub const APIC_REG_ESR: usize = 0x280;
pub const APIC_REG_LVT_TIMER: usize = 0x320;
pub const APIC_REG_TIMER_INIT_CNT: usize = 0x380;
pub const APIC_REG_TIMER_CURR_CNT: usize = 0x390;
pub const APIC_REG_TIMER_DIV: usize = 0x3E0;

// Konfigurasi Timer
pub const APIC_TIMER_VECTOR: u8 = 32; // Vector 0x20
pub const APIC_SPURIOUS_VECTOR: u8 = 0xFF;
pub const APIC_TIMER_MODE_PERIODIC: u32 = 1 << 17;
pub const APIC_TIMER_MASKED: u32 = 1 << 16;
pub const APIC_SVR_SOFTWARE_ENABLE: u32 = 1 << 8;

// Divide configuration: 0x03 = Divide by 16
pub const APIC_TIMER_DIV_16: u32 = 0x03;
// Initial count default untuk periodic tick (~100-1000 Hz tergantung clock bus)
pub const APIC_TIMER_DEFAULT_INIT_COUNT: u32 = 0x20000;
// Test initial count untuk fast bare-metal stress validation (QEMU virtual hardware)
pub const APIC_TIMER_TEST_INIT_COUNT: u32 = 0x20;

/// Global atomic tick counter untuk telemetri dan scheduler masa depan
pub static GLOBAL_TICK_COUNTER: AtomicU64 = AtomicU64::new(0);

// =========================================================================
// PART A: EVIDENCE COUNTER SEPARATION (Gate 3D Strict Evidence Mode)
// =========================================================================
/// Entry ISR Vector 32 counter: hanya bertambah saat ISR Timer dieksekusi
pub static APIC_INTERRUPT_COUNT: AtomicU64 = AtomicU64::new(0);
/// Switch counter: hanya bertambah saat interrupt Vector 32 benar-benar switch ke task berbeda
pub static APIC_PREEMPTIVE_SWITCH_COUNT: AtomicU64 = AtomicU64::new(0);
/// Interrupt tanpa pergantian task (misal quantum belum habis atau task sama)
pub static NO_SWITCH_INTERRUPT_COUNT: AtomicU64 = AtomicU64::new(0);
/// Cooperative switch counter: hanya untuk switch manual / yield
pub static COOPERATIVE_SWITCH_COUNT: AtomicU64 = AtomicU64::new(0);
/// Simulated cycle counter: hanya untuk host unit test / pemanggilan fungsi tanpa hardware interrupt
pub static SIMULATED_SCHEDULER_CYCLE_COUNT: AtomicU64 = AtomicU64::new(0);
/// EOI write counter: tepat satu kali per APIC timer interrupt entry
pub static EOI_WRITE_COUNT: AtomicU64 = AtomicU64::new(0);


/// Driver Local APIC Controller
pub struct LocalApic {
    pub virtual_base: u64,
    pub physical_base: u64,
}

impl LocalApic {
    pub const fn empty() -> Self {
        Self {
            virtual_base: 0,
            physical_base: 0,
        }
    }

    /// Membaca physical base address Local APIC dari MSR 0x1B
    pub unsafe fn read_apic_base() -> u64 {
        #[cfg(all(target_arch = "x86_64", target_os = "none"))]
        {
            let low: u32;
            let high: u32;
            core::arch::asm!(
                "rdmsr",
                in("ecx") IA32_APIC_BASE_MSR,
                out("eax") low,
                out("edx") high,
                options(nomem, nostack, preserves_flags)
            );
            ((high as u64) << 32) | (low as u64)
        }
        #[cfg(not(all(target_arch = "x86_64", target_os = "none")))]
        (DEFAULT_APIC_PHYS_BASE | IA32_APIC_BASE_ENABLE)
    }

    /// Inisialisasi Local APIC dari virtual address HHDM
    pub unsafe fn init(&mut self, virtual_base: u64, physical_base: u64) -> Result<(), &'static str> {
        let msr_val = Self::read_apic_base();
        let phys_from_msr = msr_val & 0x000F_FFFF_F000;
        if phys_from_msr != physical_base {
            return Err("ERR_APIC_PHYS_BASE_MISMATCH");
        }

        self.virtual_base = virtual_base;
        self.physical_base = physical_base;

        // 1. Aktifkan software enable via SVR (Spurious Interrupt Vector Register)
        // Set vector 0xFF dan bit 8 (APIC Software Enable)
        let svr_val = (APIC_SPURIOUS_VECTOR as u32) | APIC_SVR_SOFTWARE_ENABLE;
        self.write_reg(APIC_REG_SVR, svr_val);

        // 2. Set Task Priority Register ke 0 untuk menerima seluruh level interrupt
        self.write_reg(APIC_REG_TPR, 0);

        // 3. Clear Error Status Register
        self.write_reg(APIC_REG_ESR, 0);

        // 4. Kirim EOI awal untuk membersihkan status interrupt yang menggantung
        self.send_eoi();

        Ok(())
    }

    /// Konfigurasi dan mulai Local APIC Timer dalam Periodic Mode pada Vector 32 (0x20)
    pub unsafe fn start_periodic_timer(&mut self, init_count: u32) {
        // Step 1: Set divider konfigurasi (Divide by 16)
        self.write_reg(APIC_REG_TIMER_DIV, APIC_TIMER_DIV_16);

        // Step 2: Konfigurasi LVT Timer: Periodic mode + Vector 32 (Unmasked)
        let lvt_val = APIC_TIMER_MODE_PERIODIC | (APIC_TIMER_VECTOR as u32);
        self.write_reg(APIC_REG_LVT_TIMER, lvt_val);

        // Step 3: Muat Initial Count (memulai hitungan mundur timer secara otomatis)
        self.write_reg(APIC_REG_TIMER_INIT_CNT, init_count);
    }

    /// Mengirimkan sinyal End of Interrupt (EOI) ke APIC hardware
    #[inline(always)]
    pub unsafe fn send_eoi(&self) {
        if self.virtual_base != 0 {
            self.write_reg(APIC_REG_EOI, 0);
        }
    }

    /// Membaca register APIC MMIO 32-bit
    #[inline(always)]
    pub unsafe fn read_reg(&self, reg_offset: usize) -> u32 {
        let ptr = (self.virtual_base + reg_offset as u64) as *const u32;
        core::ptr::read_volatile(ptr)
    }

    /// Menulis register APIC MMIO 32-bit
    #[inline(always)]
    pub unsafe fn write_reg(&self, reg_offset: usize, val: u32) {
        let ptr = (self.virtual_base + reg_offset as u64) as *mut u32;
        core::ptr::write_volatile(ptr, val);
    }
}

/// Static singleton LocalApic controller instance
pub static mut STATIC_LOCAL_APIC: LocalApic = LocalApic::empty();

/// Callback hook opsional untuk secondary kernel heartbeat notification
pub type HeartbeatNotifyHook = fn(u64);
pub static mut SECONDARY_HEARTBEAT_HOOK: Option<HeartbeatNotifyHook> = None;

/// Callback hook opsional untuk scheduler tick (Gate 3 context switch trigger)
pub type SchedulerTickHook = fn(u64);
pub static mut SCHEDULER_TICK_HOOK: Option<SchedulerTickHook> = None;

/// Callback hook untuk preemptive scheduler dispatch: menerima current_rsp dan mengembalikan next_rsp
pub type PreemptiveSchedulerHook = extern "C" fn(u64) -> u64;
pub static mut PREEMPTIVE_SCHEDULER_HOOK: Option<PreemptiveSchedulerHook> = None;

/// Pasang hook callback heartbeat notification untuk secondary kernel
pub fn set_secondary_heartbeat_hook(hook: HeartbeatNotifyHook) {
    unsafe {
        SECONDARY_HEARTBEAT_HOOK = Some(hook);
    }
}

/// Pasang hook callback scheduler tick
pub fn set_scheduler_tick_hook(hook: SchedulerTickHook) {
    unsafe {
        SCHEDULER_TICK_HOOK = Some(hook);
    }
}

/// Pasang hook callback preemptive scheduler
pub fn set_preemptive_scheduler_hook(hook: PreemptiveSchedulerHook) {
    unsafe {
        PREEMPTIVE_SCHEDULER_HOOK = Some(hook);
    }
}

/// Handler interrupt tingkat Rust untuk APIC Timer (Vector 32)
/// Dipanggil langsung dari assembly ISR trampoline `isr_wrapper_apic_timer`
#[no_mangle]
pub extern "C" fn x86_64_apic_preemptive_handler(current_rsp: u64) -> u64 {
    // 1. Catat entry interrupt Vector 32 hardware secara deterministik
    APIC_INTERRUPT_COUNT.fetch_add(1, Ordering::Relaxed);
    let ticks = GLOBAL_TICK_COUNTER.fetch_add(1, Ordering::Relaxed) + 1;

    // 2. Notifikasi secondary kernel heartbeat hook jika terpasang
    if let Some(hook) = unsafe { SECONDARY_HEARTBEAT_HOOK } {
        hook(ticks);
    }

    // 3. Notifikasi scheduler tick hook jika terpasang
    if let Some(hook) = unsafe { SCHEDULER_TICK_HOOK } {
        hook(ticks);
    }

    // 4. Preemptive Task Scheduling: Tentukan task berikutnya
    let next_rsp = if let Some(sched) = unsafe { PREEMPTIVE_SCHEDULER_HOOK } {
        sched(current_rsp)
    } else {
        current_rsp
    };

    // 5. Klasifikasikan apakah interrupt menyebabkan pergantian context nyata
    if next_rsp != current_rsp && current_rsp != 0 {
        APIC_PREEMPTIVE_SWITCH_COUNT.fetch_add(1, Ordering::Relaxed);
    } else {
        NO_SWITCH_INTERRUPT_COUNT.fetch_add(1, Ordering::Relaxed);
    }

    // 6. EOI Guarantee: Kirim sinyal End of Interrupt ke hardware Local APIC tepat 1 kali
    unsafe {
        let apic_ptr = core::ptr::addr_of!(STATIC_LOCAL_APIC);
        (*apic_ptr).send_eoi();
    }
    EOI_WRITE_COUNT.fetch_add(1, Ordering::Relaxed);

    // 7. Kembalikan stack pointer yang akan dimuat ke RSP oleh assembly trampoline
    next_rsp
}

/// Handler backward compatibility untuk APIC timer
#[no_mangle]
pub extern "C" fn x86_64_apic_timer_handler() {
    let dummy_rsp = 0;
    let _ = x86_64_apic_preemptive_handler(dummy_rsp);
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_apic_base_msr_constants_and_validation() {
        let msr = IA32_APIC_BASE_MSR;
        assert_eq!(msr, 0x1B);
        let enable_bit = IA32_APIC_BASE_ENABLE;
        assert_eq!(enable_bit, 1 << 11);

        let default_val = DEFAULT_APIC_PHYS_BASE | IA32_APIC_BASE_ENABLE;
        assert_eq!(default_val & 0x000F_FFFF_F000, 0xFEE0_0000);
        assert_eq!((default_val >> 11) & 1, 1);
    }

    #[test]
    fn test_apic_timer_constants_and_vectors() {
        assert_eq!(APIC_TIMER_VECTOR, 32);
        assert_eq!(APIC_SPURIOUS_VECTOR, 0xFF);
        assert_eq!(APIC_TIMER_MODE_PERIODIC, 1 << 17);
        assert_eq!(APIC_SVR_SOFTWARE_ENABLE, 1 << 8);
    }

    // Mock MMIO buffer untuk menguji read/write Local APIC
    #[repr(C, align(4096))]
    struct MockApicMmio {
        regs: [u32; 1024],
    }

    static mut MOCK_MMIO: MockApicMmio = MockApicMmio { regs: [0; 1024] };

    #[test]
    fn test_local_apic_init_and_periodic_timer_mock() {
        let mmio_ptr = core::ptr::addr_of_mut!(MOCK_MMIO) as u64;
        let mut apic = LocalApic::empty();

        let init_res = unsafe { apic.init(mmio_ptr, DEFAULT_APIC_PHYS_BASE) };
        assert_eq!(init_res, Ok(()));

        // Verifikasi SVR terkonfigurasi dengan vector 0xFF dan bit 8
        let svr = unsafe { apic.read_reg(APIC_REG_SVR) };
        assert_eq!(svr, 0x1FF); // 0xFF | (1 << 8)

        // Verifikasi TPR = 0
        let tpr = unsafe { apic.read_reg(APIC_REG_TPR) };
        assert_eq!(tpr, 0);

        // Mulai periodic timer
        unsafe {
            apic.start_periodic_timer(0x10000);
        }

        let div = unsafe { apic.read_reg(APIC_REG_TIMER_DIV) };
        assert_eq!(div, APIC_TIMER_DIV_16);

        let lvt = unsafe { apic.read_reg(APIC_REG_LVT_TIMER) };
        assert_eq!(lvt, (1 << 17) | 32);

        let init_cnt = unsafe { apic.read_reg(APIC_REG_TIMER_INIT_CNT) };
        assert_eq!(init_cnt, 0x10000);
    }

    #[test]
    fn test_apic_timer_tick_increment_and_heartbeat_hook() {
        GLOBAL_TICK_COUNTER.store(0, Ordering::SeqCst);
        static mut HOOK_TICKS: u64 = 0;
        fn mock_hook(t: u64) {
            unsafe {
                HOOK_TICKS = t;
            }
        }

        set_secondary_heartbeat_hook(mock_hook);

        x86_64_apic_timer_handler();
        assert_eq!(GLOBAL_TICK_COUNTER.load(Ordering::SeqCst), 1);
        assert_eq!(unsafe { HOOK_TICKS }, 1);

        x86_64_apic_timer_handler();
        assert_eq!(GLOBAL_TICK_COUNTER.load(Ordering::SeqCst), 2);
        assert_eq!(unsafe { HOOK_TICKS }, 2);
    }
}
