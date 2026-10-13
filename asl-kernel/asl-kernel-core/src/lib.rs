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
use asl_afdx_ipc::{AfdxVirtualLinkRingBuffer, AfdxFrame};

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
    // Memasuki Safe Fallback Loop tanpa restart liar
    loop {
        core::hint::spin_loop();
    }
}

/// Mock entry point untuk pengujian arsitektural
///
/// # Safety
/// Caller must ensure `handover_ptr` is either null or points to a valid, initialized,
/// properly aligned `AslBootHandover` struct.
#[no_mangle]
pub unsafe extern "C" fn _asl_kernel_entry(handover_ptr: *const AslBootHandover) -> ! {
    let mut kernel = AslAvionicsKernel::new(TargetArch::X86_64);
    if !handover_ptr.is_null() {
        let handover = &*handover_ptr;
        let _ = kernel.cold_boot_init(handover);
    }

    loop {
        core::hint::spin_loop();
    }
}
