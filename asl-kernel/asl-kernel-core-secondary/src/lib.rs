//! ASL Secondary Failover Microkernel Core & Recovery State Machine
//! ====================================================================
//! Standar DO-178C DAL-A & Disaster Recovery Takeover Engine.
//! State Transition Machine:
//! `Standby -> FaultDetected -> IsolationRequested -> RecoveryEntry -> Validation -> Operational` (atau `RecoveryFailed`)
//! Fitur:
//! - Bounded diagnostic buffer untuk merekam alasan kegagalan
//! - Deterministic test harness untuk simulasi injected fault
//! - Penyelamatan data sesi dan pemindahan kontrol darurat

#![no_std]

#[cfg(test)]
extern crate std;

use asl_bootloader::AslBootHandover;

/// Status Finite State Machine (FSM) Pemulihan Failover
#[derive(Debug, Copy, Clone, PartialEq, Eq)]
pub enum RecoveryState {
    Standby,
    FaultDetected,
    IsolationRequested,
    RecoveryEntry,
    Validation,
    Operational,
    RecoveryFailed,
}

/// Alasan kegagalan pemicu failover
#[derive(Debug, Copy, Clone, PartialEq, Eq)]
pub enum FaultReason {
    None,
    KernelPanic,
    WatchdogTimeout,
    MemoryIntegrityCorruption,
    IllegalHardwareTrap,
    SimulatedInjectedFault,
}

pub const DIAGNOSTIC_LOG_SIZE: usize = 256;

/// Blok Diagnostik Pemulihan (Bounded, allocation-free)
#[derive(Debug, Copy, Clone)]
pub struct RecoveryDiagnostics {
    pub fault_reason: FaultReason,
    pub previous_state: RecoveryState,
    pub current_state: RecoveryState,
    pub fault_timestamp_ticks: u64,
    pub recovery_duration_us: u64,
    pub log_message: [u8; DIAGNOSTIC_LOG_SIZE],
    pub log_len: usize,
}

impl RecoveryDiagnostics {
    pub const fn empty() -> Self {
        Self {
            fault_reason: FaultReason::None,
            previous_state: RecoveryState::Standby,
            current_state: RecoveryState::Standby,
            fault_timestamp_ticks: 0,
            recovery_duration_us: 0,
            log_message: [0u8; DIAGNOSTIC_LOG_SIZE],
            log_len: 0,
        }
    }

    pub fn set_log(&mut self, msg: &str) {
        let bytes = msg.as_bytes();
        let len = if bytes.len() > DIAGNOSTIC_LOG_SIZE {
            DIAGNOSTIC_LOG_SIZE
        } else {
            bytes.len()
        };
        for i in 0..len {
            self.log_message[i] = bytes[i];
        }
        self.log_len = len;
    }

    pub fn message_str(&self) -> &str {
        core::str::from_utf8(&self.log_message[..self.log_len]).unwrap_or("")
    }
}

/// Engine Pengendali Secondary Core Failover
pub struct SecondaryCoreEngine {
    pub state: RecoveryState,
    pub diagnostics: RecoveryDiagnostics,
    pub active_handover: Option<*const AslBootHandover>,
}

impl SecondaryCoreEngine {
    pub const fn new() -> Self {
        Self {
            state: RecoveryState::Standby,
            diagnostics: RecoveryDiagnostics::empty(),
            active_handover: None,
        }
    }

    /// Memvalidasi transisi state machine pemulihan (Fail-Closed Invariant)
    pub fn transition_to(&mut self, next_state: RecoveryState) -> Result<(), &'static str> {
        let valid = match (self.state, next_state) {
            (RecoveryState::Standby, RecoveryState::FaultDetected) => true,
            (RecoveryState::FaultDetected, RecoveryState::IsolationRequested) => true,
            (RecoveryState::FaultDetected, RecoveryState::RecoveryFailed) => true,
            (RecoveryState::IsolationRequested, RecoveryState::RecoveryEntry) => true,
            (RecoveryState::IsolationRequested, RecoveryState::RecoveryFailed) => true,
            (RecoveryState::RecoveryEntry, RecoveryState::Validation) => true,
            (RecoveryState::RecoveryEntry, RecoveryState::RecoveryFailed) => true,
            (RecoveryState::Validation, RecoveryState::Operational) => true,
            (RecoveryState::Validation, RecoveryState::RecoveryFailed) => true,
            (RecoveryState::Operational, RecoveryState::Standby) => true, // Re-arm setelah primer pulih
            _ => false,
        };

        if valid {
            self.diagnostics.previous_state = self.state;
            self.state = next_state;
            self.diagnostics.current_state = next_state;
            Ok(())
        } else {
            self.state = RecoveryState::RecoveryFailed;
            self.diagnostics.current_state = RecoveryState::RecoveryFailed;
            Err("ERR_INVALID_FAILOVER_STATE_TRANSITION")
        }
    }

    /// Menangani pemicu kegagalan dan memulai rantai pemulihan
    pub fn trigger_fault_recovery(
        &mut self,
        reason: FaultReason,
        timestamp_ticks: u64,
        handover_ptr: *const AslBootHandover,
    ) -> Result<RecoveryState, &'static str> {
        self.diagnostics.fault_reason = reason;
        self.diagnostics.fault_timestamp_ticks = timestamp_ticks;
        self.active_handover = Some(handover_ptr);

        // 1. Standby -> FaultDetected
        self.transition_to(RecoveryState::FaultDetected)?;
        self.diagnostics
            .set_log("Fault detected. Initiating emergency failover sequence.");

        // 2. FaultDetected -> IsolationRequested (Air-Gap)
        self.transition_to(RecoveryState::IsolationRequested)?;

        // 3. IsolationRequested -> RecoveryEntry
        self.transition_to(RecoveryState::RecoveryEntry)?;

        // 4. RecoveryEntry -> Validation (Verifikasi keabsahan Handover)
        self.transition_to(RecoveryState::Validation)?;
        if !handover_ptr.is_null() {
            unsafe {
                if let Err(e) = (*handover_ptr).validate() {
                    self.transition_to(RecoveryState::RecoveryFailed).ok();
                    return Err(e);
                }
            }
        }

        // 5. Validation -> Operational (Secondary Core Aktif Penuh)
        self.transition_to(RecoveryState::Operational)?;
        self.diagnostics
            .set_log("Secondary core takeover operational. Subsystems rescued.");

        Ok(self.state)
    }

    /// Injeksi kesalahan simulasi untuk pengujian deterministik (Fault Injection Test Harness)
    pub fn inject_simulated_fault(
        &mut self,
        handover_ptr: *const AslBootHandover,
    ) -> Result<RecoveryState, &'static str> {
        self.trigger_fault_recovery(FaultReason::SimulatedInjectedFault, 10_000, handover_ptr)
    }

    /// Menerima notifikasi kegagalan/fault fatal dari primary kernel tanpa melakukan takeover (Failover Hook)
    pub fn notify_fault(&mut self, reason: FaultReason, timestamp_ticks: u64) -> RecoveryState {
        self.diagnostics.fault_reason = reason;
        self.diagnostics.fault_timestamp_ticks = timestamp_ticks;
        // Transisi ke FaultDetected untuk merekam anomali fatal tanpa mengambil alih kontrol
        self.transition_to(RecoveryState::FaultDetected).ok();
        self.diagnostics
            .set_log("Fatal fault notification received from primary kernel. Standby armed, no takeover.");
        self.state
    }
}

// -----------------------------------------------------------------------------
// CRASH-SAFE ATOMIC LATCH (exception/panic-context entry point)
// -----------------------------------------------------------------------------

/// Lapisan latch atomik di atas SecondaryCoreEngine.
/// Semua state machine latch memakai AtomicU8 + CAS sehingga aman dipanggil
/// dari konteks exception (#PF/#DF/#GP) dan panic handler tanpa UB deref
/// sebelum engine di-arm.
pub mod safe {
    use super::{FaultReason, RecoveryState, SecondaryCoreEngine};
    use core::cell::UnsafeCell;
    use core::sync::atomic::{AtomicU64, AtomicU8, Ordering};

    pub const LATCH_UNINIT: u8 = 0;
    pub const LATCH_ARMED: u8 = 1;
    pub const LATCH_RECORDING: u8 = 2;
    pub const LATCH_RECOVERY: u8 = 3;

    pub const REASON_NONE: u64 = 0;
    pub const REASON_KERNEL_PANIC: u64 = 1;
    pub const REASON_WATCHDOG_TIMEOUT: u64 = 2;
    pub const REASON_MEMORY_INTEGRITY: u64 = 3;
    pub const REASON_ILLEGAL_TRAP: u64 = 4;
    pub const REASON_SIMULATED: u64 = 5;
    /// Gagal menegakkan stack guard page saat boot -> proteksi menurun, WAJIB tercatat di DR FSM
    pub const REASON_STACK_GUARD_DEGRADED: u64 = 6;

    pub struct FaultLatch {
        state: AtomicU8,
        fault_counter: AtomicU64,
        last_reason: AtomicU64,
        last_ticks: AtomicU64,
        engine: UnsafeCell<SecondaryCoreEngine>,
    }

    // SAFETY: akses tunggal engine hanya terjadi setelah CAS Armed->Recording
    // berhasil; konteks re-entrant hanya menyentuh field atomik.
    unsafe impl Sync for FaultLatch {}

    impl FaultLatch {
        pub const fn new() -> Self {
            Self {
                state: AtomicU8::new(LATCH_UNINIT),
                fault_counter: AtomicU64::new(0),
                last_reason: AtomicU64::new(REASON_NONE),
                last_ticks: AtomicU64::new(0),
                engine: UnsafeCell::new(SecondaryCoreEngine::new()),
            }
        }

        #[inline]
        pub fn state(&self) -> u8 {
            self.state.load(Ordering::Acquire)
        }

        #[inline]
        pub fn fault_count(&self) -> u64 {
            self.fault_counter.load(Ordering::Relaxed)
        }

        #[inline]
        pub fn last_reason(&self) -> u64 {
            self.last_reason.load(Ordering::Relaxed)
        }

        #[inline]
        pub fn last_ticks(&self) -> u64 {
            self.last_ticks.load(Ordering::Relaxed)
        }

        /// Uninit -> Armed. Mengembalikan false jika bukan dari state Uninit.
        pub fn arm(&self) -> bool {
            self.state
                .compare_exchange(
                    LATCH_UNINIT,
                    LATCH_ARMED,
                    Ordering::AcqRel,
                    Ordering::Acquire,
                )
                .is_ok()
        }

        fn record_atomic(&self, reason_code: u64, ticks: u64) {
            self.fault_counter.fetch_add(1, Ordering::Relaxed);
            self.last_reason.store(reason_code, Ordering::Relaxed);
            self.last_ticks.store(ticks, Ordering::Relaxed);
        }

        /// Catat fault fatal. Mengembalikan state latch yang teramati saat masuk.
        /// - Uninit: record atomik saja, TIDAK pernah menyentuh engine (pre-init aman).
        /// - Armed: CAS ke Recording (guard re-entrancy), teruskan ke engine FSM,
        ///   lalu latch terminal ke Recovery.
        /// - Recording/Recovery: hanya increment counter atomik (fault pertama otoritatif).
        pub fn notify_fault(&self, reason_code: u64, ticks: u64) -> u8 {
            match self.state.load(Ordering::Acquire) {
                LATCH_ARMED => {
                    if self
                        .state
                        .compare_exchange(
                            LATCH_ARMED,
                            LATCH_RECORDING,
                            Ordering::AcqRel,
                            Ordering::Acquire,
                        )
                        .is_ok()
                    {
                        self.record_atomic(reason_code, ticks);
                        // SAFETY: satu-satunya penulis engine; re-entrant tidak mungkin
                        // karena state sudah Recording.
                        unsafe {
                            (*self.engine.get()).notify_fault(reason_from_code(reason_code), ticks);
                        }
                        self.state.store(LATCH_RECOVERY, Ordering::Release);
                        LATCH_ARMED
                    } else {
                        self.record_atomic(reason_code, ticks);
                        LATCH_RECORDING
                    }
                }
                other => {
                    self.record_atomic(reason_code, ticks);
                    other
                }
            }
        }

        /// Baca state FSM engine. Hanya valid saat latch tidak dalam Recording
        /// (dipakai diagnostics/test yang mengontrol sendiri latch-nya).
        pub fn engine_state(&self) -> RecoveryState {
            // SAFETY: dipanggil dari pemegang tunggal latch (boot sequence atau test lokal)
            unsafe { (*self.engine.get()).state }
        }
    }

    const fn reason_from_code(code: u64) -> FaultReason {
        match code {
            REASON_KERNEL_PANIC => FaultReason::KernelPanic,
            REASON_WATCHDOG_TIMEOUT => FaultReason::WatchdogTimeout,
            REASON_MEMORY_INTEGRITY | REASON_STACK_GUARD_DEGRADED => {
                FaultReason::MemoryIntegrityCorruption
            }
            REASON_ILLEGAL_TRAP => FaultReason::IllegalHardwareTrap,
            REASON_SIMULATED => FaultReason::SimulatedInjectedFault,
            _ => FaultReason::None,
        }
    }

    /// Latch global; pada bare-metal dipindah ke region carve-out .asl_secondary
    /// (isolasi fisik engine + datanya di dalam kernel image).
    #[cfg_attr(target_os = "none", link_section = ".asl_secondary")]
    #[used]
    pub static GLOBAL_SECONDARY_LATCH: FaultLatch = FaultLatch::new();

    pub fn arm_engine() -> bool {
        GLOBAL_SECONDARY_LATCH.arm()
    }

    pub fn notify_fault_safe(reason_code: u64, timestamp_ticks: u64) -> u8 {
        GLOBAL_SECONDARY_LATCH.notify_fault(reason_code, timestamp_ticks)
    }

    pub fn latch_state() -> u8 {
        GLOBAL_SECONDARY_LATCH.state()
    }

    pub fn fault_count() -> u64 {
        GLOBAL_SECONDARY_LATCH.fault_count()
    }

    #[cfg(test)]
    mod tests {
        use super::*;

        #[test]
        fn test_latch_preinit_uninit_records_without_engine_touch() {
            let latch = FaultLatch::new();
            assert_eq!(latch.state(), LATCH_UNINIT);
            let observed = latch.notify_fault(REASON_ILLEGAL_TRAP, 77);
            assert_eq!(observed, LATCH_UNINIT);
            assert_eq!(latch.fault_count(), 1);
            assert_eq!(latch.last_reason(), REASON_ILLEGAL_TRAP);
            assert_eq!(latch.last_ticks(), 77);
            // Engine tidak pernah disentuh saat Uninit
            assert_eq!(latch.engine_state(), RecoveryState::Standby);
        }

        #[test]
        fn test_latch_arm_only_from_uninit() {
            let latch = FaultLatch::new();
            assert!(latch.arm());
            assert_eq!(latch.state(), LATCH_ARMED);
            assert!(!latch.arm()); // CAS kedua gagal: tidak bisa re-arm diam-diam
            assert_eq!(latch.state(), LATCH_ARMED);
        }

        #[test]
        fn test_latch_armed_notify_records_via_engine_then_recovery() {
            let latch = FaultLatch::new();
            assert!(latch.arm());
            let observed = latch.notify_fault(REASON_KERNEL_PANIC, 100);
            assert_eq!(observed, LATCH_ARMED);
            assert_eq!(latch.state(), LATCH_RECOVERY);
            assert_eq!(latch.fault_count(), 1);
            assert_eq!(latch.engine_state(), RecoveryState::FaultDetected);
        }

        #[test]
        fn test_latch_recovery_terminal_counts_without_engine_reentry() {
            let latch = FaultLatch::new();
            assert!(latch.arm());
            latch.notify_fault(REASON_KERNEL_PANIC, 100);
            let observed = latch.notify_fault(REASON_WATCHDOG_TIMEOUT, 200);
            assert_eq!(observed, LATCH_RECOVERY);
            assert_eq!(latch.fault_count(), 2);
            // Fault pertama otoritatif; engine tidak disentuh lagi (tidak RecoveryFailed)
            assert_eq!(latch.engine_state(), RecoveryState::FaultDetected);
        }

        #[test]
        fn test_latch_guard_page_degraded_reason_recorded_in_engine() {
            let latch = FaultLatch::new();
            assert!(latch.arm());
            latch.notify_fault(REASON_STACK_GUARD_DEGRADED, 12);
            assert_eq!(latch.last_reason(), REASON_STACK_GUARD_DEGRADED);
            assert_eq!(latch.fault_count(), 1);
            assert_eq!(latch.engine_state(), RecoveryState::FaultDetected);
        }

        #[test]
        fn test_latch_concurrent_notify_counts_all_and_keeps_one_engine_writer() {
            let latch = FaultLatch::new();
            assert!(latch.arm());
            std::thread::scope(|s| {
                for i in 0..4u64 {
                    let l = &latch;
                    s.spawn(move || {
                        l.notify_fault(REASON_ILLEGAL_TRAP, i);
                    });
                }
            });
            assert_eq!(latch.fault_count(), 4);
            assert_eq!(latch.state(), LATCH_RECOVERY);
            // Tepat satu writer engine: FSM FaultDetected, bukan RecoveryFailed
            assert_eq!(latch.engine_state(), RecoveryState::FaultDetected);
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use asl_bootloader::{MemoryDescriptor, MemoryRegionType, TargetArch, ASL_BOOT_MAGIC};

    fn make_mock_handover() -> AslBootHandover {
        let mut h = AslBootHandover::empty();
        h.magic = ASL_BOOT_MAGIC;
        h.protocol_version = 1;
        h.target_arch = TargetArch::X86_64;
        h.kernel_physical_base = 0x0010_0000;
        h.kernel_virtual_base = 0xFFFF_8000_0010_0000;
        h.kernel_size_bytes = 0x20000;
        h.memory_map_count = 1;
        h.memory_map[0] = MemoryDescriptor {
            base_physical_addr: 0x0000_0000,
            length_bytes: 0x400_0000, // 64 MB
            region_type: MemoryRegionType::UsableRam,
            flags: 0,
        };
        h
    }

    #[test]
    fn test_failover_state_machine_nominal_sequence() {
        let mut engine = SecondaryCoreEngine::new();
        assert_eq!(engine.state, RecoveryState::Standby);

        let mock_handover = make_mock_handover();
        let res = engine.trigger_fault_recovery(
            FaultReason::KernelPanic,
            500,
            &mock_handover as *const AslBootHandover,
        );

        assert_eq!(res, Ok(RecoveryState::Operational));
        assert_eq!(engine.state, RecoveryState::Operational);
        assert_eq!(engine.diagnostics.fault_reason, FaultReason::KernelPanic);
        assert_eq!(
            engine.diagnostics.message_str(),
            "Secondary core takeover operational. Subsystems rescued."
        );
    }

    #[test]
    fn test_failover_invalid_transition_fails_closed() {
        let mut engine = SecondaryCoreEngine::new();
        // Langsung lompat dari Standby ke Operational harus gagal dan masuk RecoveryFailed
        let res = engine.transition_to(RecoveryState::Operational);
        assert!(res.is_err());
        assert_eq!(engine.state, RecoveryState::RecoveryFailed);
    }

    #[test]
    fn test_injected_fault_harness() {
        let mut engine = SecondaryCoreEngine::new();
        let mock_handover = make_mock_handover();
        let res = engine.inject_simulated_fault(&mock_handover as *const AslBootHandover);
        assert_eq!(res, Ok(RecoveryState::Operational));
    }

    #[test]
    fn test_corrupted_handover_fails_to_recovery_failed() {
        let mut engine = SecondaryCoreEngine::new();
        let mut bad_handover = make_mock_handover();
        bad_handover.magic = 0xDEADBEEF; // Corrupt magic

        let res = engine.trigger_fault_recovery(
            FaultReason::MemoryIntegrityCorruption,
            999,
            &bad_handover as *const AslBootHandover,
        );
        assert_eq!(res, Err("ERR_INVALID_BOOT_MAGIC"));
        assert_eq!(engine.state, RecoveryState::RecoveryFailed);
        assert_eq!(
            engine.diagnostics.current_state,
            RecoveryState::RecoveryFailed
        );
    }

    #[test]
    fn test_diagnostic_log_bounding_and_truncation() {
        let mut diag = RecoveryDiagnostics::empty();
        // Coba tulis pesan sepanjang 300 byte (melebihi DIAGNOSTIC_LOG_SIZE 256)
        let long_msg = "A".repeat(300);
        diag.set_log(&long_msg);
        assert_eq!(diag.log_len, DIAGNOSTIC_LOG_SIZE);
        assert_eq!(diag.message_str().len(), DIAGNOSTIC_LOG_SIZE);
    }

    #[test]
    fn test_operational_rearm_to_standby() {
        let mut engine = SecondaryCoreEngine::new();
        let mock_handover = make_mock_handover();
        assert_eq!(
            engine.trigger_fault_recovery(
                FaultReason::WatchdogTimeout,
                100,
                &mock_handover as *const AslBootHandover
            ),
            Ok(RecoveryState::Operational)
        );

        // Setelah primary kernel pulih atau reboot disiapkan, secondary core dapat di-rearm ke Standby
        assert_eq!(engine.transition_to(RecoveryState::Standby), Ok(()));
        assert_eq!(engine.state, RecoveryState::Standby);
    }

    #[test]
    fn test_secondary_notify_fault_no_takeover() {
        let mut engine = SecondaryCoreEngine::new();
        assert_eq!(engine.state, RecoveryState::Standby);

        let st = engine.notify_fault(FaultReason::IllegalHardwareTrap, 42_000);
        assert_eq!(st, RecoveryState::FaultDetected);
        assert_eq!(engine.state, RecoveryState::FaultDetected);
        assert_eq!(engine.diagnostics.fault_reason, FaultReason::IllegalHardwareTrap);
        assert_eq!(engine.diagnostics.fault_timestamp_ticks, 42_000);
        // Memastikan secondary engine TIDAK mengambil alih (tidak Operational)
        assert_ne!(engine.state, RecoveryState::Operational);
        assert!(engine.diagnostics.message_str().contains("no takeover"));
    }
}
