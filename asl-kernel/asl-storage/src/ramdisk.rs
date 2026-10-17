//! In-Memory Deterministic Block Device for unit, property, and regression testing

use crate::block::{BlockDevice, StorageError, StorageResult, SECTOR_SIZE};

pub const DEFAULT_RAMDISK_SECTORS: usize = 128;
pub const DEFAULT_RAMDISK_BYTES: usize = DEFAULT_RAMDISK_SECTORS * SECTOR_SIZE;

pub struct MemBlockDevice<const TOTAL_BYTES: usize = DEFAULT_RAMDISK_BYTES> {
    pub storage: [u8; TOTAL_BYTES],
    pub read_only: bool,
    pub inject_error: bool,
}

impl<const TOTAL_BYTES: usize> MemBlockDevice<TOTAL_BYTES> {
    pub const fn new() -> Self {
        Self {
            storage: [0u8; TOTAL_BYTES],
            read_only: false,
            inject_error: false,
        }
    }

    pub fn set_read_only(&mut self, ro: bool) {
        self.read_only = ro;
    }

    pub fn set_inject_error(&mut self, err: bool) {
        self.inject_error = err;
    }
}

impl<const TOTAL_BYTES: usize> BlockDevice for MemBlockDevice<TOTAL_BYTES> {
    fn total_sectors(&self) -> u64 {
        (TOTAL_BYTES / SECTOR_SIZE) as u64
    }

    fn read_sectors(&mut self, start_lba: u64, sectors: u32, buf: &mut [u8]) -> StorageResult<()> {
        if sectors == 0 {
            return Ok(());
        }
        if self.inject_error {
            return Err(StorageError::IoError);
        }

        let needed_bytes = (sectors as usize)
            .checked_mul(SECTOR_SIZE)
            .ok_or(StorageError::InvalidParameter)?;
        if buf.len() < needed_bytes {
            return Err(StorageError::BufferTooSmall);
        }

        let end_lba = start_lba
            .checked_add(sectors as u64)
            .ok_or(StorageError::InvalidLba)?;
        if end_lba > self.total_sectors() {
            return Err(StorageError::InvalidLba);
        }

        let start_offset = (start_lba as usize) * SECTOR_SIZE;
        let end_offset = start_offset + needed_bytes;

        buf[..needed_bytes].copy_from_slice(&self.storage[start_offset..end_offset]);
        Ok(())
    }

    fn write_sectors(&mut self, start_lba: u64, sectors: u32, buf: &[u8]) -> StorageResult<()> {
        if sectors == 0 {
            return Ok(());
        }
        if self.read_only {
            return Err(StorageError::WriteProtected);
        }
        if self.inject_error {
            return Err(StorageError::IoError);
        }

        let needed_bytes = (sectors as usize)
            .checked_mul(SECTOR_SIZE)
            .ok_or(StorageError::InvalidParameter)?;
        if buf.len() < needed_bytes {
            return Err(StorageError::BufferTooSmall);
        }

        let end_lba = start_lba
            .checked_add(sectors as u64)
            .ok_or(StorageError::InvalidLba)?;
        if end_lba > self.total_sectors() {
            return Err(StorageError::InvalidLba);
        }

        let start_offset = (start_lba as usize) * SECTOR_SIZE;
        let end_offset = start_offset + needed_bytes;

        self.storage[start_offset..end_offset].copy_from_slice(&buf[..needed_bytes]);
        Ok(())
    }
}
