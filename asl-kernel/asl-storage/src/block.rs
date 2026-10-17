//! Bounded Block Device Abstraction & Storage Errors

pub const SECTOR_SIZE: usize = 512;

/// Standard Storage Error Codes (Fail-Safe, Deterministic)
#[derive(Debug, Copy, Clone, PartialEq, Eq)]
pub enum StorageError {
    DeviceNotFound,
    Timeout,
    IoError,
    InvalidLba,
    BufferTooSmall,
    WriteProtected,
    DeviceBusy,
    UnsupportedFeature,
    CorruptedMetadata,
    NotMounted,
    NoSuchEntry,
    AlreadyExists,
    IsADirectory,
    NotADirectory,
    DirectoryNotEmpty,
    NoSpaceLeft,
    InvalidParameter,
}

pub type StorageResult<T> = Result<T, StorageError>;

/// Core Block Device Interface (Sector-level reading and writing)
pub trait BlockDevice {
    /// Total number of 512-byte sectors on the device
    fn total_sectors(&self) -> u64;

    /// Capacity in bytes (safe checked calculation)
    fn capacity_bytes(&self) -> u64 {
        self.total_sectors().saturating_mul(SECTOR_SIZE as u64)
    }

    /// Read consecutive sectors starting at LBA into buffer.
    /// Buffer length MUST be a non-zero multiple of SECTOR_SIZE (512).
    fn read_sectors(&mut self, start_lba: u64, sectors: u32, buf: &mut [u8]) -> StorageResult<()>;

    /// Write consecutive sectors starting at LBA from buffer.
    /// Buffer length MUST be a non-zero multiple of SECTOR_SIZE (512).
    fn write_sectors(&mut self, start_lba: u64, sectors: u32, buf: &[u8]) -> StorageResult<()>;

    /// Flush any volatile write caches
    fn flush(&mut self) -> StorageResult<()> {
        Ok(())
    }

    /// Fault injection hook for simulation and testability
    fn set_inject_error(&mut self, _err: bool) {}
}
