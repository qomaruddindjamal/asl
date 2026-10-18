//! ASL Hardware Abstraction Layer (HAL) Core Traits
//! ================================================
//! Kontrak Abstraksi Hardware Universal untuk 150 Arsitektur Dunia.
//! Mengisolasi manipulasi register silikon, interrupt controller, timer deterministik,
//! dan Serial Debugger tanpa tergantung pada jenis CPU fisik.

#![no_std]

pub mod cache;
pub use cache::{
    BounceBufferPolicy, BounceDecision, CacheMaintenanceOps, CoherentNoopOps, IsolatedBounceBuffer,
    CACHE_LINE,
};

pub mod mmio;
pub use mmio::MmioRegister;

pub mod dma_space;
pub use dma_space::{
    DmaAddressFault, DmaAddressSpace, DmaDeviceAddress, DmaFaultRecord, IdentityDmaSpace,
    IommuTranslationSpace, MpuFirewallDmaSpace,
};

pub mod x86_fault;
pub use x86_fault::*;

pub mod apic;
pub use apic::*;

use asl_bootloader::TargetArch;

/// Trait pengontrol interupsi CPU (APIC/PIC di x86, GIC di ARM, PLIC di RISC-V, dsb.)
pub trait InterruptController {
    /// Inisialisasi interrupt controller
    fn init(&mut self) -> Result<(), &'static str>;
    /// Aktifkan interupsi hardware tertentu
    fn enable_irq(&mut self, irq: u32);
    /// Nonaktifkan interupsi hardware tertentu
    fn disable_irq(&mut self, irq: u32);
    /// Sinyal End of Interrupt (EOI)
    fn end_of_interrupt(&mut self, irq: u32);
}

/// Trait pewaktu deterministik avionik (Timer dengan resolusi sub-mikrodetik)
pub trait DeterministicTimer {
    /// Inisialisasi timer dengan frekuensi target (Hz)
    fn init(&mut self, frequency_hz: u64) -> Result<(), &'static str>;
    /// Membaca tick timer presisi tinggi saat ini (Hardware Clock Tick)
    fn current_ticks(&self) -> u64;
    /// Membaca nanodetik yang telah berlalu
    fn elapsed_nanos(&self) -> u64;
}

/// Trait output diagnostik serial awal (UART / Mini UART / Early Console)
pub trait EarlyDiagnosticConsole {
    /// Inisialisasi baudrate serial (default 115200)
    fn init(&mut self, base_addr: u64, baud_rate: u32) -> Result<(), &'static str>;
    /// Mengirimkan 1 byte data
    fn write_byte(&mut self, byte: u8);
    /// Mengirimkan string ASCII
    fn write_str(&mut self, s: &str) {
        for b in s.bytes() {
            self.write_byte(b);
        }
    }
}

/// Trait Manajemen Partisi Memori Hardware (MMU / MPU)
pub trait MemoryManagementUnit {
    /// Mengaktifkan proteksi halaman (Paging / MPU regions)
    fn enable_paging(&mut self) -> Result<(), &'static str>;
    /// Map halaman virtual ke alamat fisik
    fn map_page(&mut self, virt_addr: u64, phys_addr: u64, flags: u32) -> Result<(), &'static str>;
    /// Flush TLB (Translation Lookaside Buffer)
    fn flush_tlb(&mut self, virt_addr: u64);
}

/// State mesin arsitektur aktif
pub struct ArchitectureRuntime {
    pub arch: TargetArch,
    pub cpu_cores_count: u32,
    pub page_size: usize,
}

impl ArchitectureRuntime {
    pub const fn new(arch: TargetArch) -> Self {
        Self {
            arch,
            cpu_cores_count: 1,
            page_size: 4096, // Standar 4KB paging
        }
    }
}
