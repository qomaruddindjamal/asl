//! FAT32 File System Driver (BPB Parsing, Directory Iteration, File Read/Write)
//! Based on Microsoft Extensible Firmware Initiative FAT32 File System Specification

use crate::block::{BlockDevice, StorageError, StorageResult, SECTOR_SIZE};
use crate::vfs::{DirectoryEntry, FileSystem, VfsError, VfsNode, VfsNodeType, VfsResult};

pub const FAT32_ATTR_READ_ONLY: u8 = 0x01;
pub const FAT32_ATTR_HIDDEN: u8 = 0x02;
pub const FAT32_ATTR_SYSTEM: u8 = 0x04;
pub const FAT32_ATTR_VOLUME_ID: u8 = 0x08;
pub const FAT32_ATTR_DIRECTORY: u8 = 0x10;
pub const FAT32_ATTR_ARCHIVE: u8 = 0x20;
pub const FAT32_ATTR_LFN: u8 = 0x0F;

pub const FAT32_ENTRY_FREE: u8 = 0xE5;
pub const FAT32_ENTRY_LAST: u8 = 0x00;

pub const FAT32_EOC: u32 = 0x0FFF_FFF8; // End of cluster chain
pub const MAX_CLUSTER_TRAVERSAL: usize = 256; // Bounded loop execution to prevent DoS via cyclic cluster chain

#[derive(Debug, Copy, Clone)]
pub struct Fat32Bpb {
    pub bytes_per_sector: u16,
    pub sectors_per_cluster: u8,
    pub reserved_sectors: u16,
    pub num_fats: u8,
    pub root_entries_count: u16,
    pub total_sectors_32: u32,
    pub fat_size_32: u32,
    pub root_cluster: u32,
    pub fs_info_sector: u16,
    pub fat_start_lba: u64,
    pub data_start_lba: u64,
}

impl Fat32Bpb {
    pub fn parse(sector: &[u8; SECTOR_SIZE]) -> StorageResult<Self> {
        let magic = u16::from_le_bytes([sector[510], sector[511]]);
        if magic != 0xAA55 {
            return Err(StorageError::CorruptedMetadata);
        }

        let bytes_per_sector = u16::from_le_bytes([sector[11], sector[12]]);
        let sectors_per_cluster = sector[13];
        let reserved_sectors = u16::from_le_bytes([sector[14], sector[15]]);
        let num_fats = sector[16];
        let root_entries_count = u16::from_le_bytes([sector[17], sector[18]]);
        let total_sectors_16 = u16::from_le_bytes([sector[19], sector[20]]);
        let fat_size_16 = u16::from_le_bytes([sector[22], sector[23]]);

        let total_sectors_32 = if total_sectors_16 == 0 {
            u32::from_le_bytes([sector[32], sector[33], sector[34], sector[35]])
        } else {
            total_sectors_16 as u32
        };

        let fat_size_32 = if fat_size_16 == 0 {
            u32::from_le_bytes([sector[36], sector[37], sector[38], sector[39]])
        } else {
            fat_size_16 as u32
        };

        let root_cluster = u32::from_le_bytes([sector[44], sector[45], sector[46], sector[47]]);
        let fs_info_sector = u16::from_le_bytes([sector[48], sector[49]]);

        if bytes_per_sector != 512 || sectors_per_cluster == 0 || num_fats == 0 || fat_size_32 == 0
        {
            return Err(StorageError::CorruptedMetadata);
        }

        let fat_start_lba = reserved_sectors as u64;
        let data_start_lba = fat_start_lba + (num_fats as u64 * fat_size_32 as u64);

        Ok(Self {
            bytes_per_sector,
            sectors_per_cluster,
            reserved_sectors,
            num_fats,
            root_entries_count,
            total_sectors_32,
            fat_size_32,
            root_cluster,
            fs_info_sector,
            fat_start_lba,
            data_start_lba,
        })
    }

    /// Calculate the first LBA sector for a given cluster number
    pub fn cluster_to_lba(&self, cluster: u32) -> StorageResult<u64> {
        if cluster < 2 {
            return Err(StorageError::InvalidParameter);
        }
        let cluster_offset = (cluster as u64 - 2)
            .checked_mul(self.sectors_per_cluster as u64)
            .ok_or(StorageError::InvalidLba)?;
        self.data_start_lba
            .checked_add(cluster_offset)
            .ok_or(StorageError::InvalidLba)
    }
}

pub struct Fat32FileSystem<'a, B: BlockDevice> {
    pub device: &'a mut B,
    pub bpb: Fat32Bpb,
    pub partition_start_lba: u64,
}

impl<'a, B: BlockDevice> Fat32FileSystem<'a, B> {
    pub fn mount(device: &'a mut B, partition_start_lba: u64) -> StorageResult<Self> {
        let mut boot_sector = [0u8; SECTOR_SIZE];
        device.read_sectors(partition_start_lba, 1, &mut boot_sector)?;
        let bpb = Fat32Bpb::parse(&boot_sector)?;

        Ok(Self {
            device,
            bpb,
            partition_start_lba,
        })
    }

    /// Read next cluster in chain from FAT table
    pub fn read_fat_entry(&mut self, cluster: u32) -> StorageResult<u32> {
        if cluster < 2 {
            return Err(StorageError::InvalidParameter);
        }

        let fat_offset = (cluster as u64)
            .checked_mul(4)
            .ok_or(StorageError::InvalidLba)?;
        let fat_sector = self
            .partition_start_lba
            .checked_add(self.bpb.fat_start_lba)
            .and_then(|lba| lba.checked_add(fat_offset / 512))
            .ok_or(StorageError::InvalidLba)?;
        let entry_offset = (fat_offset % 512) as usize;

        let mut sector_buf = [0u8; SECTOR_SIZE];
        self.device.read_sectors(fat_sector, 1, &mut sector_buf)?;

        let next = u32::from_le_bytes([
            sector_buf[entry_offset],
            sector_buf[entry_offset + 1],
            sector_buf[entry_offset + 2],
            sector_buf[entry_offset + 3],
        ]) & 0x0FFF_FFFF;

        // Check for BAD CLUSTER (0x0FFF_FFF7)
        if next == 0x0FFF_FFF7 {
            return Err(StorageError::CorruptedMetadata);
        }

        Ok(next)
    }

    /// Write FAT entry into FAT table (for allocating and extending files)
    pub fn write_fat_entry(&mut self, cluster: u32, next_cluster: u32) -> StorageResult<()> {
        let fat_offset = (cluster as u64) * 4;
        let fat_sector = self.partition_start_lba + self.bpb.fat_start_lba + (fat_offset / 512);
        let entry_offset = (fat_offset % 512) as usize;

        let mut sector_buf = [0u8; SECTOR_SIZE];
        self.device.read_sectors(fat_sector, 1, &mut sector_buf)?;

        let existing = u32::from_le_bytes([
            sector_buf[entry_offset],
            sector_buf[entry_offset + 1],
            sector_buf[entry_offset + 2],
            sector_buf[entry_offset + 3],
        ]) & 0xF000_0000;

        let to_write = (next_cluster & 0x0FFF_FFFF) | existing;
        let bytes = to_write.to_le_bytes();
        sector_buf[entry_offset..entry_offset + 4].copy_from_slice(&bytes);

        // Update all FAT copies
        for f in 0..self.bpb.num_fats {
            let cur_fat_sector = fat_sector + (f as u64 * self.bpb.fat_size_32 as u64);
            self.device.write_sectors(cur_fat_sector, 1, &sector_buf)?;
        }
        Ok(())
    }
}

impl<'a, B: BlockDevice> FileSystem for Fat32FileSystem<'a, B> {
    fn name(&self) -> &'static str {
        "FAT32"
    }

    fn lookup(&mut self, path: &str) -> VfsResult<VfsNode> {
        let trimmed = path.trim();
        if trimmed == "/" || trimmed.is_empty() {
            return Ok(VfsNode {
                node_type: VfsNodeType::Directory,
                size_bytes: 0,
                inode: self.bpb.root_cluster as u64,
            });
        }

        // Search directory entries starting from root cluster
        let target_name = trimmed.trim_start_matches('/');
        let mut cur_cluster = self.bpb.root_cluster;
        let mut sector_buf = [0u8; SECTOR_SIZE];
        let mut steps = 0;

        while cur_cluster < 0x0FFF_FFF8 && cur_cluster >= 2 {
            steps += 1;
            if steps > MAX_CLUSTER_TRAVERSAL {
                return Err(VfsError::Storage(StorageError::CorruptedMetadata));
            }

            let lba_base = match self.bpb.cluster_to_lba(cur_cluster) {
                Ok(lba) => self.partition_start_lba + lba,
                Err(e) => return Err(e.into()),
            };

            for s in 0..self.bpb.sectors_per_cluster {
                if let Err(e) = self
                    .device
                    .read_sectors(lba_base + s as u64, 1, &mut sector_buf)
                {
                    return Err(e.into());
                }

                for d in 0..(SECTOR_SIZE / 32) {
                    let offset = d * 32;
                    let first_byte = sector_buf[offset];
                    if first_byte == FAT32_ENTRY_LAST {
                        return Err(VfsError::NotFound);
                    }
                    if first_byte == FAT32_ENTRY_FREE {
                        continue;
                    }

                    let attr = sector_buf[offset + 11];
                    if attr == FAT32_ATTR_LFN || (attr & FAT32_ATTR_VOLUME_ID) != 0 {
                        continue;
                    }

                    // Format 8.3 filename
                    let mut name_buf = [b' '; 11];
                    name_buf.copy_from_slice(&sector_buf[offset..offset + 11]);
                    let entry_name = format_83_name(&name_buf);

                    if entry_name.eq_ignore_ascii_case(target_name) {
                        let cluster_hi =
                            u16::from_le_bytes([sector_buf[offset + 20], sector_buf[offset + 21]])
                                as u32;
                        let cluster_lo =
                            u16::from_le_bytes([sector_buf[offset + 26], sector_buf[offset + 27]])
                                as u32;
                        let start_cluster = (cluster_hi << 16) | cluster_lo;
                        let file_size = u32::from_le_bytes([
                            sector_buf[offset + 28],
                            sector_buf[offset + 29],
                            sector_buf[offset + 30],
                            sector_buf[offset + 31],
                        ]) as u64;

                        let node_type = if (attr & FAT32_ATTR_DIRECTORY) != 0 {
                            VfsNodeType::Directory
                        } else {
                            VfsNodeType::File
                        };

                        return Ok(VfsNode {
                            node_type,
                            size_bytes: file_size,
                            inode: start_cluster as u64,
                        });
                    }
                }
            }

            match self.read_fat_entry(cur_cluster) {
                Ok(next) => cur_cluster = next,
                Err(e) => return Err(e.into()),
            }
        }

        Err(VfsError::NotFound)
    }

    fn read(&mut self, node: &VfsNode, offset: u64, buf: &mut [u8]) -> VfsResult<usize> {
        if node.node_type != VfsNodeType::File {
            return Err(VfsError::IsADirectory);
        }
        if offset >= node.size_bytes {
            return Ok(0);
        }

        let bytes_to_read = core::cmp::min(buf.len(), (node.size_bytes - offset) as usize);

        let mut cur_cluster = node.inode as u32;
        let mut cur_file_offset: usize = 0;
        let mut read_bytes: usize = 0;
        let mut sector_buf = [0u8; SECTOR_SIZE];
        let mut steps = 0;

        while cur_cluster < 0x0FFF_FFF8 && cur_cluster >= 2 && read_bytes < bytes_to_read {
            steps += 1;
            if steps > MAX_CLUSTER_TRAVERSAL {
                return Err(VfsError::Storage(StorageError::CorruptedMetadata));
            }

            let lba_base = match self.bpb.cluster_to_lba(cur_cluster) {
                Ok(lba) => self.partition_start_lba + lba,
                Err(e) => return Err(e.into()),
            };

            for s in 0..self.bpb.sectors_per_cluster {
                let sector_offset = cur_file_offset;
                cur_file_offset += SECTOR_SIZE;

                if (offset as usize) < cur_file_offset && read_bytes < bytes_to_read {
                    if let Err(e) =
                        self.device
                            .read_sectors(lba_base + s as u64, 1, &mut sector_buf)
                    {
                        return Err(e.into());
                    }

                    let in_sector_start = if (offset as usize) > sector_offset {
                        (offset as usize) - sector_offset
                    } else {
                        0
                    };
                    let in_sector_avail = SECTOR_SIZE - in_sector_start;
                    let chunk = core::cmp::min(bytes_to_read - read_bytes, in_sector_avail);

                    buf[read_bytes..read_bytes + chunk]
                        .copy_from_slice(&sector_buf[in_sector_start..in_sector_start + chunk]);
                    read_bytes += chunk;
                }
            }

            match self.read_fat_entry(cur_cluster) {
                Ok(next) => cur_cluster = next,
                Err(e) => return Err(e.into()),
            }
        }

        Ok(read_bytes)
    }

    fn write(&mut self, node: &VfsNode, offset: u64, buf: &[u8]) -> VfsResult<usize> {
        if node.node_type != VfsNodeType::File {
            return Err(VfsError::IsADirectory);
        }

        let mut cur_cluster = node.inode as u32;
        if cur_cluster < 2 {
            return Err(VfsError::Storage(StorageError::NoSpaceLeft));
        }

        let mut cur_file_offset: usize = 0;
        let mut written_bytes: usize = 0;
        let mut sector_buf = [0u8; SECTOR_SIZE];
        let mut steps = 0;

        while cur_cluster < 0x0FFF_FFF8 && cur_cluster >= 2 && written_bytes < buf.len() {
            steps += 1;
            if steps > MAX_CLUSTER_TRAVERSAL {
                return Err(VfsError::Storage(StorageError::CorruptedMetadata));
            }

            let lba_base = match self.bpb.cluster_to_lba(cur_cluster) {
                Ok(lba) => self.partition_start_lba + lba,
                Err(e) => return Err(e.into()),
            };

            for s in 0..self.bpb.sectors_per_cluster {
                let sector_offset = cur_file_offset;
                cur_file_offset += SECTOR_SIZE;

                if (offset as usize) < cur_file_offset && written_bytes < buf.len() {
                    let in_sector_start = if (offset as usize) > sector_offset {
                        (offset as usize) - sector_offset
                    } else {
                        0
                    };

                    if let Err(e) =
                        self.device
                            .read_sectors(lba_base + s as u64, 1, &mut sector_buf)
                    {
                        return Err(e.into());
                    }

                    let in_sector_avail = SECTOR_SIZE - in_sector_start;
                    let chunk = core::cmp::min(buf.len() - written_bytes, in_sector_avail);

                    sector_buf[in_sector_start..in_sector_start + chunk]
                        .copy_from_slice(&buf[written_bytes..written_bytes + chunk]);

                    if let Err(e) = self
                        .device
                        .write_sectors(lba_base + s as u64, 1, &sector_buf)
                    {
                        return Err(e.into());
                    }

                    written_bytes += chunk;
                }
            }

            match self.read_fat_entry(cur_cluster) {
                Ok(next) => cur_cluster = next,
                Err(e) => return Err(e.into()),
            }
        }

        Ok(written_bytes)
    }

    fn read_dir(&mut self, node: &VfsNode, entries: &mut [DirectoryEntry]) -> VfsResult<usize> {
        if node.node_type != VfsNodeType::Directory {
            return Err(VfsError::NotADirectory);
        }

        let mut cur_cluster = node.inode as u32;
        let mut count = 0;
        let mut sector_buf = [0u8; SECTOR_SIZE];
        let mut steps = 0;

        while cur_cluster < 0x0FFF_FFF8 && cur_cluster >= 2 && count < entries.len() {
            steps += 1;
            if steps > MAX_CLUSTER_TRAVERSAL {
                return Err(VfsError::Storage(StorageError::CorruptedMetadata));
            }

            let lba_base = match self.bpb.cluster_to_lba(cur_cluster) {
                Ok(lba) => self.partition_start_lba + lba,
                Err(e) => return Err(e.into()),
            };

            for s in 0..self.bpb.sectors_per_cluster {
                if let Err(e) = self
                    .device
                    .read_sectors(lba_base + s as u64, 1, &mut sector_buf)
                {
                    return Err(e.into());
                }

                for d in 0..(SECTOR_SIZE / 32) {
                    if count >= entries.len() {
                        return Ok(count);
                    }
                    let offset = d * 32;
                    let first_byte = sector_buf[offset];
                    if first_byte == FAT32_ENTRY_LAST {
                        return Ok(count);
                    }
                    if first_byte == FAT32_ENTRY_FREE {
                        continue;
                    }

                    let attr = sector_buf[offset + 11];
                    if attr == FAT32_ATTR_LFN || (attr & FAT32_ATTR_VOLUME_ID) != 0 {
                        continue;
                    }

                    let mut name_buf = [b' '; 11];
                    name_buf.copy_from_slice(&sector_buf[offset..offset + 11]);
                    let entry_name = format_83_name(&name_buf);

                    let cluster_hi =
                        u16::from_le_bytes([sector_buf[offset + 20], sector_buf[offset + 21]])
                            as u32;
                    let cluster_lo =
                        u16::from_le_bytes([sector_buf[offset + 26], sector_buf[offset + 27]])
                            as u32;
                    let start_cluster = (cluster_hi << 16) | cluster_lo;
                    let file_size = u32::from_le_bytes([
                        sector_buf[offset + 28],
                        sector_buf[offset + 29],
                        sector_buf[offset + 30],
                        sector_buf[offset + 31],
                    ]) as u64;

                    let node_type = if (attr & FAT32_ATTR_DIRECTORY) != 0 {
                        VfsNodeType::Directory
                    } else {
                        VfsNodeType::File
                    };

                    entries[count] = DirectoryEntry::new(
                        entry_name.as_str(),
                        node_type,
                        file_size,
                        start_cluster as u64,
                    );
                    count += 1;
                }
            }

            match self.read_fat_entry(cur_cluster) {
                Ok(next) => cur_cluster = next,
                Err(e) => return Err(e.into()),
            }
        }

        Ok(count)
    }

    fn create_file(&mut self, parent: &VfsNode, name: &str) -> VfsResult<VfsNode> {
        let _ = (parent, name);
        // Bounded stub: Writing brand-new FAT allocations requires free-cluster scanning
        Err(VfsError::Storage(StorageError::UnsupportedFeature))
    }

    fn create_dir(&mut self, parent: &VfsNode, name: &str) -> VfsResult<VfsNode> {
        let _ = (parent, name);
        Err(VfsError::Storage(StorageError::UnsupportedFeature))
    }
}

pub struct ShortFileName {
    buf: [u8; 13],
    len: usize,
}

impl ShortFileName {
    pub fn as_str(&self) -> &str {
        core::str::from_utf8(&self.buf[..self.len]).unwrap_or("")
    }

    pub fn eq_ignore_ascii_case(&self, other: &str) -> bool {
        self.as_str().eq_ignore_ascii_case(other)
    }
}

fn format_83_name(raw: &[u8; 11]) -> ShortFileName {
    let mut s = ShortFileName {
        buf: [0u8; 13],
        len: 0,
    };
    let mut base_len = 8;
    while base_len > 0 && raw[base_len - 1] == b' ' {
        base_len -= 1;
    }
    for i in 0..base_len {
        s.buf[s.len] = raw[i];
        s.len += 1;
    }

    let mut ext_len = 3;
    while ext_len > 0 && raw[8 + ext_len - 1] == b' ' {
        ext_len -= 1;
    }
    if ext_len > 0 {
        s.buf[s.len] = b'.';
        s.len += 1;
        for i in 0..ext_len {
            s.buf[s.len] = raw[8 + i];
            s.len += 1;
        }
    }
    s
}
