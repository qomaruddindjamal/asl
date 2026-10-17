//! Ext4 File System Inspection & Read Support
//! Standard Linux Second Extended File System (Ext4) Superblock & Metadata

use crate::block::{BlockDevice, StorageError, StorageResult};
use crate::vfs::{DirectoryEntry, FileSystem, VfsError, VfsNode, VfsNodeType, VfsResult};

pub const EXT4_SUPERBLOCK_OFFSET: u64 = 1024; // 1024 bytes from partition start
pub const EXT4_MAGIC: u16 = 0xEF53;

pub const EXT4_FEATURE_RO_COMPAT_SPARSE_SUPER: u32 = 0x0001;
pub const EXT4_FEATURE_RO_COMPAT_LARGE_FILE: u32 = 0x0002;
pub const EXT4_FEATURE_RO_COMPAT_BTREE_DIR: u32 = 0x0004;
pub const EXT4_FEATURE_RO_COMPAT_HUGE_FILE: u32 = 0x0008;
pub const EXT4_FEATURE_RO_COMPAT_GDT_CSUM: u32 = 0x0010;
pub const EXT4_FEATURE_RO_COMPAT_DIR_NLINK: u32 = 0x0020;
pub const EXT4_FEATURE_RO_COMPAT_EXTRA_ISIZE: u32 = 0x0040;

pub const EXT4_FEATURE_INCOMPAT_FILETYPE: u32 = 0x0002;
pub const EXT4_FEATURE_INCOMPAT_RECOVER: u32 = 0x0004;
pub const EXT4_FEATURE_INCOMPAT_JOURNAL_DEV: u32 = 0x0008;
pub const EXT4_FEATURE_INCOMPAT_META_BG: u32 = 0x0010;
pub const EXT4_FEATURE_INCOMPAT_EXTENTS: u32 = 0x0040;
pub const EXT4_FEATURE_INCOMPAT_64BIT: u32 = 0x0080;
pub const EXT4_FEATURE_INCOMPAT_MMP: u32 = 0x0100;
pub const EXT4_FEATURE_INCOMPAT_FLEX_BG: u32 = 0x0200;

#[derive(Debug, Copy, Clone)]
pub struct Ext4Superblock {
    pub inodes_count: u32,
    pub blocks_count: u64,
    pub free_blocks_count: u64,
    pub free_inodes_count: u32,
    pub first_data_block: u32,
    pub log_block_size: u32,
    pub block_size: u32,
    pub blocks_per_group: u32,
    pub inodes_per_group: u32,
    pub magic: u16,
    pub state: u16,
    pub errors: u16,
    pub minor_rev_level: u16,
    pub creator_os: u32,
    pub rev_level: u32,
    pub feature_compat: u32,
    pub feature_incompat: u32,
    pub feature_ro_compat: u32,
    pub uuid: [u8; 16],
    pub volume_name: [u8; 16],
    pub inode_size: u16,
    pub block_group_nr: u16,
}

impl Ext4Superblock {
    pub fn parse(raw: &[u8]) -> StorageResult<Self> {
        if raw.len() < 1024 {
            return Err(StorageError::BufferTooSmall);
        }

        let magic = u16::from_le_bytes([raw[0x38], raw[0x39]]);
        if magic != EXT4_MAGIC {
            return Err(StorageError::CorruptedMetadata);
        }

        let inodes_count = u32::from_le_bytes([raw[0x00], raw[0x01], raw[0x02], raw[0x03]]);
        let blocks_count_lo =
            u32::from_le_bytes([raw[0x04], raw[0x05], raw[0x06], raw[0x07]]) as u64;
        let free_blocks_lo =
            u32::from_le_bytes([raw[0x0C], raw[0x0D], raw[0x0E], raw[0x0F]]) as u64;
        let free_inodes_count = u32::from_le_bytes([raw[0x10], raw[0x11], raw[0x12], raw[0x13]]);
        let first_data_block = u32::from_le_bytes([raw[0x14], raw[0x15], raw[0x16], raw[0x17]]);
        let log_block_size = u32::from_le_bytes([raw[0x18], raw[0x19], raw[0x1A], raw[0x1B]]);
        let block_size = 1024u32.checked_shl(log_block_size).unwrap_or(4096);

        let blocks_per_group = u32::from_le_bytes([raw[0x20], raw[0x21], raw[0x22], raw[0x23]]);
        let inodes_per_group = u32::from_le_bytes([raw[0x28], raw[0x29], raw[0x2A], raw[0x2B]]);

        let state = u16::from_le_bytes([raw[0x3A], raw[0x3B]]);
        let errors = u16::from_le_bytes([raw[0x3C], raw[0x3D]]);
        let minor_rev_level = u16::from_le_bytes([raw[0x3E], raw[0x3F]]);
        let creator_os = u32::from_le_bytes([raw[0x48], raw[0x49], raw[0x4A], raw[0x4B]]);
        let rev_level = u32::from_le_bytes([raw[0x4C], raw[0x4D], raw[0x4E], raw[0x4F]]);

        let feature_compat = u32::from_le_bytes([raw[0x60], raw[0x61], raw[0x62], raw[0x63]]);
        let feature_incompat = u32::from_le_bytes([raw[0x64], raw[0x65], raw[0x66], raw[0x67]]);
        let feature_ro_compat = u32::from_le_bytes([raw[0x68], raw[0x69], raw[0x6A], raw[0x6B]]);

        let mut uuid = [0u8; 16];
        uuid.copy_from_slice(&raw[0x68..0x78]);

        let mut volume_name = [0u8; 16];
        volume_name.copy_from_slice(&raw[0x78..0x88]);

        let inode_size = if rev_level >= 1 {
            u16::from_le_bytes([raw[0x58], raw[0x59]])
        } else {
            128
        };

        let block_group_nr = u16::from_le_bytes([raw[0x5A], raw[0x5B]]);

        // Check for 64-bit feature
        let blocks_count =
            if (feature_incompat & EXT4_FEATURE_INCOMPAT_64BIT) != 0 && raw.len() >= 0x154 {
                let blocks_count_hi =
                    u32::from_le_bytes([raw[0x150], raw[0x151], raw[0x152], raw[0x153]]) as u64;
                (blocks_count_hi << 32) | blocks_count_lo
            } else {
                blocks_count_lo
            };

        let free_blocks_count =
            if (feature_incompat & EXT4_FEATURE_INCOMPAT_64BIT) != 0 && raw.len() >= 0x158 {
                let free_blocks_hi =
                    u32::from_le_bytes([raw[0x154], raw[0x155], raw[0x156], raw[0x157]]) as u64;
                (free_blocks_hi << 32) | free_blocks_lo
            } else {
                free_blocks_lo
            };

        Ok(Self {
            inodes_count,
            blocks_count,
            free_blocks_count,
            free_inodes_count,
            first_data_block,
            log_block_size,
            block_size,
            blocks_per_group,
            inodes_per_group,
            magic,
            state,
            errors,
            minor_rev_level,
            creator_os,
            rev_level,
            feature_compat,
            feature_incompat,
            feature_ro_compat,
            uuid,
            volume_name,
            inode_size,
            block_group_nr,
        })
    }

    /// Read volume label as UTF-8 string
    pub fn volume_name_str(&self) -> &str {
        let mut len = 0;
        while len < 16 && self.volume_name[len] != 0 {
            len += 1;
        }
        core::str::from_utf8(&self.volume_name[..len]).unwrap_or("")
    }

    /// Verify if journaling is active
    pub fn has_journal(&self) -> bool {
        (self.feature_compat & 0x0004) != 0
    }

    /// Verify if extents tree is used for data blocks
    pub fn has_extents(&self) -> bool {
        (self.feature_incompat & EXT4_FEATURE_INCOMPAT_EXTENTS) != 0
    }
}

pub struct Ext4FileSystem<'a, B: BlockDevice> {
    pub device: &'a mut B,
    pub superblock: Ext4Superblock,
    pub partition_start_lba: u64,
}

impl<'a, B: BlockDevice> Ext4FileSystem<'a, B> {
    pub fn mount(device: &'a mut B, partition_start_lba: u64) -> StorageResult<Self> {
        // Superblock is located at offset 1024 (LBA 2 for 512B sectors)
        let sb_lba = partition_start_lba + (EXT4_SUPERBLOCK_OFFSET / 512);
        let mut sb_buf = [0u8; 1024]; // 2 sectors

        device.read_sectors(sb_lba, 2, &mut sb_buf)?;
        let superblock = Ext4Superblock::parse(&sb_buf)?;

        Ok(Self {
            device,
            superblock,
            partition_start_lba,
        })
    }
}

impl<'a, B: BlockDevice> FileSystem for Ext4FileSystem<'a, B> {
    fn name(&self) -> &'static str {
        "Ext4 (Read-Only)"
    }

    fn lookup(&mut self, path: &str) -> VfsResult<VfsNode> {
        let trimmed = path.trim();
        if trimmed == "/" || trimmed.is_empty() {
            return Ok(VfsNode {
                node_type: VfsNodeType::Directory,
                size_bytes: 4096,
                inode: 2, // Standard Ext4 root inode is 2
            });
        }
        // Sub-path traversal requires extent tree parsing
        Err(VfsError::NotFound)
    }

    fn read(&mut self, node: &VfsNode, offset: u64, buf: &mut [u8]) -> VfsResult<usize> {
        let _ = (node, offset, buf);
        Err(VfsError::Storage(StorageError::UnsupportedFeature))
    }

    fn write(&mut self, node: &VfsNode, offset: u64, buf: &[u8]) -> VfsResult<usize> {
        let _ = (node, offset, buf);
        // Explicitly Reject Write operations to Ext4 as mandated by roadmap
        Err(VfsError::ReadOnly)
    }

    fn read_dir(&mut self, node: &VfsNode, entries: &mut [DirectoryEntry]) -> VfsResult<usize> {
        if node.node_type != VfsNodeType::Directory {
            return Err(VfsError::NotADirectory);
        }
        if entries.is_empty() {
            return Ok(0);
        }
        // Root directory basic stub
        entries[0] = DirectoryEntry::new(".", VfsNodeType::Directory, 4096, 2);
        if entries.len() > 1 {
            entries[1] = DirectoryEntry::new("..", VfsNodeType::Directory, 4096, 2);
            Ok(2)
        } else {
            Ok(1)
        }
    }

    fn create_file(&mut self, parent: &VfsNode, name: &str) -> VfsResult<VfsNode> {
        let _ = (parent, name);
        Err(VfsError::ReadOnly)
    }

    fn create_dir(&mut self, parent: &VfsNode, name: &str) -> VfsResult<VfsNode> {
        let _ = (parent, name);
        Err(VfsError::ReadOnly)
    }
}
