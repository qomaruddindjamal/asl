//! Software RAID 0 (Striping) & RAID 1 (Mirroring) Block Device Subsystem
//! =========================================================================
//! DO-178C DAL-A Deterministic Software RAID Engine:
//!
//! 1. RAID 0 (Block Striping):
//!    - Requires minimum 2 member disks.
//!    - Stripe size configurable in sectors (must be a power-of-two, non-zero).
//!    - Total volume capacity: (minimum member sectors / stripe_size) * stripe_size * num_members.
//!    - Logical LBA to member LBA mapping with boundary crossing chunk splitting.
//!    - Zero redundancy: Failure on any member fails the operation immediately.
//!
//! 2. RAID 1 (Block Mirroring):
//!    - Requires minimum 2 member disks.
//!    - Writes data across all healthy mirror members.
//!    - Reads from the first healthy member (active read redundancy).
//!    - Member health tracking: Healthy vs Degraded/Failed.
//!    - Strict Consistency Policy: Partial write failure marks the failed member as Failed,
//!      propagates the error, and does NOT report false mirror success.
//!    - Reads and writes continue in explicit Degraded mode while at least 1 member is healthy.
//!    - If all members fail, operations return `StorageError::DeviceNotFound`.
//!    - In-memory runtime state: No persistent metadata on disk is assumed or fabricated.
//!      Auto-rebuild and crash recovery are explicitly deferred and documented.

use crate::block::{BlockDevice, StorageError, StorageResult, SECTOR_SIZE};

pub const MAX_RAID_MEMBERS: usize = 4;
pub const DEFAULT_STRIPE_SECTORS: u32 = 16; // 16 * 512B = 8KB stripe chunk

// =========================================================================
// RAID 0 (Striping)
// =========================================================================

/// Software RAID 0 (Striping) volume controller
pub struct Raid0Volume<'a> {
    pub members: [&'a mut dyn BlockDevice; MAX_RAID_MEMBERS],
    pub member_count: usize,
    pub stripe_sectors: u32,
    pub total_capacity_sectors: u64,
    pub member_usable_sectors: u64,
}

impl<'a> Raid0Volume<'a> {
    /// Create a new RAID 0 striping volume from an array of members and stripe size (in sectors).
    ///
    /// Validation:
    /// - `member_count` must be >= 2 and <= MAX_RAID_MEMBERS.
    /// - `stripe_sectors` must be > 0 and a power of two.
    /// - Each member must have at least `stripe_sectors` capacity.
    pub fn new(
        members: [&'a mut dyn BlockDevice; MAX_RAID_MEMBERS],
        member_count: usize,
        stripe_sectors: u32,
    ) -> StorageResult<Self> {
        if member_count < 2 || member_count > MAX_RAID_MEMBERS {
            return Err(StorageError::InvalidParameter);
        }
        if stripe_sectors == 0 || !stripe_sectors.is_power_of_two() {
            return Err(StorageError::InvalidParameter);
        }

        // Determine minimum capacity across all members
        let mut min_member_sectors = u64::MAX;
        for i in 0..member_count {
            let cap = members[i].total_sectors();
            if cap < stripe_sectors as u64 {
                return Err(StorageError::InvalidParameter);
            }
            if cap < min_member_sectors {
                min_member_sectors = cap;
            }
        }

        // Align usable sectors per member to full stripe boundaries
        let stripes_per_member = min_member_sectors / (stripe_sectors as u64);
        if stripes_per_member == 0 {
            return Err(StorageError::InvalidParameter);
        }
        let member_usable_sectors = stripes_per_member * (stripe_sectors as u64);
        let total_capacity_sectors = member_usable_sectors
            .checked_mul(member_count as u64)
            .ok_or(StorageError::InvalidParameter)?;

        Ok(Self {
            members,
            member_count,
            stripe_sectors,
            total_capacity_sectors,
            member_usable_sectors,
        })
    }

    /// Number of active member disks
    pub fn member_count(&self) -> usize {
        self.member_count
    }

    /// Configured stripe size in sectors
    pub fn stripe_sectors(&self) -> u32 {
        self.stripe_sectors
    }

    /// Usable sector count per member disk
    pub fn member_usable_sectors(&self) -> u64 {
        self.member_usable_sectors
    }
}

impl<'a> BlockDevice for Raid0Volume<'a> {
    fn total_sectors(&self) -> u64 {
        self.total_capacity_sectors
    }

    fn read_sectors(&mut self, start_lba: u64, sectors: u32, buf: &mut [u8]) -> StorageResult<()> {
        if sectors == 0 {
            return Ok(());
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
        if end_lba > self.total_capacity_sectors {
            return Err(StorageError::InvalidLba);
        }

        let stripe_u64 = self.stripe_sectors as u64;
        let mut current_lba = start_lba;
        let mut sectors_remaining = sectors;
        let mut buf_offset = 0usize;

        while sectors_remaining > 0 {
            // Stripe chunk math:
            // global_stripe_index = current_lba / stripe_size
            // disk_index = global_stripe_index % member_count
            // member_stripe_index = global_stripe_index / member_count
            // in_stripe_offset = current_lba % stripe_size
            let global_stripe = current_lba / stripe_u64;
            let disk_idx = (global_stripe % (self.member_count as u64)) as usize;
            let member_stripe = global_stripe / (self.member_count as u64);
            let in_stripe_offset = current_lba % stripe_u64;

            let sectors_avail_in_stripe = (stripe_u64 - in_stripe_offset) as u32;
            let chunk_sectors = core::cmp::min(sectors_remaining, sectors_avail_in_stripe);
            let chunk_bytes = (chunk_sectors as usize) * SECTOR_SIZE;

            let member_lba = member_stripe
                .checked_mul(stripe_u64)
                .and_then(|base| base.checked_add(in_stripe_offset))
                .ok_or(StorageError::InvalidLba)?;

            self.members[disk_idx].read_sectors(
                member_lba,
                chunk_sectors,
                &mut buf[buf_offset..buf_offset + chunk_bytes],
            )?;

            current_lba += chunk_sectors as u64;
            buf_offset += chunk_bytes;
            sectors_remaining -= chunk_sectors;
        }

        Ok(())
    }

    fn write_sectors(&mut self, start_lba: u64, sectors: u32, buf: &[u8]) -> StorageResult<()> {
        if sectors == 0 {
            return Ok(());
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
        if end_lba > self.total_capacity_sectors {
            return Err(StorageError::InvalidLba);
        }

        let stripe_u64 = self.stripe_sectors as u64;
        let mut current_lba = start_lba;
        let mut sectors_remaining = sectors;
        let mut buf_offset = 0usize;

        while sectors_remaining > 0 {
            let global_stripe = current_lba / stripe_u64;
            let disk_idx = (global_stripe % (self.member_count as u64)) as usize;
            let member_stripe = global_stripe / (self.member_count as u64);
            let in_stripe_offset = current_lba % stripe_u64;

            let sectors_avail_in_stripe = (stripe_u64 - in_stripe_offset) as u32;
            let chunk_sectors = core::cmp::min(sectors_remaining, sectors_avail_in_stripe);
            let chunk_bytes = (chunk_sectors as usize) * SECTOR_SIZE;

            let member_lba = member_stripe
                .checked_mul(stripe_u64)
                .and_then(|base| base.checked_add(in_stripe_offset))
                .ok_or(StorageError::InvalidLba)?;

            self.members[disk_idx].write_sectors(
                member_lba,
                chunk_sectors,
                &buf[buf_offset..buf_offset + chunk_bytes],
            )?;

            current_lba += chunk_sectors as u64;
            buf_offset += chunk_bytes;
            sectors_remaining -= chunk_sectors;
        }

        Ok(())
    }

    fn flush(&mut self) -> StorageResult<()> {
        for i in 0..self.member_count {
            self.members[i].flush()?;
        }
        Ok(())
    }
}

// =========================================================================
// RAID 1 (Mirroring)
// =========================================================================

/// Health state of a RAID 1 member disk
#[derive(Debug, Copy, Clone, PartialEq, Eq)]
pub enum Raid1MemberStatus {
    Healthy,
    Failed,
}

/// Overall volume health state
#[derive(Debug, Copy, Clone, PartialEq, Eq)]
pub enum Raid1VolumeState {
    Optimal,
    Degraded,
    Offline,
}

/// Software RAID 1 (Mirroring) volume controller
pub struct Raid1Volume<'a> {
    pub members: [&'a mut dyn BlockDevice; MAX_RAID_MEMBERS],
    pub status: [Raid1MemberStatus; MAX_RAID_MEMBERS],
    pub member_count: usize,
    pub capacity_sectors: u64,
}

impl<'a> Raid1Volume<'a> {
    /// Create a new RAID 1 mirroring volume from members.
    ///
    /// Validation:
    /// - `member_count` must be >= 2 and <= MAX_RAID_MEMBERS.
    /// - Capacity is the minimum capacity among all members.
    pub fn new(
        members: [&'a mut dyn BlockDevice; MAX_RAID_MEMBERS],
        member_count: usize,
    ) -> StorageResult<Self> {
        if member_count < 2 || member_count > MAX_RAID_MEMBERS {
            return Err(StorageError::InvalidParameter);
        }

        let mut min_sectors = u64::MAX;
        for i in 0..member_count {
            let cap = members[i].total_sectors();
            if cap == 0 {
                return Err(StorageError::InvalidParameter);
            }
            if cap < min_sectors {
                min_sectors = cap;
            }
        }

        let mut status = [Raid1MemberStatus::Failed; MAX_RAID_MEMBERS];
        for i in 0..member_count {
            status[i] = Raid1MemberStatus::Healthy;
        }

        Ok(Self {
            members,
            status,
            member_count,
            capacity_sectors: min_sectors,
        })
    }

    /// Number of member disks in mirror set
    pub fn member_count(&self) -> usize {
        self.member_count
    }

    /// Overall volume health status
    pub fn volume_state(&self) -> Raid1VolumeState {
        let mut healthy_count = 0;
        for i in 0..self.member_count {
            if self.status[i] == Raid1MemberStatus::Healthy {
                healthy_count += 1;
            }
        }

        if healthy_count == self.member_count {
            Raid1VolumeState::Optimal
        } else if healthy_count > 0 {
            Raid1VolumeState::Degraded
        } else {
            Raid1VolumeState::Offline
        }
    }

    /// Status of a specific member index
    pub fn member_status(&self, index: usize) -> Option<Raid1MemberStatus> {
        if index < self.member_count {
            Some(self.status[index])
        } else {
            None
        }
    }

    /// Mark a member disk as failed (manual or fault-injection)
    pub fn mark_member_failed(&mut self, index: usize) -> StorageResult<()> {
        if index >= self.member_count {
            return Err(StorageError::InvalidParameter);
        }
        self.status[index] = Raid1MemberStatus::Failed;
        Ok(())
    }
}

impl<'a> BlockDevice for Raid1Volume<'a> {
    fn total_sectors(&self) -> u64 {
        self.capacity_sectors
    }

    fn read_sectors(&mut self, start_lba: u64, sectors: u32, buf: &mut [u8]) -> StorageResult<()> {
        if sectors == 0 {
            return Ok(());
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
        if end_lba > self.capacity_sectors {
            return Err(StorageError::InvalidLba);
        }

        // Read redundancy policy:
        // Try reading from the first healthy member.
        // If a healthy member fails I/O during read, mark it Failed and fallback
        // to the next available healthy member.
        let mut last_err = StorageError::DeviceNotFound;
        for i in 0..self.member_count {
            if self.status[i] == Raid1MemberStatus::Healthy {
                match self.members[i].read_sectors(start_lba, sectors, buf) {
                    Ok(()) => return Ok(()),
                    Err(e) => {
                        self.status[i] = Raid1MemberStatus::Failed;
                        last_err = e;
                    }
                }
            }
        }

        Err(last_err)
    }

    fn write_sectors(&mut self, start_lba: u64, sectors: u32, buf: &[u8]) -> StorageResult<()> {
        if sectors == 0 {
            return Ok(());
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
        if end_lba > self.capacity_sectors {
            return Err(StorageError::InvalidLba);
        }

        // Consistency Policy:
        // Attempt to write to all currently healthy members.
        // If any member fails write, mark that member as Failed and record error.
        // If ANY member fails during an attempted write, propagate the error fail-closed!
        // We do NOT return Ok(()) on partial write success so callers know redundancy broke.
        let mut any_healthy = false;
        let mut first_err: Option<StorageError> = None;

        for i in 0..self.member_count {
            if self.status[i] == Raid1MemberStatus::Healthy {
                any_healthy = true;
                if let Err(e) = self.members[i].write_sectors(start_lba, sectors, buf) {
                    self.status[i] = Raid1MemberStatus::Failed;
                    if first_err.is_none() {
                        first_err = Some(e);
                    }
                }
            }
        }

        if !any_healthy {
            return Err(StorageError::DeviceNotFound);
        }

        if let Some(err) = first_err {
            return Err(err);
        }

        Ok(())
    }

    fn flush(&mut self) -> StorageResult<()> {
        let mut any_healthy = false;
        let mut first_err: Option<StorageError> = None;

        for i in 0..self.member_count {
            if self.status[i] == Raid1MemberStatus::Healthy {
                any_healthy = true;
                if let Err(e) = self.members[i].flush() {
                    self.status[i] = Raid1MemberStatus::Failed;
                    if first_err.is_none() {
                        first_err = Some(e);
                    }
                }
            }
        }

        if !any_healthy {
            return Err(StorageError::DeviceNotFound);
        }

        if let Some(err) = first_err {
            return Err(err);
        }

        Ok(())
    }
}
