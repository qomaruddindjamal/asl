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
use crate::partition::crc32;

pub const MAX_RAID_MEMBERS: usize = 4;
pub const DEFAULT_STRIPE_SECTORS: u32 = 16; // 16 * 512B = 8KB stripe chunk

// =========================================================================
// ASL RAID Persistent Metadata / Superblock Wire Format
// =========================================================================
// Layout (512 bytes exact sector):
//  [00..08] Magic signature: b"ASL_RAID" (8 bytes)
//  [08..10] Format version: u16 (current: 1)
//  [10..12] Raid type: u16 (0 = RAID 0, 1 = RAID 1)
//  [12..16] Header size: u32 (always 512)
//  [16..20] Header CRC32: u32 (IEEE 802.3 over 512 bytes with offset 16..20 zeroed)
//  [20..28] Generation sequence: u64 (modular sequence counter)
//  [28..44] Array UUID: [u8; 16]
//  [44..60] Member UUID: [u8; 16]
//  [60..64] Member index: u32 (0..member_count-1)
//  [64..68] Member count: u32 (2..MAX_RAID_MEMBERS)
//  [68..72] Stripe sectors: u32 (power of two, non-zero for RAID 0, 0 for RAID 1)
//  [72..80] Array total capacity in sectors: u64
//  [80..88] Member usable capacity in sectors: u64
//  [88..96] Data area start LBA: u64
//  [96..98] Metadata state: u16 (1 = Active/Clean, 2 = Degraded)
//  [98..128] Reserved space: [u8; 30] (must be all 0x00)
//  [128..512] Reserved payload space: [u8; 384] (must be all 0x00)

pub const RAID_SUPERBLOCK_MAGIC: &[u8; 8] = b"ASL_RAID";
pub const RAID_SUPERBLOCK_VERSION_1: u16 = 1;
pub const RAID_SUPERBLOCK_WIRE_SIZE: usize = 512;
pub const RAID_RESERVED_SECTORS: u64 = 8; // Superblock located at LBA 0 of dedicated partition / reserved range

#[derive(Debug, Copy, Clone, PartialEq, Eq)]
#[repr(u16)]
pub enum RaidType {
    Raid0 = 0,
    Raid1 = 1,
}

impl RaidType {
    pub fn from_u16(val: u16) -> Option<Self> {
        match val {
            0 => Some(RaidType::Raid0),
            1 => Some(RaidType::Raid1),
            _ => None,
        }
    }
}

#[derive(Debug, Copy, Clone, PartialEq, Eq)]
#[repr(u16)]
pub enum RaidMetadataState {
    ActiveClean = 1,
    Degraded = 2,
}

impl RaidMetadataState {
    pub fn from_u16(val: u16) -> Option<Self> {
        match val {
            1 => Some(RaidMetadataState::ActiveClean),
            2 => Some(RaidMetadataState::Degraded),
            _ => None,
        }
    }
}

/// Generation counter comparison result accounting for wraparound
#[derive(Debug, Copy, Clone, PartialEq, Eq)]
pub enum GenerationOrder {
    Equal,
    Older,
    Newer,
    Ambiguous,
}

/// Decoded in-memory representation of persistent RAID Superblock
#[derive(Debug, Copy, Clone, PartialEq, Eq)]
pub struct RaidSuperblock {
    pub version: u16,
    pub raid_type: RaidType,
    pub generation: u64,
    pub array_uuid: [u8; 16],
    pub member_uuid: [u8; 16],
    pub member_index: u32,
    pub member_count: u32,
    pub stripe_sectors: u32,
    pub array_capacity_sectors: u64,
    pub member_usable_sectors: u64,
    pub data_start_lba: u64,
    pub state: RaidMetadataState,
}

impl RaidSuperblock {
    /// Compare two generation counters with modular half-space wraparound handling.
    /// Range window: u64::MAX / 2. If the absolute distance is exactly 2^63, the order is Ambiguous.
    pub fn compare_generation(gen_a: u64, gen_b: u64) -> GenerationOrder {
        if gen_a == gen_b {
            return GenerationOrder::Equal;
        }
        let diff = gen_a.wrapping_sub(gen_b);
        const HALF_WINDOW: u64 = 1u64 << 63;
        if diff == HALF_WINDOW {
            GenerationOrder::Ambiguous
        } else if diff < HALF_WINDOW {
            GenerationOrder::Newer
        } else {
            GenerationOrder::Older
        }
    }

    /// Encode this superblock into a standard 512-byte wire sector buffer
    pub fn encode(&self, out_buf: &mut [u8; SECTOR_SIZE]) -> StorageResult<()> {
        out_buf.fill(0);

        out_buf[0..8].copy_from_slice(RAID_SUPERBLOCK_MAGIC);
        out_buf[8..10].copy_from_slice(&self.version.to_le_bytes());
        out_buf[10..12].copy_from_slice(&(self.raid_type as u16).to_le_bytes());
        out_buf[12..16].copy_from_slice(&(RAID_SUPERBLOCK_WIRE_SIZE as u32).to_le_bytes());
        // 16..20: CRC field left zeroed during calculation
        out_buf[20..28].copy_from_slice(&self.generation.to_le_bytes());
        out_buf[28..44].copy_from_slice(&self.array_uuid);
        out_buf[44..60].copy_from_slice(&self.member_uuid);
        out_buf[60..64].copy_from_slice(&self.member_index.to_le_bytes());
        out_buf[64..68].copy_from_slice(&self.member_count.to_le_bytes());
        out_buf[68..72].copy_from_slice(&self.stripe_sectors.to_le_bytes());
        out_buf[72..80].copy_from_slice(&self.array_capacity_sectors.to_le_bytes());
        out_buf[80..88].copy_from_slice(&self.member_usable_sectors.to_le_bytes());
        out_buf[88..96].copy_from_slice(&self.data_start_lba.to_le_bytes());
        out_buf[96..98].copy_from_slice(&(self.state as u16).to_le_bytes());
        // 98..512: Reserved fields are strictly zero

        let computed_crc = crc32(out_buf);
        out_buf[16..20].copy_from_slice(&computed_crc.to_le_bytes());
        Ok(())
    }

    /// Decode and strictly validate a 512-byte wire sector buffer
    pub fn decode(buf: &[u8; SECTOR_SIZE]) -> StorageResult<Self> {
        // 1. Check Magic
        if &buf[0..8] != RAID_SUPERBLOCK_MAGIC {
            return Err(StorageError::CorruptedMetadata);
        }

        // 2. Validate CRC32
        let recorded_crc = u32::from_le_bytes([buf[16], buf[17], buf[18], buf[19]]);
        let mut crc_buf = *buf;
        crc_buf[16..20].fill(0);
        let computed_crc = crc32(&crc_buf);
        if computed_crc != recorded_crc {
            return Err(StorageError::CorruptedMetadata);
        }

        // 3. Check Header Size
        let header_size = u32::from_le_bytes([buf[12], buf[13], buf[14], buf[15]]) as usize;
        if header_size != RAID_SUPERBLOCK_WIRE_SIZE {
            return Err(StorageError::CorruptedMetadata);
        }

        // 4. Check Version Compatibility
        let version = u16::from_le_bytes([buf[8], buf[9]]);
        if version != RAID_SUPERBLOCK_VERSION_1 {
            return Err(StorageError::UnsupportedFeature);
        }

        // 5. Check Reserved fields must be zero (strict wire format validation)
        if buf[98..128].iter().any(|&b| b != 0) || buf[128..512].iter().any(|&b| b != 0) {
            return Err(StorageError::CorruptedMetadata);
        }

        // 6. Parse and validate Enums
        let raid_type_raw = u16::from_le_bytes([buf[10], buf[11]]);
        let raid_type = RaidType::from_u16(raid_type_raw).ok_or(StorageError::CorruptedMetadata)?;

        let state_raw = u16::from_le_bytes([buf[96], buf[97]]);
        let state =
            RaidMetadataState::from_u16(state_raw).ok_or(StorageError::CorruptedMetadata)?;

        // 7. Parse numeric parameters
        let generation = u64::from_le_bytes([
            buf[20], buf[21], buf[22], buf[23], buf[24], buf[25], buf[26], buf[27],
        ]);

        let mut array_uuid = [0u8; 16];
        array_uuid.copy_from_slice(&buf[28..44]);
        if array_uuid.iter().all(|&b| b == 0) {
            return Err(StorageError::CorruptedMetadata); // Disallow all-zero UUID
        }

        let mut member_uuid = [0u8; 16];
        member_uuid.copy_from_slice(&buf[44..60]);
        if member_uuid.iter().all(|&b| b == 0) {
            return Err(StorageError::CorruptedMetadata);
        }

        let member_index = u32::from_le_bytes([buf[60], buf[61], buf[62], buf[63]]);
        let member_count = u32::from_le_bytes([buf[64], buf[65], buf[66], buf[67]]);
        if member_count < 2 || member_count > (MAX_RAID_MEMBERS as u32) {
            return Err(StorageError::CorruptedMetadata);
        }
        if member_index >= member_count {
            return Err(StorageError::CorruptedMetadata);
        }

        let stripe_sectors = u32::from_le_bytes([buf[68], buf[69], buf[70], buf[71]]);
        if raid_type == RaidType::Raid0 {
            if stripe_sectors == 0 || !stripe_sectors.is_power_of_two() {
                return Err(StorageError::CorruptedMetadata);
            }
        }

        let array_capacity = u64::from_le_bytes([
            buf[72], buf[73], buf[74], buf[75], buf[76], buf[77], buf[78], buf[79],
        ]);
        let member_usable = u64::from_le_bytes([
            buf[80], buf[81], buf[82], buf[83], buf[84], buf[85], buf[86], buf[87],
        ]);
        let data_start_lba = u64::from_le_bytes([
            buf[88], buf[89], buf[90], buf[91], buf[92], buf[93], buf[94], buf[95],
        ]);

        if array_capacity == 0 || member_usable == 0 || data_start_lba == 0 {
            return Err(StorageError::CorruptedMetadata);
        }

        // Semantic capacity relation checks
        if raid_type == RaidType::Raid0 {
            let expected_capacity = member_usable
                .checked_mul(member_count as u64)
                .ok_or(StorageError::CorruptedMetadata)?;
            if array_capacity != expected_capacity {
                return Err(StorageError::CorruptedMetadata);
            }
        } else if raid_type == RaidType::Raid1 {
            if array_capacity != member_usable {
                return Err(StorageError::CorruptedMetadata);
            }
        }

        Ok(Self {
            version,
            raid_type,
            generation,
            array_uuid,
            member_uuid,
            member_index,
            member_count,
            stripe_sectors,
            array_capacity_sectors: array_capacity,
            member_usable_sectors: member_usable,
            data_start_lba,
            state,
        })
    }

    /// Match compatibility between two members:
    /// - Array UUID, version, raid_type, member_count, capacities, and data_start_lba MUST match exactly.
    /// - Member UUID and member_index MUST be distinct.
    pub fn validate_peer_compatibility(&self, peer: &Self) -> StorageResult<()> {
        if self.array_uuid != peer.array_uuid {
            return Err(StorageError::CorruptedMetadata);
        }
        if self.version != peer.version || self.raid_type != peer.raid_type {
            return Err(StorageError::CorruptedMetadata);
        }
        if self.member_count != peer.member_count {
            return Err(StorageError::CorruptedMetadata);
        }
        if self.stripe_sectors != peer.stripe_sectors {
            return Err(StorageError::CorruptedMetadata);
        }
        if self.array_capacity_sectors != peer.array_capacity_sectors
            || self.member_usable_sectors != peer.member_usable_sectors
            || self.data_start_lba != peer.data_start_lba
        {
            return Err(StorageError::CorruptedMetadata);
        }
        if self.member_index == peer.member_index || self.member_uuid == peer.member_uuid {
            return Err(StorageError::AlreadyExists); // Duplicate member index or UUID
        }
        Ok(())
    }
}

/// Explicit Reservation Contract for On-Disk Metadata Placement.
///
/// Under DO-178C DAL-A safety rules, writing RAID metadata blindly to LBA 0 of an raw
/// block device is strictly forbidden because LBA 0 contains MBR or GPT Protective MBR.
/// Furthermore, partition tables (GPT header at LBA 1, partition entries at LBA 2..33)
/// and filesystems must be protected against collision.
///
/// An explicit partition reservation proves:
/// 1. The target device is an isolated, dedicated partition or storage volume slice.
/// 2. Metadata is placed at an explicit, non-zero offset within the partition (`metadata_lba >= reservation_start_lba`).
/// 3. The reservation range does NOT overlap LBA 0 of the underlying storage medium.
/// 4. Capacity bounds are verified without 64-bit integer overflows.
#[derive(Debug, Copy, Clone, PartialEq, Eq)]
pub struct DedicatedPartitionReservation {
    /// Start LBA of the dedicated partition on the underlying device.
    /// MUST be >= 1 (strictly prohibiting collision with LBA 0).
    pub partition_start_lba: u64,
    /// Total sector count of the dedicated partition.
    pub partition_sector_count: u64,
    /// Relative sector offset within the partition dedicated for metadata.
    /// Superblock will reside at `partition_start_lba + metadata_offset_lba`.
    pub metadata_offset_lba: u64,
    /// Relative start offset within the partition where user data begins.
    pub data_offset_lba: u64,
}

impl DedicatedPartitionReservation {
    /// Validate and create a dedicated partition reservation contract from explicit parameters.
    ///
    /// Rules:
    /// - `partition_start_lba >= 1` (Never overlaps LBA 0 / MBR / protective MBR).
    /// - `partition_sector_count >= 2` (Must have room for at least 1 metadata sector + data).
    /// - `data_offset_lba > metadata_offset_lba` (Data area strictly separates from metadata sector).
    /// - `data_offset_lba < partition_sector_count` (Data area within reservation).
    /// - Arithmetic overflow checks on all bounds.
    pub fn new(
        partition_start_lba: u64,
        partition_sector_count: u64,
        metadata_offset_lba: u64,
        data_offset_lba: u64,
    ) -> StorageResult<Self> {
        // Rule 1: Fail closed if partition starts at LBA 0
        if partition_start_lba == 0 {
            return Err(StorageError::InvalidParameter);
        }

        // Rule 2: Minimum partition sector count must be at least 2 (metadata + data)
        if partition_sector_count < 2 {
            return Err(StorageError::InvalidParameter);
        }

        // Rule 3: Strict non-overlap between metadata sector (1 sector) and data area
        let metadata_end = metadata_offset_lba
            .checked_add(1)
            .ok_or(StorageError::InvalidParameter)?;
        if metadata_end > data_offset_lba {
            return Err(StorageError::InvalidParameter);
        }

        // Rule 4: Data area must not exceed partition boundaries
        if data_offset_lba >= partition_sector_count {
            return Err(StorageError::InvalidParameter);
        }

        // Rule 5: Ensure partition end LBA does not overflow u64
        partition_start_lba
            .checked_add(partition_sector_count)
            .ok_or(StorageError::InvalidParameter)?;

        Ok(Self {
            partition_start_lba,
            partition_sector_count,
            metadata_offset_lba,
            data_offset_lba,
        })
    }

    /// Construct a reservation from a verified `Partition` entry obtained from
    /// a validated partition table (MBR or GPT).
    ///
    /// Safety Contract:
    /// - Strictly requires `part.part_type == PartitionType::RaidMember` (proving the partition
    ///   has been explicitly tagged/allocated as a RAID volume member, e.g. MBR 0xFD or Linux RAID GUID).
    /// - Rejects general purpose data, FAT, NTFS, ext4, EFI, and Swap partitions to prevent
    ///   accidental overwrite of existing filesystems or system data.
    /// - Enforces `part.sector_count >= 2`.
    /// - The caller explicitly specifies metadata and data offsets within that verified partition.
    pub fn from_verified_partition(
        part: &crate::partition::Partition,
        metadata_offset_lba: u64,
        data_offset_lba: u64,
    ) -> StorageResult<Self> {
        // Strict partition provenance check: Only dedicated RAID partitions are authorized
        if part.part_type != crate::partition::PartitionType::RaidMember {
            return Err(StorageError::InvalidParameter);
        }

        Self::new(
            part.start_lba,
            part.sector_count,
            metadata_offset_lba,
            data_offset_lba,
        )
    }

    /// Calculate the absolute on-disk LBA for the metadata superblock.
    /// Guaranteed to be >= 1 and strictly within the partition bounds.
    pub fn absolute_metadata_lba(&self) -> StorageResult<u64> {
        let abs_lba = self
            .partition_start_lba
            .checked_add(self.metadata_offset_lba)
            .ok_or(StorageError::InvalidParameter)?;
        if abs_lba == 0 {
            return Err(StorageError::InvalidParameter);
        }
        let partition_end = self
            .partition_start_lba
            .checked_add(self.partition_sector_count)
            .ok_or(StorageError::InvalidParameter)?;
        if abs_lba >= partition_end {
            return Err(StorageError::InvalidParameter);
        }
        Ok(abs_lba)
    }

    /// Calculate the absolute on-disk LBA where member data starts.
    pub fn absolute_data_start_lba(&self) -> StorageResult<u64> {
        let abs_data = self
            .partition_start_lba
            .checked_add(self.data_offset_lba)
            .ok_or(StorageError::InvalidParameter)?;
        let partition_end = self
            .partition_start_lba
            .checked_add(self.partition_sector_count)
            .ok_or(StorageError::InvalidParameter)?;
        if abs_data > partition_end {
            return Err(StorageError::InvalidParameter);
        }
        Ok(abs_data)
    }

    /// Usable member sector count in this dedicated partition reservation.
    pub fn usable_member_sectors(&self) -> u64 {
        self.partition_sector_count
            .saturating_sub(self.data_offset_lba)
    }
}

/// Explicit Authorization Policy for RAID Member Initialization.
///
/// Under DO-178C DAL-A safety standards:
/// - Simply having `PartitionType::RaidMember` does NOT prove the partition is unformatted or empty.
/// - Writing a brand-new superblock to an existing member could destroy an active array or overwrite
///   prior metadata without an explicit administrative authorization policy.
/// Explicit Authorization Policy for RAID Member Initialization.
///
/// Under DO-178C DAL-A safety standards:
/// - Simply having `PartitionType::RaidMember` does NOT prove the partition is unformatted or empty.
/// - Writing a brand-new superblock to an existing member destroys an active array or corrupts
///   prior metadata unless verified by a real kernel security authorization framework.
/// - Because ASL Core Kernel has not yet integrated an active runtime partition ownership manager
///   or interactive administrative authorization capability subsystem, destructive re-initialization
///   of an already-formatted RAID member is strictly closed fail-closed (`Err(StorageError::WriteProtected)`).
#[derive(Debug, Copy, Clone, PartialEq, Eq)]
pub enum RaidInitializationPolicy {
    /// Authorize initializing a new array member only if no valid superblock currently exists.
    /// If a valid superblock is already present on the reserved sector, initialization fails closed
    /// (`Err(StorageError::AlreadyExists)`) to prevent accidental destruction.
    CreateNewArraySafe,
}

impl RaidSuperblock {
    /// Initialize a new RAID array member on disk under an explicit initialization policy.
    ///
    /// Safety rules:
    /// 1. If `policy == RaidInitializationPolicy::CreateNewArraySafe`, this method attempts to read
    ///    an existing superblock from the target LBA. If a valid superblock already exists,
    ///    it fails closed with `Err(StorageError::AlreadyExists)`.
    /// 2. If no valid superblock exists (corrupted, uninitialized, or blank), it proceeds to write.
    /// 3. Destructive reinitialization of an existing active array member is strictly disabled
    ///    and deferred until a real administrative capability subsystem is introduced.
    pub fn initialize_member<B: BlockDevice>(
        &self,
        dev: &mut B,
        reservation: &DedicatedPartitionReservation,
        policy: RaidInitializationPolicy,
    ) -> StorageResult<()> {
        match policy {
            RaidInitializationPolicy::CreateNewArraySafe => {
                // Check if a valid superblock already exists at the metadata LBA
                if Self::read_from_device(dev, reservation).is_ok() {
                    // Reject overwriting existing active RAID member metadata fail-closed!
                    return Err(StorageError::AlreadyExists);
                }
            }
        }

        self.write_to_device(dev, reservation)
    }

    /// Operational Update Protocol for an Existing Array Member:
    ///
    /// Under DO-178C DAL-A consistency requirements:
    /// 1. Reads the existing on-disk superblock and verifies array identity continuity:
    ///    - Array UUID, RAID type, member count, and member index MUST match.
    ///    - New generation counter MUST be strictly newer than existing generation (`GenerationOrder::Newer`).
    /// 2. Executes write and flush.
    /// 3. Performs strict read-back verification: reads back the superblock from disk, decodes it,
    ///    and verifies full equivalence with the updated superblock.
    /// 4. If any step fails (write, flush, or read-back verification), returns an error immediately.
    ///
    /// NOTE: ASL does NOT claim distributed cross-device multi-disk atomic commit transactions.
    /// Each member disk is updated individually with fail-safe read-back verification.
    pub fn update_operational_member<B: BlockDevice>(
        &self,
        dev: &mut B,
        reservation: &DedicatedPartitionReservation,
    ) -> StorageResult<()> {
        // Step 1: Read and verify existing superblock identity continuity
        let existing = Self::read_from_device(dev, reservation)?;
        if existing.array_uuid != self.array_uuid
            || existing.member_uuid != self.member_uuid
            || existing.member_index != self.member_index
            || existing.raid_type != self.raid_type
            || existing.member_count != self.member_count
        {
            return Err(StorageError::CorruptedMetadata);
        }

        // Enforce generation progression
        if Self::compare_generation(self.generation, existing.generation) != GenerationOrder::Newer
        {
            return Err(StorageError::InvalidParameter);
        }

        // Step 2: Write and flush
        self.write_to_device(dev, reservation)?;

        // Step 3: Strict Read-Back Verification
        let readback = Self::read_from_device(dev, reservation)?;
        if readback != *self {
            return Err(StorageError::CorruptedMetadata);
        }

        Ok(())
    }

    /// Safe On-Disk Write Protocol with Explicit Partition Reservation Contract:
    ///
    /// Strictly prevents writing metadata to LBA 0 (which may contain MBR/GPT) or
    /// unreserved disk regions.
    ///
    /// Requires an explicit `DedicatedPartitionReservation` contract that guarantees:
    /// - The metadata LBA is >= 1 (never LBA 0).
    /// - The device capacity satisfies the reservation.
    /// - The superblock's data start LBA and usable capacity match the reservation contract.
    pub fn write_to_device<B: BlockDevice>(
        &self,
        dev: &mut B,
        reservation: &DedicatedPartitionReservation,
    ) -> StorageResult<()> {
        let target_lba = reservation.absolute_metadata_lba()?;
        if target_lba == 0 {
            // Uncompromising fail-closed check
            return Err(StorageError::InvalidParameter);
        }

        // Validate device capacity against total reservation
        let part_end = reservation
            .partition_start_lba
            .checked_add(reservation.partition_sector_count)
            .ok_or(StorageError::InvalidParameter)?;
        if dev.total_sectors() < part_end {
            return Err(StorageError::InvalidParameter);
        }

        // Validate contract consistency with superblock parameters
        let expected_data_start = reservation.absolute_data_start_lba()?;
        if self.data_start_lba != expected_data_start {
            return Err(StorageError::InvalidParameter);
        }
        if self.member_usable_sectors > reservation.usable_member_sectors() {
            return Err(StorageError::InvalidParameter);
        }

        let mut sector_buf = [0u8; SECTOR_SIZE];
        self.encode(&mut sector_buf)?;

        // Write exclusively to the explicit reservation metadata LBA (never LBA 0)
        dev.write_sectors(target_lba, 1, &sector_buf)?;
        dev.flush()?;
        Ok(())
    }

    /// Safe On-Disk Read Protocol with Explicit Partition Reservation Contract:
    ///
    /// Reads and decodes the superblock from the explicit metadata LBA defined
    /// by `DedicatedPartitionReservation`.
    pub fn read_from_device<B: BlockDevice>(
        dev: &mut B,
        reservation: &DedicatedPartitionReservation,
    ) -> StorageResult<Self> {
        let target_lba = reservation.absolute_metadata_lba()?;
        if target_lba == 0 {
            return Err(StorageError::InvalidParameter);
        }

        let part_end = reservation
            .partition_start_lba
            .checked_add(reservation.partition_sector_count)
            .ok_or(StorageError::InvalidParameter)?;
        if dev.total_sectors() < part_end {
            return Err(StorageError::InvalidParameter);
        }

        let mut sector_buf = [0u8; SECTOR_SIZE];
        dev.read_sectors(target_lba, 1, &mut sector_buf)?;
        let sb = Self::decode(&sector_buf)?;

        // Ensure the decoded superblock matches the reservation contract
        let expected_data_start = reservation.absolute_data_start_lba()?;
        if sb.data_start_lba != expected_data_start {
            return Err(StorageError::CorruptedMetadata);
        }

        Ok(sb)
    }
}

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
