//! Partition Table Parser (MBR & GPT) with Strict Overlap & Boundary Validation

use crate::block::{BlockDevice, StorageError, StorageResult, SECTOR_SIZE};

pub const MAX_PARTITIONS: usize = 16;
pub const MBR_MAGIC: u16 = 0xAA55;
pub const GPT_SIGNATURE: &[u8; 8] = b"EFI PART";

#[derive(Debug, Copy, Clone, PartialEq, Eq)]
pub enum PartitionType {
    Empty,
    Fat12_16,
    Fat32,
    NtfsExfat,
    LinuxNative,
    LinuxSwap,
    EfiSystem,
    GptGenericData,
    Other(u8),
}

#[derive(Debug, Copy, Clone, PartialEq, Eq)]
pub struct Partition {
    pub part_num: u8,
    pub part_type: PartitionType,
    pub start_lba: u64,
    pub sector_count: u64,
    pub bootable: bool,
}

impl Partition {
    pub const fn empty() -> Self {
        Self {
            part_num: 0,
            part_type: PartitionType::Empty,
            start_lba: 0,
            sector_count: 0,
            bootable: false,
        }
    }

    pub fn end_lba(&self) -> u64 {
        self.start_lba.saturating_add(self.sector_count)
    }

    pub fn size_bytes(&self) -> u64 {
        self.sector_count.saturating_mul(SECTOR_SIZE as u64)
    }
}

pub struct PartitionTable {
    pub partitions: [Partition; MAX_PARTITIONS],
    pub count: usize,
    pub is_gpt: bool,
}

/// Standard IEEE 802.3 / POSIX CRC-32 (Polynomial 0xEDB88320)
pub fn crc32(data: &[u8]) -> u32 {
    let mut crc = 0xFFFF_FFFFu32;
    for &byte in data {
        crc ^= byte as u32;
        for _ in 0..8 {
            let mask = (crc & 1).wrapping_neg();
            crc = (crc >> 1) ^ (0xEDB8_8320 & mask);
        }
    }
    !crc
}

impl PartitionTable {
    pub const fn new() -> Self {
        Self {
            partitions: [Partition::empty(); MAX_PARTITIONS],
            count: 0,
            is_gpt: false,
        }
    }

    /// Parse partition table from any BlockDevice (auto-detects GPT vs MBR)
    pub fn parse<B: BlockDevice>(dev: &mut B) -> StorageResult<Self> {
        let total_sectors = dev.total_sectors();
        if total_sectors < 2 {
            return Err(StorageError::InvalidParameter);
        }

        let mut sector_buf = [0u8; SECTOR_SIZE];
        let mut table = Self::new();

        // 1. Check LBA 1 for GPT Header
        dev.read_sectors(1, 1, &mut sector_buf)?;
        if &sector_buf[0..8] == GPT_SIGNATURE {
            table.is_gpt = true;
            table.parse_gpt(dev, &sector_buf, total_sectors)?;
            table.validate_boundaries(total_sectors)?;
            return Ok(table);
        }

        // 2. Check LBA 0 for MBR
        dev.read_sectors(0, 1, &mut sector_buf)?;
        let magic = u16::from_le_bytes([sector_buf[510], sector_buf[511]]);
        if magic != MBR_MAGIC {
            return Err(StorageError::CorruptedMetadata);
        }

        table.parse_mbr(&sector_buf, total_sectors)?;
        table.validate_boundaries(total_sectors)?;
        Ok(table)
    }

    fn parse_mbr(&mut self, sector: &[u8; SECTOR_SIZE], disk_sectors: u64) -> StorageResult<()> {
        let entries_offset = 446;
        for i in 0..4 {
            let offset = entries_offset + (i * 16);
            let status = sector[offset];
            let type_byte = sector[offset + 4];
            let start_lba = u32::from_le_bytes([
                sector[offset + 8],
                sector[offset + 9],
                sector[offset + 10],
                sector[offset + 11],
            ]) as u64;
            let sector_count = u32::from_le_bytes([
                sector[offset + 12],
                sector[offset + 13],
                sector[offset + 14],
                sector[offset + 15],
            ]) as u64;

            if type_byte == 0x00 || sector_count == 0 {
                continue;
            }

            // Protective MBR check for GPT
            if type_byte == 0xEE {
                // Disk is marked protective MBR for GPT
                continue;
            }

            let part_type = match type_byte {
                0x01 | 0x04 | 0x06 | 0x0E => PartitionType::Fat12_16,
                0x0B | 0x0C => PartitionType::Fat32,
                0x07 => PartitionType::NtfsExfat,
                0x83 => PartitionType::LinuxNative,
                0x82 => PartitionType::LinuxSwap,
                0xEF => PartitionType::EfiSystem,
                other => PartitionType::Other(other),
            };

            // Range validation
            if start_lba >= disk_sectors || start_lba.saturating_add(sector_count) > disk_sectors {
                return Err(StorageError::InvalidLba);
            }

            if self.count < MAX_PARTITIONS {
                self.partitions[self.count] = Partition {
                    part_num: (i + 1) as u8,
                    part_type,
                    start_lba,
                    sector_count,
                    bootable: (status == 0x80),
                };
                self.count += 1;
            }
        }
        Ok(())
    }

    fn parse_gpt<B: BlockDevice>(
        &mut self,
        dev: &mut B,
        header_sector: &[u8; SECTOR_SIZE],
        disk_sectors: u64,
    ) -> StorageResult<()> {
        let header_size = u32::from_le_bytes([
            header_sector[12],
            header_sector[13],
            header_sector[14],
            header_sector[15],
        ]) as usize;
        if header_size < 92 || header_size > SECTOR_SIZE {
            return Err(StorageError::CorruptedMetadata);
        }

        // Validate GPT Header CRC32 (CRC is computed with CRC field zeroed)
        let recorded_crc = u32::from_le_bytes([
            header_sector[16],
            header_sector[17],
            header_sector[18],
            header_sector[19],
        ]);
        let mut crc_check_hdr = [0u8; SECTOR_SIZE];
        crc_check_hdr[..header_size].copy_from_slice(&header_sector[..header_size]);
        crc_check_hdr[16..20].fill(0); // Zero out CRC field for verification
        let computed_crc = crc32(&crc_check_hdr[..header_size]);

        // If header CRC is non-zero, it must match
        if recorded_crc != 0 && computed_crc != recorded_crc {
            return Err(StorageError::CorruptedMetadata);
        }

        let _my_lba = u64::from_le_bytes([
            header_sector[24],
            header_sector[25],
            header_sector[26],
            header_sector[27],
            header_sector[28],
            header_sector[29],
            header_sector[30],
            header_sector[31],
        ]);
        let first_usable_lba = u64::from_le_bytes([
            header_sector[40],
            header_sector[41],
            header_sector[42],
            header_sector[43],
            header_sector[44],
            header_sector[45],
            header_sector[46],
            header_sector[47],
        ]);
        let last_usable_lba = u64::from_le_bytes([
            header_sector[48],
            header_sector[49],
            header_sector[50],
            header_sector[51],
            header_sector[52],
            header_sector[53],
            header_sector[54],
            header_sector[55],
        ]);

        if first_usable_lba > last_usable_lba || last_usable_lba >= disk_sectors {
            return Err(StorageError::CorruptedMetadata);
        }

        let part_entry_lba = u64::from_le_bytes([
            header_sector[72],
            header_sector[73],
            header_sector[74],
            header_sector[75],
            header_sector[76],
            header_sector[77],
            header_sector[78],
            header_sector[79],
        ]);
        let num_part_entries = u32::from_le_bytes([
            header_sector[80],
            header_sector[81],
            header_sector[82],
            header_sector[83],
        ]) as usize;
        let entry_size = u32::from_le_bytes([
            header_sector[84],
            header_sector[85],
            header_sector[86],
            header_sector[87],
        ]) as usize;

        if entry_size < 128 || entry_size > SECTOR_SIZE || part_entry_lba >= disk_sectors {
            return Err(StorageError::CorruptedMetadata);
        }

        let mut buf = [0u8; SECTOR_SIZE];
        let entries_per_sector = SECTOR_SIZE / entry_size;
        let max_to_scan = if num_part_entries > 32 {
            32
        } else {
            num_part_entries
        };

        let mut scanned = 0;
        let mut cur_sector = part_entry_lba;

        while scanned < max_to_scan && self.count < MAX_PARTITIONS {
            dev.read_sectors(cur_sector, 1, &mut buf)?;
            for e in 0..entries_per_sector {
                if scanned >= max_to_scan || self.count >= MAX_PARTITIONS {
                    break;
                }
                let offset = e * entry_size;
                let type_guid = &buf[offset..offset + 16];
                if type_guid.iter().all(|&b| b == 0) {
                    scanned += 1;
                    continue;
                }

                let first_lba = u64::from_le_bytes([
                    buf[offset + 32],
                    buf[offset + 33],
                    buf[offset + 34],
                    buf[offset + 35],
                    buf[offset + 36],
                    buf[offset + 37],
                    buf[offset + 38],
                    buf[offset + 39],
                ]);
                let last_lba = u64::from_le_bytes([
                    buf[offset + 40],
                    buf[offset + 41],
                    buf[offset + 42],
                    buf[offset + 43],
                    buf[offset + 44],
                    buf[offset + 45],
                    buf[offset + 46],
                    buf[offset + 47],
                ]);

                if last_lba >= first_lba && first_lba < disk_sectors && last_lba < disk_sectors {
                    let sector_count = last_lba - first_lba + 1;
                    self.partitions[self.count] = Partition {
                        part_num: (scanned + 1) as u8,
                        part_type: PartitionType::GptGenericData,
                        start_lba: first_lba,
                        sector_count,
                        bootable: false,
                    };
                    self.count += 1;
                }
                scanned += 1;
            }
            cur_sector += 1;
        }
        Ok(())
    }

    /// Strict overlap and boundary validation
    pub fn validate_boundaries(&self, disk_sectors: u64) -> StorageResult<()> {
        for i in 0..self.count {
            let p1 = &self.partitions[i];
            if p1.sector_count == 0 {
                return Err(StorageError::InvalidParameter);
            }
            let p1_end = match p1.start_lba.checked_add(p1.sector_count) {
                Some(end) => end,
                None => return Err(StorageError::InvalidLba),
            };
            if p1_end > disk_sectors {
                return Err(StorageError::InvalidLba);
            }

            // Check against all other partitions for overlap
            for j in (i + 1)..self.count {
                let p2 = &self.partitions[j];
                let p2_end = match p2.start_lba.checked_add(p2.sector_count) {
                    Some(end) => end,
                    None => return Err(StorageError::InvalidLba),
                };

                // Overlap occurs if start < other_end && end > other_start
                if p1.start_lba < p2_end && p1_end > p2.start_lba {
                    return Err(StorageError::CorruptedMetadata);
                }
            }
        }
        Ok(())
    }
}
