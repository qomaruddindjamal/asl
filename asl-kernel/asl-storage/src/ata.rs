//! ATA PIO Hard Disk Driver (Standard x86 IDE Port I/O with Timeouts & LBA28/48)

#[cfg(target_arch = "x86_64")]
use crate::block::SECTOR_SIZE;
use crate::block::{BlockDevice, StorageError, StorageResult};

pub const ATA_PRIMARY_IO_BASE: u16 = 0x1F0;
pub const ATA_PRIMARY_CTRL_BASE: u16 = 0x3F6;
pub const ATA_SECONDARY_IO_BASE: u16 = 0x170;
pub const ATA_SECONDARY_CTRL_BASE: u16 = 0x376;

// ATA Registers (Offsets from IO base)
pub const ATA_REG_DATA: u16 = 0;
pub const ATA_REG_ERROR_OR_FEATURES: u16 = 1;
pub const ATA_REG_SECTOR_COUNT: u16 = 2;
pub const ATA_REG_LBA_LO: u16 = 3;
pub const ATA_REG_LBA_MID: u16 = 4;
pub const ATA_REG_LBA_HI: u16 = 5;
pub const ATA_REG_DRIVE_HEAD: u16 = 6;
pub const ATA_REG_STATUS_OR_COMMAND: u16 = 7;

// ATA Commands
pub const ATA_CMD_READ_PIO: u8 = 0x20;
pub const ATA_CMD_WRITE_PIO: u8 = 0x30;
pub const ATA_CMD_IDENTIFY: u8 = 0xEC;
pub const ATA_CMD_CACHE_FLUSH: u8 = 0xE7;

// Status Flags
pub const ATA_SR_BSY: u8 = 0x80; // Busy
pub const ATA_SR_DRDY: u8 = 0x40; // Drive Ready
pub const ATA_SR_DF: u8 = 0x20; // Drive Write Fault
pub const ATA_SR_DSC: u8 = 0x10; // Drive Seek Complete
pub const ATA_SR_DRQ: u8 = 0x08; // Data Request Ready
pub const ATA_SR_CORR: u8 = 0x04; // Corrected Data
pub const ATA_SR_IDX: u8 = 0x02; // Index
pub const ATA_SR_ERR: u8 = 0x01; // Error

#[derive(Debug, Copy, Clone, PartialEq, Eq)]
pub enum AtaChannel {
    Primary,
    Secondary,
}

#[derive(Debug, Copy, Clone, PartialEq, Eq)]
pub enum AtaDeviceType {
    None,
    Pata,
    Sata,
    Papi,
    Unknown,
}

#[inline(always)]
unsafe fn inb(port: u16) -> u8 {
    let ret: u8;
    #[cfg(target_arch = "x86_64")]
    core::arch::asm!("in al, dx", out("al") ret, in("dx") port, options(nomem, nostack, preserves_flags));
    #[cfg(not(target_arch = "x86_64"))]
    {
        let _ = port;
        ret = 0;
    }
    ret
}

#[inline(always)]
unsafe fn outb(port: u16, val: u8) {
    #[cfg(target_arch = "x86_64")]
    core::arch::asm!("out dx, al", in("dx") port, in("al") val, options(nomem, nostack, preserves_flags));
    #[cfg(not(target_arch = "x86_64"))]
    {
        let _ = (port, val);
    }
}

#[cfg(target_arch = "x86_64")]
#[inline(always)]
unsafe fn inw(port: u16) -> u16 {
    let ret: u16;
    core::arch::asm!("in ax, dx", out("ax") ret, in("dx") port, options(nomem, nostack, preserves_flags));
    ret
}

#[cfg(target_arch = "x86_64")]
#[inline(always)]
unsafe fn outw(port: u16, val: u16) {
    core::arch::asm!("out dx, ax", in("dx") port, in("ax") val, options(nomem, nostack, preserves_flags));
}

pub struct AtaPioDriver {
    pub io_base: u16,
    pub ctrl_base: u16,
    pub is_slave: bool,
    pub device_type: AtaDeviceType,
    pub sectors_total: u64,
}

impl AtaPioDriver {
    pub const fn new(channel: AtaChannel, is_slave: bool) -> Self {
        let (io_base, ctrl_base) = match channel {
            AtaChannel::Primary => (ATA_PRIMARY_IO_BASE, ATA_PRIMARY_CTRL_BASE),
            AtaChannel::Secondary => (ATA_SECONDARY_IO_BASE, ATA_SECONDARY_CTRL_BASE),
        };
        Self {
            io_base,
            ctrl_base,
            is_slave,
            device_type: AtaDeviceType::None,
            sectors_total: 0,
        }
    }

    /// Read an ATA port register
    pub fn read_reg(&self, reg: u16) -> u8 {
        unsafe { inb(self.io_base + reg) }
    }

    /// Write an ATA port register
    pub fn write_reg(&self, reg: u16, val: u8) {
        unsafe { outb(self.io_base + reg, val) }
    }

    /// 400ns Delay through 4 status register reads
    pub fn io_delay(&self) {
        for _ in 0..4 {
            let _ = unsafe { inb(self.ctrl_base) };
        }
    }

    /// Wait until drive is not busy, with bounded timeout
    pub fn wait_not_busy(&self) -> StorageResult<u8> {
        let mut timeout: u32 = 100_000;
        while timeout > 0 {
            let status = self.read_reg(ATA_REG_STATUS_OR_COMMAND);
            if (status & ATA_SR_BSY) == 0 {
                return Ok(status);
            }
            core::hint::spin_loop();
            timeout -= 1;
        }
        Err(StorageError::Timeout)
    }

    /// Wait until data request is ready (DRQ = 1)
    pub fn wait_drq(&self) -> StorageResult<()> {
        let mut timeout: u32 = 100_000;
        while timeout > 0 {
            let status = self.read_reg(ATA_REG_STATUS_OR_COMMAND);
            if (status & ATA_SR_ERR) != 0 {
                return Err(StorageError::IoError);
            }
            if (status & ATA_SR_DRQ) != 0 {
                return Ok(());
            }
            core::hint::spin_loop();
            timeout -= 1;
        }
        Err(StorageError::Timeout)
    }

    /// Query and identify the drive via ATA_CMD_IDENTIFY
    pub fn identify(&mut self) -> StorageResult<AtaDeviceType> {
        #[cfg(not(target_arch = "x86_64"))]
        {
            return Err(StorageError::UnsupportedFeature);
        }

        #[cfg(target_arch = "x86_64")]
        {
            // Select drive
            let drive_sel = if self.is_slave { 0xB0 } else { 0xA0 };
            self.write_reg(ATA_REG_DRIVE_HEAD, drive_sel);
            self.io_delay();

            // Clear sector count & LBA registers
            self.write_reg(ATA_REG_SECTOR_COUNT, 0);
            self.write_reg(ATA_REG_LBA_LO, 0);
            self.write_reg(ATA_REG_LBA_MID, 0);
            self.write_reg(ATA_REG_LBA_HI, 0);

            // Send IDENTIFY command
            self.write_reg(ATA_REG_STATUS_OR_COMMAND, ATA_CMD_IDENTIFY);
            self.io_delay();

            let status = self.read_reg(ATA_REG_STATUS_OR_COMMAND);
            if status == 0 || status == 0xFF {
                self.device_type = AtaDeviceType::None;
                return Ok(AtaDeviceType::None);
            }

            // Wait for BSY to clear
            let status = self.wait_not_busy()?;
            if (status & ATA_SR_ERR) != 0 {
                // Check mid/hi registers to detect ATAPI/SATA
                let mid = self.read_reg(ATA_REG_LBA_MID);
                let hi = self.read_reg(ATA_REG_LBA_HI);
                if (mid == 0x14 && hi == 0xEB) || (mid == 0x69 && hi == 0x96) {
                    self.device_type = AtaDeviceType::Papi;
                    return Ok(AtaDeviceType::Papi);
                } else if mid == 0x3C && hi == 0xC3 {
                    self.device_type = AtaDeviceType::Sata;
                    return Ok(AtaDeviceType::Sata);
                }
                self.device_type = AtaDeviceType::None;
                return Ok(AtaDeviceType::None);
            }

            self.wait_drq()?;

            // Read 256 16-bit words of IDENTIFY data
            let mut words = [0u16; 256];
            for w in 0..256 {
                words[w] = unsafe { inw(self.io_base + ATA_REG_DATA) };
            }

            // Extract LBA28 or LBA48 sector count
            let lba48_supported = (words[83] & (1 << 10)) != 0;
            let total_sectors = if lba48_supported {
                (words[100] as u64)
                    | ((words[101] as u64) << 16)
                    | ((words[102] as u64) << 32)
                    | ((words[103] as u64) << 48)
            } else {
                (words[60] as u64) | ((words[61] as u64) << 16)
            };

            self.sectors_total = total_sectors;
            self.device_type = AtaDeviceType::Pata;
            Ok(AtaDeviceType::Pata)
        }
    }
}

impl BlockDevice for AtaPioDriver {
    fn total_sectors(&self) -> u64 {
        self.sectors_total
    }

    fn read_sectors(&mut self, start_lba: u64, sectors: u32, buf: &mut [u8]) -> StorageResult<()> {
        #[cfg(not(target_arch = "x86_64"))]
        {
            let _ = (start_lba, sectors, buf);
            return Err(StorageError::UnsupportedFeature);
        }

        #[cfg(target_arch = "x86_64")]
        {
            let needed = (sectors as usize)
                .checked_mul(SECTOR_SIZE)
                .ok_or(StorageError::InvalidParameter)?;
            if buf.len() < needed {
                return Err(StorageError::BufferTooSmall);
            }
            if start_lba >= self.sectors_total
                || start_lba.saturating_add(sectors as u64) > self.sectors_total
            {
                return Err(StorageError::InvalidLba);
            }

            for s in 0..sectors {
                let lba = start_lba + s as u64;
                self.wait_not_busy()?;

                // LBA28 Drive / Head Register
                let drive_head =
                    0xE0 | (if self.is_slave { 0x10 } else { 0x00 }) | (((lba >> 24) & 0x0F) as u8);
                self.write_reg(ATA_REG_DRIVE_HEAD, drive_head);
                self.write_reg(ATA_REG_SECTOR_COUNT, 1);
                self.write_reg(ATA_REG_LBA_LO, (lba & 0xFF) as u8);
                self.write_reg(ATA_REG_LBA_MID, ((lba >> 8) & 0xFF) as u8);
                self.write_reg(ATA_REG_LBA_HI, ((lba >> 16) & 0xFF) as u8);
                self.write_reg(ATA_REG_STATUS_OR_COMMAND, ATA_CMD_READ_PIO);

                self.wait_drq()?;

                let offset = (s as usize) * SECTOR_SIZE;
                for w in 0..256 {
                    let word = unsafe { inw(self.io_base + ATA_REG_DATA) };
                    let bytes = word.to_le_bytes();
                    buf[offset + w * 2] = bytes[0];
                    buf[offset + w * 2 + 1] = bytes[1];
                }
            }
            Ok(())
        }
    }

    fn write_sectors(&mut self, start_lba: u64, sectors: u32, buf: &[u8]) -> StorageResult<()> {
        #[cfg(not(target_arch = "x86_64"))]
        {
            let _ = (start_lba, sectors, buf);
            return Err(StorageError::UnsupportedFeature);
        }

        #[cfg(target_arch = "x86_64")]
        {
            let needed = (sectors as usize)
                .checked_mul(SECTOR_SIZE)
                .ok_or(StorageError::InvalidParameter)?;
            if buf.len() < needed {
                return Err(StorageError::BufferTooSmall);
            }
            if start_lba >= self.sectors_total
                || start_lba.saturating_add(sectors as u64) > self.sectors_total
            {
                return Err(StorageError::InvalidLba);
            }

            for s in 0..sectors {
                let lba = start_lba + s as u64;
                self.wait_not_busy()?;

                let drive_head =
                    0xE0 | (if self.is_slave { 0x10 } else { 0x00 }) | (((lba >> 24) & 0x0F) as u8);
                self.write_reg(ATA_REG_DRIVE_HEAD, drive_head);
                self.write_reg(ATA_REG_SECTOR_COUNT, 1);
                self.write_reg(ATA_REG_LBA_LO, (lba & 0xFF) as u8);
                self.write_reg(ATA_REG_LBA_MID, ((lba >> 8) & 0xFF) as u8);
                self.write_reg(ATA_REG_LBA_HI, ((lba >> 16) & 0xFF) as u8);
                self.write_reg(ATA_REG_STATUS_OR_COMMAND, ATA_CMD_WRITE_PIO);

                self.wait_drq()?;

                let offset = (s as usize) * SECTOR_SIZE;
                for w in 0..256 {
                    let word = u16::from_le_bytes([buf[offset + w * 2], buf[offset + w * 2 + 1]]);
                    unsafe { outw(self.io_base + ATA_REG_DATA, word) };
                }
            }

            // Flush write cache
            self.write_reg(ATA_REG_STATUS_OR_COMMAND, ATA_CMD_CACHE_FLUSH);
            self.wait_not_busy()?;
            Ok(())
        }
    }
}
