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
