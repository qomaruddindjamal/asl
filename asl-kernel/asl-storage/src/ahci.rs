//! AHCI (Advanced Host Controller Interface) SATA HBA Discovery & Memory Layout
//! Standard Serial ATA AHCI 1.3.1 Specification

use crate::block::{StorageError, StorageResult};
use asl_driver_pci::{BarType, PciDeviceDescriptor};

pub const AHCI_PCI_CLASS_MASS_STORAGE: u8 = 0x01;
pub const AHCI_PCI_SUBCLASS_SATA: u8 = 0x06;
pub const AHCI_PCI_PROG_IF_AHCI: u8 = 0x01;

pub const HBA_GHC_AE: u32 = 1 << 31; // AHCI Enable
pub const HBA_GHC_IE: u32 = 1 << 1; // Interrupt Enable
pub const HBA_GHC_HR: u32 = 1 << 0; // HBA Reset

pub const HBA_PX_CMD_ST: u32 = 1 << 0; // Start
pub const HBA_PX_CMD_FRE: u32 = 1 << 4; // FIS Receive Enable
pub const HBA_PX_CMD_FR: u32 = 1 << 14; // FIS Receive Running
pub const HBA_PX_CMD_CR: u32 = 1 << 15; // Command List Running

pub const SATA_SIG_ATA: u32 = 0x00000101; // SATA Drive
pub const SATA_SIG_ATAPI: u32 = 0xEB140101; // SATAPI Drive
pub const SATA_SIG_SEMB: u32 = 0xC33C0101; // Enclosure Management
pub const SATA_SIG_PM: u32 = 0x96690101; // Port Multiplier

#[derive(Debug, Copy, Clone, PartialEq, Eq)]
pub enum AhciPortType {
    None,
    Sata,
    Satapi,
    Enclosure,
    PortMultiplier,
}

/// Generic Host Control (GHC) Memory Registers
#[repr(C)]
#[derive(Debug, Copy, Clone)]
pub struct HbaGenericHostControl {
    pub cap: u32,     // 0x00, Host Capability
    pub ghc: u32,     // 0x04, Global Host Control
    pub is: u32,      // 0x08, Interrupt Status
    pub pi: u32,      // 0x0C, Ports Implemented
    pub vs: u32,      // 0x10, Version
    pub ccc_ctl: u32, // 0x14, Command Coalescing Control
    pub ccc_pts: u32, // 0x18, Command Coalescing Ports
    pub em_loc: u32,  // 0x1C, Enclosure Management Location
    pub em_ctl: u32,  // 0x20, Enclosure Management Control
    pub cap2: u32,    // 0x24, Host Capabilities Extended
    pub bohc: u32,    // 0x28, BIOS/OS Handoff Control and Status
}

/// Port Memory Registers
#[repr(C)]
#[derive(Debug, Copy, Clone)]
pub struct HbaPortRegisters {
    pub clb: u32,         // 0x00, Command List Base Address (1KB aligned)
    pub clbu: u32,        // 0x04, Command List Base Address Upper 32-bits
    pub fb: u32,          // 0x08, FIS Base Address (256B aligned)
    pub fbu: u32,         // 0x0C, FIS Base Address Upper 32-bits
    pub is: u32,          // 0x10, Interrupt Status
    pub ie: u32,          // 0x14, Interrupt Enable
    pub cmd: u32,         // 0x18, Command and Status
    pub rsv0: u32,        // 0x1C, Reserved
    pub tfd: u32,         // 0x20, Task File Data
    pub sig: u32,         // 0x24, Signature
    pub ssts: u32,        // 0x28, SATA Status (SCR0: SStatus)
    pub sctl: u32,        // 0x2C, SATA Control (SCR2: SControl)
    pub serr: u32,        // 0x30, SATA Error (SCR1: SError)
    pub sact: u32,        // 0x34, SATA Active (SCR3: SActive)
    pub ci: u32,          // 0x38, Command Issue
    pub sntf: u32,        // 0x3C, SATA Notification (SCR4: SNotification)
    pub fbs: u32,         // 0x40, FIS-based Switching Control
    pub devslp: u32,      // 0x44, Device Sleep
    pub rsv1: [u32; 10],  // 0x48-0x6F, Reserved
    pub vendor: [u32; 4], // 0x70-0x7F, Vendor specific
}

#[derive(Debug, Copy, Clone, PartialEq, Eq)]
pub struct HbaPort {
    pub port_num: u8,
    pub port_type: AhciPortType,
    pub base_addr: usize,
    pub implemented: bool,
}

impl HbaPort {
    pub const fn empty() -> Self {
        Self {
            port_num: 0,
            port_type: AhciPortType::None,
            base_addr: 0,
            implemented: false,
        }
    }
}

pub struct AhciController {
    pub abar_mmio: usize,
    pub ports: [HbaPort; 32],
    pub port_count: usize,
    pub ports_implemented_mask: u32,
    pub version: u32,
}

impl AhciController {
    pub const fn new() -> Self {
        Self {
            abar_mmio: 0,
            ports: [HbaPort::empty(); 32],
            port_count: 0,
            ports_implemented_mask: 0,
            version: 0,
        }
    }

    /// Check if a PCI device matches an AHCI SATA Controller
    pub fn is_ahci_pci(dev: &PciDeviceDescriptor) -> bool {
        dev.class_code == AHCI_PCI_CLASS_MASS_STORAGE
            && dev.subclass == AHCI_PCI_SUBCLASS_SATA
            && dev.prog_if == AHCI_PCI_PROG_IF_AHCI
    }

    /// Discover AHCI controller from PCI device descriptor and validate BAR5 (ABAR)
    pub fn probe_pci(dev: &PciDeviceDescriptor) -> StorageResult<Self> {
        if !Self::is_ahci_pci(dev) {
            return Err(StorageError::DeviceNotFound);
        }

        // AHCI specifies BAR5 is the AHCI Base Address (ABAR) in Memory MMIO space
        let mmio_addr = match dev.bars[5].bar_type {
            BarType::Memory32 { address, .. } => address as usize,
            BarType::Memory64 { address, .. } => address as usize,
            _ => return Err(StorageError::CorruptedMetadata),
        };

        if mmio_addr == 0 {
            return Err(StorageError::InvalidParameter);
        }

        let mut controller = Self::new();
        controller.abar_mmio = mmio_addr;
        Ok(controller)
    }

    /// Read HBA Generic Host Control Registers from MMIO
    pub unsafe fn inspect_hba(&mut self) -> StorageResult<()> {
        if self.abar_mmio == 0 {
            return Err(StorageError::NotMounted);
        }

        let ghc_ptr = self.abar_mmio as *const HbaGenericHostControl;
        self.ports_implemented_mask = core::ptr::read_volatile(core::ptr::addr_of!((*ghc_ptr).pi));
        self.version = core::ptr::read_volatile(core::ptr::addr_of!((*ghc_ptr).vs));

        let mut count = 0;
        for i in 0..32 {
            if (self.ports_implemented_mask & (1 << i)) != 0 {
                let port_addr = self.abar_mmio + 0x100 + (i * 0x80);
                let port_ptr = port_addr as *const HbaPortRegisters;

                let ssts = core::ptr::read_volatile(core::ptr::addr_of!((*port_ptr).ssts));
                let sig = core::ptr::read_volatile(core::ptr::addr_of!((*port_ptr).sig));

                // Device detection flags (SStatus bits 0..3: DET 3 = Device present & Comm established)
                let det = ssts & 0x0F;
                let ipm = (ssts >> 8) & 0x0F;

                let port_type = if det == 3 && ipm == 1 {
                    match sig {
                        SATA_SIG_ATA => AhciPortType::Sata,
                        SATA_SIG_ATAPI => AhciPortType::Satapi,
                        SATA_SIG_SEMB => AhciPortType::Enclosure,
                        SATA_SIG_PM => AhciPortType::PortMultiplier,
                        _ => AhciPortType::None,
                    }
                } else {
                    AhciPortType::None
                };

                self.ports[count] = HbaPort {
                    port_num: i as u8,
                    port_type,
                    base_addr: port_addr,
                    implemented: true,
                };
                count += 1;
            }
        }
        self.port_count = count;
        Ok(())
    }
}
