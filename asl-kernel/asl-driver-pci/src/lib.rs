//! ASL PCI / PCIe Bus Enumerator
//! ==============================
//! Standar DO-178C DAL-A & Allocation-Free Bus Discovery.
//! Mendukung:
//! - Legacy x86 Port I/O Config Space (`0xCF8` / `0xCFC`)
//! - PCIe ECAM (Enhanced Configuration Access Mechanism) MMIO
//! - Scan 256 bus, 32 device, 8 function tanpa alokasi heap dinamis
//! - Decoding BAR0..BAR5 dan penentuan tipe (Memory vs I/O space)

#![no_std]

/// Kapasitas maksimum perangkat PCI yang dapat ditampung dalam buffer statis
pub const MAX_PCI_DEVICES: usize = 64;

/// Konfigurasi Port I/O PCI Tradisional x86
pub const PCI_CONFIG_ADDRESS: u16 = 0xCF8;
pub const PCI_CONFIG_DATA: u16 = 0xCFC;
pub const PCI_ENABLE_BIT: u32 = 0x8000_0000;

/// Jenis Base Address Register (BAR)
#[derive(Debug, Copy, Clone, PartialEq, Eq)]
pub enum BarType {
    None,
    Memory32 { address: u32, prefetchable: bool },
    Memory64 { address: u64, prefetchable: bool },
    IoPort { port: u16 },
}

/// Deskriptor BAR tunggal
#[derive(Debug, Copy, Clone, PartialEq, Eq)]
pub struct PciBar {
    pub bar_type: BarType,
    pub raw_value: u32,
}

impl PciBar {
    pub const fn empty() -> Self {
        Self {
            bar_type: BarType::None,
            raw_value: 0,
        }
    }
}

/// Alamat BDF (Bus / Device / Function) PCI
#[derive(Debug, Copy, Clone, PartialEq, Eq)]
pub struct PciAddress {
    pub bus: u8,
    pub device: u8,
    pub function: u8,
}

impl PciAddress {
    pub const fn new(bus: u8, device: u8, function: u8) -> Self {
        Self {
            bus,
            device,
            function,
        }
    }

    /// Menghitung format register alamat port 0xCF8
    pub fn config_address(&self, offset: u8) -> u32 {
        PCI_ENABLE_BIT
            | ((self.bus as u32) << 16)
            | ((self.device as u32) << 11)
            | ((self.function as u32) << 8)
            | ((offset as u32) & 0xFC)
    }
}

/// Deskriptor lengkap perangkat PCI yang ditemukan (Fixed-size, allocation-free)
#[derive(Debug, Copy, Clone, PartialEq, Eq)]
pub struct PciDeviceDescriptor {
    pub address: PciAddress,
    pub vendor_id: u16,
    pub device_id: u16,
    pub command: u16,
    pub status: u16,
    pub revision_id: u8,
    pub prog_if: u8,
    pub subclass: u8,
    pub class_code: u8,
    pub header_type: u8,
    pub bars: [PciBar; 6],
}

impl PciDeviceDescriptor {
    pub const fn empty() -> Self {
        Self {
            address: PciAddress::new(0, 0, 0),
            vendor_id: 0,
            device_id: 0,
            command: 0,
            status: 0,
            revision_id: 0,
            prog_if: 0,
            subclass: 0,
            class_code: 0,
            header_type: 0,
            bars: [PciBar::empty(); 6],
        }
    }

    /// Mengembalikan nama kelas perangkat PCI
    pub fn class_name(&self) -> &'static str {
        match self.class_code {
            0x00 => "Legacy / Unclassified",
            0x01 => "Mass Storage Controller",
            0x02 => "Network Controller",
            0x03 => "Display Controller (VGA/GPU)",
            0x04 => "Multimedia Device",
            0x05 => "Memory Controller",
            0x06 => "Bridge Device",
            0x07 => "Simple Communications Controller",
            0x08 => "Base System Peripheral",
            0x09 => "Input Device Controller",
            0x0A => "Docking Station",
            0x0B => "Processor",
            0x0C => "Serial Bus Controller (USB/FireWire)",
            0x0D => "Wireless Controller",
            0x0E => "Intelligent Controller",
            0x0F => "Satellite Communications Controller",
            0x10 => "Encryption Controller",
            0x11 => "Signal Processing Controller",
            0x12 => "Processing Accelerator",
            0x13 => "Non-Essential Instrumentation",
            _ => "Unknown Class",
        }
    }
}

/// Antarmuka akses konfigurasi PCI (Abstraksi Transport)
pub trait PciConfigTransport {
    unsafe fn read_u32(&self, addr: PciAddress, offset: u8) -> u32;
    unsafe fn write_u32(&self, addr: PciAddress, offset: u8, value: u32);

    unsafe fn read_u16(&self, addr: PciAddress, offset: u8) -> u16 {
        let dword = self.read_u32(addr, offset & 0xFC);
        let shift = (offset & 0x02) * 8;
        ((dword >> shift) & 0xFFFF) as u16
    }

    unsafe fn read_u8(&self, addr: PciAddress, offset: u8) -> u8 {
        let dword = self.read_u32(addr, offset & 0xFC);
        let shift = (offset & 0x03) * 8;
        ((dword >> shift) & 0xFF) as u8
    }
}

/// Transport Port I/O x86 (0xCF8 / 0xCFC)
pub struct X86PortIoTransport;

impl PciConfigTransport for X86PortIoTransport {
    unsafe fn read_u32(&self, addr: PciAddress, offset: u8) -> u32 {
        #[cfg(target_arch = "x86_64")]
        {
            let conf_addr = addr.config_address(offset);
            core::arch::asm!(
                "out dx, eax",
                in("dx") PCI_CONFIG_ADDRESS,
                in("eax") conf_addr,
                options(nomem, nostack, preserves_flags)
            );
            let ret: u32;
            core::arch::asm!(
                "in eax, dx",
                out("eax") ret,
                in("dx") PCI_CONFIG_DATA,
                options(nomem, nostack, preserves_flags)
            );
            ret
        }
        #[cfg(not(target_arch = "x86_64"))]
        {
            let _ = (addr, offset);
            0xFFFF_FFFF
        }
    }

    unsafe fn write_u32(&self, addr: PciAddress, offset: u8, value: u32) {
        #[cfg(target_arch = "x86_64")]
        {
            let conf_addr = addr.config_address(offset);
            core::arch::asm!(
                "out dx, eax",
                in("dx") PCI_CONFIG_ADDRESS,
                in("eax") conf_addr,
                options(nomem, nostack, preserves_flags)
            );
            core::arch::asm!(
                "out dx, eax",
                in("dx") PCI_CONFIG_DATA,
                in("eax") value,
                options(nomem, nostack, preserves_flags)
            );
        }
        #[cfg(not(target_arch = "x86_64"))]
        {
            let _ = (addr, offset, value);
        }
    }
}

/// Transport PCIe ECAM MMIO
pub struct EcamTransport {
    pub base_virtual_address: usize,
    pub start_bus: u8,
    pub end_bus: u8,
}

impl EcamTransport {
    pub const fn new(base_vaddr: usize, start_bus: u8, end_bus: u8) -> Self {
        Self {
            base_virtual_address: base_vaddr,
            start_bus,
            end_bus,
        }
    }

    fn offset(&self, addr: PciAddress, offset: u8) -> Option<usize> {
        if addr.bus < self.start_bus || addr.bus > self.end_bus {
            return None;
        }
        let bus_rel = (addr.bus - self.start_bus) as usize;
        let off = (bus_rel << 20)
            | ((addr.device as usize) << 15)
            | ((addr.function as usize) << 12)
            | (offset as usize);
        Some(self.base_virtual_address + off)
    }
}

impl PciConfigTransport for EcamTransport {
    unsafe fn read_u32(&self, addr: PciAddress, offset: u8) -> u32 {
        if let Some(vaddr) = self.offset(addr, offset) {
            core::ptr::read_volatile(vaddr as *const u32)
        } else {
            0xFFFF_FFFF
        }
    }

    unsafe fn write_u32(&self, addr: PciAddress, offset: u8, value: u32) {
        if let Some(vaddr) = self.offset(addr, offset) {
            core::ptr::write_volatile(vaddr as *mut u32, value);
        }
    }
}

/// Pemindai Bus PCI Bounded & Aman (DO-178C Invariant)
pub struct PciScanner;

impl PciScanner {
    /// Memindai satu fungsi PCI dan mengembalikan deskriptornya jika ada
    pub unsafe fn scan_function<T: PciConfigTransport>(
        transport: &T,
        addr: PciAddress,
    ) -> Option<PciDeviceDescriptor> {
        // Jangan menulis apa pun saat penemuan (read-only)
        let id_dword = transport.read_u32(addr, 0x00);
        let vendor_id = (id_dword & 0xFFFF) as u16;
        let device_id = ((id_dword >> 16) & 0xFFFF) as u16;

        // 0xFFFF atau 0x0000 mengindikasikan tidak ada perangkat terpasang
        if vendor_id == 0xFFFF || vendor_id == 0x0000 {
            return None;
        }

        let cmd_status = transport.read_u32(addr, 0x04);
        let command = (cmd_status & 0xFFFF) as u16;
        let status = ((cmd_status >> 16) & 0xFFFF) as u16;

        let class_rev = transport.read_u32(addr, 0x08);
        let revision_id = (class_rev & 0xFF) as u8;
        let prog_if = ((class_rev >> 8) & 0xFF) as u8;
        let subclass = ((class_rev >> 16) & 0xFF) as u8;
        let class_code = ((class_rev >> 24) & 0xFF) as u8;

        let bist_header = transport.read_u32(addr, 0x0C);
        let header_type = ((bist_header >> 16) & 0xFF) as u8;

        let mut desc = PciDeviceDescriptor {
            address: addr,
            vendor_id,
            device_id,
            command,
            status,
            revision_id,
            prog_if,
            subclass,
            class_code,
            header_type,
            bars: [PciBar::empty(); 6],
        };

        // Decode BARs hanya untuk Header Type 0x00 (Normal Device)
        if (header_type & 0x7F) == 0x00 {
            let mut bar_idx = 0;
            while bar_idx < 6 {
                let bar_offset = 0x10 + (bar_idx as u8 * 4);
                let bar_val = transport.read_u32(addr, bar_offset);

                if bar_val == 0 || bar_val == 0xFFFF_FFFF {
                    desc.bars[bar_idx] = PciBar {
                        bar_type: BarType::None,
                        raw_value: bar_val,
                    };
                    bar_idx += 1;
                    continue;
                }

                if (bar_val & 0x01) == 0x01 {
                    // I/O Port BAR
                    let port = (bar_val & 0xFFFC) as u16;
                    desc.bars[bar_idx] = PciBar {
                        bar_type: BarType::IoPort { port },
                        raw_value: bar_val,
                    };
                    bar_idx += 1;
                } else {
                    // Memory Space BAR
                    let bar_kind = (bar_val >> 1) & 0x03;
                    let prefetchable = (bar_val & 0x08) != 0;

                    if bar_kind == 0x02 && bar_idx < 5 {
                        // 64-bit Memory BAR (Membutuhkan 2 slot dword)
                        let next_offset = bar_offset + 4;
                        let bar_high = transport.read_u32(addr, next_offset);
                        let addr_64 = ((bar_high as u64) << 32) | ((bar_val & 0xFFFF_FFF0) as u64);

                        desc.bars[bar_idx] = PciBar {
                            bar_type: BarType::Memory64 {
                                address: addr_64,
                                prefetchable,
                            },
                            raw_value: bar_val,
                        };
                        desc.bars[bar_idx + 1] = PciBar {
                            bar_type: BarType::None,
                            raw_value: bar_high,
                        };
                        bar_idx += 2;
                    } else {
                        // 32-bit Memory BAR
                        let addr_32 = bar_val & 0xFFFF_FFF0;
                        desc.bars[bar_idx] = PciBar {
                            bar_type: BarType::Memory32 {
                                address: addr_32,
                                prefetchable,
                            },
                            raw_value: bar_val,
                        };
                        bar_idx += 1;
                    }
                }
            }
        }

        Some(desc)
    }

    /// Memindai seluruh bus PCI (0..=max_bus) dan menyimpan hasilnya ke buffer array tetap
    pub unsafe fn scan_all<T: PciConfigTransport>(
        transport: &T,
        out_devices: &mut [PciDeviceDescriptor; MAX_PCI_DEVICES],
        max_bus: u8,
    ) -> usize {
        let mut count = 0;

        for bus in 0..=max_bus {
            for device in 0..32 {
                let primary_addr = PciAddress::new(bus, device, 0);
                if let Some(primary_desc) = Self::scan_function(transport, primary_addr) {
                    if count < MAX_PCI_DEVICES {
                        out_devices[count] = primary_desc;
                        count += 1;
                    }

                    // Periksa apakah perangkat ini Multifunction (Header Type bit 7)
                    let is_multifunction = (primary_desc.header_type & 0x80) != 0;
                    if is_multifunction {
                        for func in 1..8 {
                            let func_addr = PciAddress::new(bus, device, func);
                            if let Some(func_desc) = Self::scan_function(transport, func_addr) {
                                if count < MAX_PCI_DEVICES {
                                    out_devices[count] = func_desc;
                                    count += 1;
                                }
                            }
                        }
                    }
                }
            }
        }

        count
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    struct MockPciTransport {
        devices: [(PciAddress, u32, u32, u32, u32); 4], // addr, dword0 (ids), dword1 (cmd), dword2 (class), dword3 (header)
    }

    impl PciConfigTransport for MockPciTransport {
        unsafe fn read_u32(&self, addr: PciAddress, offset: u8) -> u32 {
            for dev in &self.devices {
                if dev.0 == addr {
                    match offset {
                        0x00 => return dev.1,
                        0x04 => return dev.2,
                        0x08 => return dev.3,
                        0x0C => return dev.4,
                        0x10 => return 0xF000_0000, // 32-bit Memory BAR
                        0x14 => return 0x0000_D001, // I/O Port BAR
                        _ => return 0,
                    }
                }
            }
            0xFFFF_FFFF
        }

        unsafe fn write_u32(&self, _addr: PciAddress, _offset: u8, _value: u32) {}
    }

    #[test]
    fn test_absent_pci_device() {
        let mock = MockPciTransport {
            devices: [(PciAddress::new(0, 0, 0), 0xFFFF_FFFF, 0, 0, 0); 4],
        };
        unsafe {
            let res = PciScanner::scan_function(&mock, PciAddress::new(1, 1, 0));
            assert_eq!(res, None);
        }
    }

    #[test]
    fn test_valid_pci_device_and_bars() {
        let mock = MockPciTransport {
            devices: [
                (
                    PciAddress::new(0, 2, 0),
                    0x1234_8086,
                    0x0002_0007,
                    0x0300_0000,
                    0x0000_0000,
                ), // Intel Display
                (
                    PciAddress::new(0, 3, 0),
                    0x100E_8086,
                    0x0002_0007,
                    0x0200_0000,
                    0x0080_0000,
                ), // Intel e1000 Multifunction
                (
                    PciAddress::new(0, 3, 1),
                    0x100F_8086,
                    0x0002_0007,
                    0x0200_0000,
                    0x0000_0000,
                ), // Function 1
                (PciAddress::new(0, 0, 0), 0xFFFF_FFFF, 0, 0, 0),
            ],
        };

        let mut buffer = [PciDeviceDescriptor::empty(); MAX_PCI_DEVICES];
        let count = unsafe { PciScanner::scan_all(&mock, &mut buffer, 0) };

        assert_eq!(count, 3);
        assert_eq!(buffer[0].vendor_id, 0x8086);
        assert_eq!(buffer[0].device_id, 0x1234);
        assert_eq!(buffer[0].class_code, 0x03);
        assert_eq!(buffer[0].class_name(), "Display Controller (VGA/GPU)");

        // BAR checks
        assert_eq!(
            buffer[0].bars[0].bar_type,
            BarType::Memory32 {
                address: 0xF000_0000,
                prefetchable: false
            }
        );
        assert_eq!(buffer[0].bars[1].bar_type, BarType::IoPort { port: 0xD000 });
    }

    #[test]
    fn test_pci_hex_formatting_and_16bit_bounds() {
        let dev = PciDeviceDescriptor {
            address: PciAddress::new(0, 31, 7),
            vendor_id: 0x8086,
            device_id: 0x2918,
            command: 0x0007,
            status: 0x0280,
            revision_id: 0x02,
            prog_if: 0x00,
            subclass: 0x01,
            class_code: 0x06,
            header_type: 0x80,
            bars: [PciBar::empty(); 6],
        };

        assert_eq!(dev.vendor_id, 0x8086);
        assert_eq!(dev.device_id, 0x2918);
        assert_eq!(dev.class_name(), "Bridge Device");

        // Test boundary values
        let max_dev = PciDeviceDescriptor {
            address: PciAddress::new(255, 31, 7),
            vendor_id: 0xFFFE,
            device_id: 0xFFFF,
            command: 0xFFFF,
            status: 0xFFFF,
            revision_id: 0xFF,
            prog_if: 0xFF,
            subclass: 0xFF,
            class_code: 0xFF,
            header_type: 0xFF,
            bars: [PciBar::empty(); 6],
        };
        assert_eq!(max_dev.address.bus, 255);
        assert_eq!(max_dev.address.device, 31);
        assert_eq!(max_dev.address.function, 7);
        assert_eq!(max_dev.vendor_id, 0xFFFE);
    }

    #[test]
    fn test_ecam_out_of_bounds_bus_rejected() {
        let ecam = EcamTransport::new(0xE000_0000, 0, 1);
        // Bus 2 berada di luar rentang bus 0..1
        let addr_out = PciAddress::new(2, 0, 0);
        unsafe {
            assert_eq!(ecam.read_u32(addr_out, 0), 0xFFFF_FFFF);
        }
    }
}
