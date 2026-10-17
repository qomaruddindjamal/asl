//! ASL Storage Subsystem: Block Device, Partition Parser, ATA PIO, AHCI, and VFS
//! ===================================================================================
//! DO-178C DAL-A & Safe Storage Management:
//! 1. Bounded BlockDevice trait (LBA, sector addressing, zero-copy buffer validation)
//! 2. Partition Parser (MBR & GPT) with strict boundary and overlap validation
//! 3. ATA PIO Driver (Primary & Secondary channel, IDENTIFY DEVICE, LBA28/48, timeouts)
//! 4. AHCI Controller Discovery & HBA Memory Registers (CAP, GHC, PxCLB, PxFIS, PxCMD, PxIS)
//! 5. Virtual File System (VFS) with mount points, inode/dir entries, path resolution, and read/write interfaces
//! 6. FAT32 Driver (BPB validation, Cluster math, root directory iteration, file read/write)
//! 7. Ext4 Reader (Superblock verification, magic 0xEF53, block size, inode size, read-only capability)
//! 8. Safe In-Memory Block Device for deterministic unit & fuzz testing

#![no_std]

pub mod ahci;
pub mod ata;
pub mod block;
pub mod ext4;
pub mod fat32;
pub mod partition;
pub mod raid;
pub mod ramdisk;
pub mod vfs;

pub use ahci::{AhciController, AhciPortType, HbaPort};
pub use ata::{
    AtaChannel, AtaDeviceType, AtaPioDriver, ATA_PRIMARY_CTRL_BASE, ATA_PRIMARY_IO_BASE,
};
pub use block::{BlockDevice, StorageError, StorageResult, SECTOR_SIZE};
pub use ext4::{Ext4FileSystem, Ext4Superblock, EXT4_FEATURE_INCOMPAT_EXTENTS, EXT4_MAGIC};
pub use fat32::{Fat32Bpb, Fat32FileSystem};
pub use partition::{
    Partition, PartitionTable, PartitionType, GPT_GUID_LINUX_RAID, GPT_SIGNATURE, MBR_MAGIC,
    MBR_TYPE_LINUX_RAID,
};
pub use raid::{
    DedicatedPartitionReservation, GenerationOrder, Raid0Volume, Raid1MemberStatus, Raid1Volume,
    Raid1VolumeState, RaidInitializationPolicy, RaidMetadataState, RaidSuperblock, RaidType,
    DEFAULT_STRIPE_SECTORS, MAX_RAID_MEMBERS, RAID_RESERVED_SECTORS, RAID_SUPERBLOCK_MAGIC,
    RAID_SUPERBLOCK_VERSION_1, RAID_SUPERBLOCK_WIRE_SIZE,
};
pub use ramdisk::MemBlockDevice;
pub use vfs::{DirectoryEntry, FileSystem, Vfs, VfsError, VfsNode, VfsNodeType, VfsResult};

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_ramdisk_bounded_read_write() {
        let mut dev = MemBlockDevice::<{ 16 * SECTOR_SIZE }>::new();
        assert_eq!(dev.total_sectors(), 16);
        assert_eq!(dev.capacity_bytes(), 16 * 512);

        let write_buf = [0xA5u8; 512];
        assert_eq!(dev.write_sectors(0, 1, &write_buf), Ok(()));

        let mut read_buf = [0u8; 512];
        assert_eq!(dev.read_sectors(0, 1, &mut read_buf), Ok(()));
        assert_eq!(read_buf, write_buf);

        // Out of bounds LBA
        assert_eq!(
            dev.read_sectors(16, 1, &mut read_buf),
            Err(StorageError::InvalidLba)
        );
        assert_eq!(
            dev.write_sectors(16, 1, &write_buf),
            Err(StorageError::InvalidLba)
        );

        // Buffer too small
        let mut small_buf = [0u8; 256];
        assert_eq!(
            dev.read_sectors(0, 1, &mut small_buf),
            Err(StorageError::BufferTooSmall)
        );
    }

    #[test]
    fn test_ramdisk_read_only_and_error_injection() {
        let mut dev = MemBlockDevice::<{ 16 * SECTOR_SIZE }>::new();
        dev.set_read_only(true);
        let buf = [0x55u8; 512];
        assert_eq!(
            dev.write_sectors(0, 1, &buf),
            Err(StorageError::WriteProtected)
        );

        dev.set_read_only(false);
        dev.set_inject_error(true);
        let mut r_buf = [0u8; 512];
        assert_eq!(
            dev.read_sectors(0, 1, &mut r_buf),
            Err(StorageError::IoError)
        );
        assert_eq!(dev.write_sectors(0, 1, &buf), Err(StorageError::IoError));
    }

    #[test]
    fn test_mbr_partition_parsing_and_boundary_validation() {
        let mut dev = MemBlockDevice::<{ 1024 * SECTOR_SIZE }>::new();

        // Construct valid MBR at sector 0
        let mut mbr = [0u8; SECTOR_SIZE];
        mbr[510] = 0x55;
        mbr[511] = 0xAA;

        // Partition 1: FAT32, start LBA 64, count 200
        let p1_offset = 446;
        mbr[p1_offset + 4] = 0x0C; // FAT32 LBA
        let p1_start = 64u32.to_le_bytes();
        let p1_count = 200u32.to_le_bytes();
        mbr[p1_offset + 8..p1_offset + 12].copy_from_slice(&p1_start);
        mbr[p1_offset + 12..p1_offset + 16].copy_from_slice(&p1_count);

        // Partition 2: Linux Native (Ext4), start LBA 300, count 500
        let p2_offset = 446 + 16;
        mbr[p2_offset + 4] = 0x83; // Linux native
        let p2_start = 300u32.to_le_bytes();
        let p2_count = 500u32.to_le_bytes();
        mbr[p2_offset + 8..p2_offset + 12].copy_from_slice(&p2_start);
        mbr[p2_offset + 12..p2_offset + 16].copy_from_slice(&p2_count);

        dev.write_sectors(0, 1, &mbr).unwrap();

        let table = PartitionTable::parse(&mut dev).unwrap();
        assert_eq!(table.count, 2);
        assert_eq!(table.is_gpt, false);
        assert_eq!(table.partitions[0].part_type, PartitionType::Fat32);
        assert_eq!(table.partitions[0].start_lba, 64);
        assert_eq!(table.partitions[0].sector_count, 200);
        assert_eq!(table.partitions[1].part_type, PartitionType::LinuxNative);
        assert_eq!(table.partitions[1].start_lba, 300);
        assert_eq!(table.partitions[1].sector_count, 500);
    }

    #[test]
    fn test_mbr_overlapping_partition_rejection() {
        let mut dev = MemBlockDevice::<{ 1024 * SECTOR_SIZE }>::new();

        let mut mbr = [0u8; SECTOR_SIZE];
        mbr[510] = 0x55;
        mbr[511] = 0xAA;

        // Partition 1: LBA 100..300
        let p1_offset = 446;
        mbr[p1_offset + 4] = 0x0B;
        mbr[p1_offset + 8..p1_offset + 12].copy_from_slice(&100u32.to_le_bytes());
        mbr[p1_offset + 12..p1_offset + 16].copy_from_slice(&200u32.to_le_bytes());

        // Partition 2: Overlapping LBA 250..450
        let p2_offset = 446 + 16;
        mbr[p2_offset + 4] = 0x83;
        mbr[p2_offset + 8..p2_offset + 12].copy_from_slice(&250u32.to_le_bytes());
        mbr[p2_offset + 12..p2_offset + 16].copy_from_slice(&200u32.to_le_bytes());

        dev.write_sectors(0, 1, &mbr).unwrap();

        // Must reject overlapping partition table
        assert_eq!(
            PartitionTable::parse(&mut dev).err(),
            Some(StorageError::CorruptedMetadata)
        );
    }

    #[test]
    fn test_fat32_bpb_validation_and_cluster_math() {
        let mut sector = [0u8; SECTOR_SIZE];
        sector[510] = 0x55;
        sector[511] = 0xAA;
        sector[11..13].copy_from_slice(&512u16.to_le_bytes()); // Bytes per sector: 512
        sector[13] = 8; // Sectors per cluster: 8 (4KB cluster)
        sector[14..16].copy_from_slice(&32u16.to_le_bytes()); // Reserved sectors: 32
        sector[16] = 2; // Num FATs: 2
        sector[36..40].copy_from_slice(&1000u32.to_le_bytes()); // FAT size 32: 1000 sectors
        sector[44..48].copy_from_slice(&2u32.to_le_bytes()); // Root cluster: 2

        let bpb = Fat32Bpb::parse(&sector).unwrap();
        assert_eq!(bpb.bytes_per_sector, 512);
        assert_eq!(bpb.sectors_per_cluster, 8);
        assert_eq!(bpb.fat_start_lba, 32);
        assert_eq!(bpb.data_start_lba, 32 + (2 * 1000)); // 2032

        // Cluster 2 first LBA should match data_start_lba
        assert_eq!(bpb.cluster_to_lba(2).unwrap(), 2032);
        // Cluster 3 should be 2032 + 8 = 2040
        assert_eq!(bpb.cluster_to_lba(3).unwrap(), 2040);
        // Cluster < 2 is invalid
        assert_eq!(
            bpb.cluster_to_lba(1).err(),
            Some(StorageError::InvalidParameter)
        );
    }

    #[test]
    fn test_ext4_superblock_parsing() {
        let mut raw = [0u8; 1024];
        raw[0x38] = 0x53; // Magic 0xEF53
        raw[0x39] = 0xEF;
        raw[0x00..0x04].copy_from_slice(&1024u32.to_le_bytes()); // Inodes count
        raw[0x04..0x08].copy_from_slice(&4096u32.to_le_bytes()); // Blocks count
        raw[0x18..0x1C].copy_from_slice(&2u32.to_le_bytes()); // Log block size 2 (1024 << 2 = 4096)
        raw[0x4C..0x50].copy_from_slice(&1u32.to_le_bytes()); // Rev level 1
        raw[0x58..0x5A].copy_from_slice(&256u16.to_le_bytes()); // Inode size 256
        raw[0x64..0x68].copy_from_slice(&EXT4_FEATURE_INCOMPAT_EXTENTS.to_le_bytes());

        // Volume name
        let name = b"ASL_BOOT_ROOT";
        raw[0x78..0x78 + name.len()].copy_from_slice(name);

        let sb = Ext4Superblock::parse(&raw).unwrap();
        assert_eq!(sb.magic, EXT4_MAGIC);
        assert_eq!(sb.block_size, 4096);
        assert_eq!(sb.inodes_count, 1024);
        assert_eq!(sb.blocks_count, 4096);
        assert_eq!(sb.inode_size, 256);
        assert_eq!(sb.volume_name_str(), "ASL_BOOT_ROOT");
        assert_eq!(sb.has_extents(), true);
    }

    #[test]
    fn test_vfs_directory_and_file_operations() {
        let mut vfs = Vfs::new();
        assert_eq!(vfs.is_mounted(), false);

        // Dummy filesystem implementation for testing VFS dispatch
        struct TestFs;
        impl FileSystem for TestFs {
            fn name(&self) -> &'static str {
                "TestFs"
            }
            fn lookup(&mut self, path: &str) -> VfsResult<VfsNode> {
                if path == "/" {
                    Ok(VfsNode {
                        node_type: VfsNodeType::Directory,
                        size_bytes: 0,
                        inode: 1,
                    })
                } else if path == "/test.txt" {
                    Ok(VfsNode {
                        node_type: VfsNodeType::File,
                        size_bytes: 12,
                        inode: 2,
                    })
                } else {
                    Err(VfsError::NotFound)
                }
            }
            fn read(&mut self, node: &VfsNode, offset: u64, buf: &mut [u8]) -> VfsResult<usize> {
                if node.inode == 2 {
                    let content = b"Hello, ASL!";
                    if offset >= content.len() as u64 {
                        return Ok(0);
                    }
                    let start = offset as usize;
                    let len = core::cmp::min(buf.len(), content.len() - start);
                    buf[..len].copy_from_slice(&content[start..start + len]);
                    Ok(len)
                } else {
                    Err(VfsError::IsADirectory)
                }
            }
            fn write(&mut self, _node: &VfsNode, _offset: u64, _buf: &[u8]) -> VfsResult<usize> {
                Err(VfsError::ReadOnly)
            }
            fn read_dir(
                &mut self,
                node: &VfsNode,
                entries: &mut [DirectoryEntry],
            ) -> VfsResult<usize> {
                if node.inode == 1 && entries.len() >= 1 {
                    entries[0] = DirectoryEntry::new("test.txt", VfsNodeType::File, 12, 2);
                    Ok(1)
                } else {
                    Ok(0)
                }
            }
            fn create_file(&mut self, _parent: &VfsNode, _name: &str) -> VfsResult<VfsNode> {
                Err(VfsError::ReadOnly)
            }
            fn create_dir(&mut self, _parent: &VfsNode, _name: &str) -> VfsResult<VfsNode> {
                Err(VfsError::ReadOnly)
            }
        }

        let mut test_fs = TestFs;
        vfs.mount_root(&mut test_fs);
        assert_eq!(vfs.is_mounted(), true);

        let root_node = vfs.open("/").unwrap();
        assert_eq!(root_node.node_type, VfsNodeType::Directory);

        let mut entries = [DirectoryEntry::empty(); 4];
        let count = vfs.read_dir(&root_node, &mut entries).unwrap();
        assert_eq!(count, 1);
        assert_eq!(entries[0].name_as_str(), "test.txt");

        let file_node = vfs.open("/test.txt").unwrap();
        assert_eq!(file_node.node_type, VfsNodeType::File);
        let mut content_buf = [0u8; 32];
        let r_len = vfs.read(&file_node, 0, &mut content_buf).unwrap();
        assert_eq!(&content_buf[..r_len], b"Hello, ASL!");
    }

    #[test]
    fn test_gpt_crc32_and_bounds() {
        let mut dev = MemBlockDevice::<{ 1024 * SECTOR_SIZE }>::new();

        let mut gpt_hdr = [0u8; SECTOR_SIZE];
        gpt_hdr[0..8].copy_from_slice(GPT_SIGNATURE);
        gpt_hdr[12..16].copy_from_slice(&92u32.to_le_bytes()); // Header size 92
        gpt_hdr[24..32].copy_from_slice(&1u64.to_le_bytes()); // My LBA 1
        gpt_hdr[40..48].copy_from_slice(&34u64.to_le_bytes()); // First usable LBA 34
        gpt_hdr[48..56].copy_from_slice(&1000u64.to_le_bytes()); // Last usable LBA 1000
        gpt_hdr[72..80].copy_from_slice(&2u64.to_le_bytes()); // Partition entries LBA 2
        gpt_hdr[80..84].copy_from_slice(&32u32.to_le_bytes()); // Num entries 32
        gpt_hdr[84..88].copy_from_slice(&128u32.to_le_bytes()); // Entry size 128

        // Compute and store valid CRC32
        let computed = partition::crc32(&gpt_hdr[..92]);
        gpt_hdr[16..20].copy_from_slice(&computed.to_le_bytes());

        // Partition 1 at LBA 2 (first entry)
        let mut part_entries = [0u8; SECTOR_SIZE];
        part_entries[0..16].copy_from_slice(&[0xAF; 16]); // Non-empty partition type GUID
        part_entries[32..40].copy_from_slice(&64u64.to_le_bytes()); // First LBA 64
        part_entries[40..48].copy_from_slice(&200u64.to_le_bytes()); // Last LBA 200

        dev.write_sectors(1, 1, &gpt_hdr).unwrap();
        dev.write_sectors(2, 1, &part_entries).unwrap();

        let table = PartitionTable::parse(&mut dev).unwrap();
        assert_eq!(table.is_gpt, true);
        assert_eq!(table.count, 1);
        assert_eq!(table.partitions[0].start_lba, 64);
        assert_eq!(table.partitions[0].sector_count, 137); // 200 - 64 + 1
    }

    #[test]
    fn test_fat32_full_read_write_directory_cycle() {
        let mut dev = MemBlockDevice::<{ 256 * SECTOR_SIZE }>::new();

        // 1. Build FAT32 boot sector at sector 0
        let mut bpb_sec = [0u8; SECTOR_SIZE];
        bpb_sec[510] = 0x55;
        bpb_sec[511] = 0xAA;
        bpb_sec[11..13].copy_from_slice(&512u16.to_le_bytes());
        bpb_sec[13] = 1; // 1 sector per cluster
        bpb_sec[14..16].copy_from_slice(&32u16.to_le_bytes()); // 32 reserved sectors
        bpb_sec[16] = 2; // 2 FATs
        bpb_sec[36..40].copy_from_slice(&10u32.to_le_bytes()); // 10 sectors per FAT
        bpb_sec[44..48].copy_from_slice(&2u32.to_le_bytes()); // Root cluster 2

        dev.write_sectors(0, 1, &bpb_sec).unwrap();

        // Data start LBA = 32 + (2 * 10) = 52. Cluster 2 is at LBA 52.
        // Cluster 3 is at LBA 53.

        // Setup FAT: Cluster 2 points to EOC (0x0FFF_FFF8)
        let mut fat_sec = [0u8; SECTOR_SIZE];
        fat_sec[8..12].copy_from_slice(&0x0FFF_FFF8u32.to_le_bytes()); // Cluster 2 EOC
        fat_sec[12..16].copy_from_slice(&0x0FFF_FFF8u32.to_le_bytes()); // Cluster 3 EOC
        dev.write_sectors(32, 1, &fat_sec).unwrap(); // FAT 1
        dev.write_sectors(42, 1, &fat_sec).unwrap(); // FAT 2

        // Write a directory entry in Cluster 2 (LBA 52) for "CONFIG  TXT" -> Cluster 3, size 16
        let mut root_dir_sec = [0u8; SECTOR_SIZE];
        root_dir_sec[0..11].copy_from_slice(b"CONFIG  TXT");
        root_dir_sec[11] = 0x20; // Archive
        root_dir_sec[20..22].copy_from_slice(&0u16.to_le_bytes()); // Cluster Hi: 0
        root_dir_sec[26..28].copy_from_slice(&3u16.to_le_bytes()); // Cluster Lo: 3
        root_dir_sec[28..32].copy_from_slice(&16u32.to_le_bytes()); // Size: 16
        dev.write_sectors(52, 1, &root_dir_sec).unwrap();

        // Write payload to Cluster 3 (LBA 53)
        let mut file_sec = [0u8; SECTOR_SIZE];
        file_sec[..16].copy_from_slice(b"ASL_STORAGE_V102");
        dev.write_sectors(53, 1, &file_sec).unwrap();

        // Mount FAT32 and read directory & file
        let mut fat_fs = Fat32FileSystem::mount(&mut dev, 0).unwrap();
        let root = fat_fs.lookup("/").unwrap();
        assert_eq!(root.node_type, VfsNodeType::Directory);

        let mut entries = [DirectoryEntry::empty(); 4];
        let count = fat_fs.read_dir(&root, &mut entries).unwrap();
        assert_eq!(count, 1);
        assert_eq!(entries[0].name_as_str(), "CONFIG.TXT");

        let file = fat_fs.lookup("CONFIG.TXT").unwrap();
        assert_eq!(file.node_type, VfsNodeType::File);
        assert_eq!(file.size_bytes, 16);

        let mut read_buf = [0u8; 16];
        let bytes_read = fat_fs.read(&file, 0, &mut read_buf).unwrap();
        assert_eq!(bytes_read, 16);
        assert_eq!(&read_buf, b"ASL_STORAGE_V102");

        // Write update to file and read back
        let new_data = b"ASL_MODIFIED_DAT";
        let bytes_written = fat_fs.write(&file, 0, new_data).unwrap();
        assert_eq!(bytes_written, 16);

        let mut readback_buf = [0u8; 16];
        fat_fs.read(&file, 0, &mut readback_buf).unwrap();
        assert_eq!(&readback_buf, new_data);
    }

    #[test]
    fn test_fat32_fragmented_file_read_write_flush() {
        let mut dev = MemBlockDevice::<{ 256 * SECTOR_SIZE }>::new();

        // BPB at sector 0: 1 sector per cluster, 32 reserved, 2 FATs, 10 sectors/FAT, root at cluster 2
        let mut bpb_sec = [0u8; SECTOR_SIZE];
        bpb_sec[510] = 0x55;
        bpb_sec[511] = 0xAA;
        bpb_sec[11..13].copy_from_slice(&512u16.to_le_bytes());
        bpb_sec[13] = 1;
        bpb_sec[14..16].copy_from_slice(&32u16.to_le_bytes());
        bpb_sec[16] = 2;
        bpb_sec[36..40].copy_from_slice(&10u32.to_le_bytes());
        bpb_sec[44..48].copy_from_slice(&2u32.to_le_bytes());
        dev.write_sectors(0, 1, &bpb_sec).unwrap();

        // Fragmented file: Cluster 3 -> Cluster 7 -> EOC (0x0FFF_FFF8)
        let mut fat_sec = [0u8; SECTOR_SIZE];
        fat_sec[8..12].copy_from_slice(&0x0FFF_FFF8u32.to_le_bytes()); // Cluster 2 (root dir) EOC
        fat_sec[12..16].copy_from_slice(&7u32.to_le_bytes()); // Cluster 3 -> 7 (fragmented jump)
        fat_sec[28..32].copy_from_slice(&0x0FFF_FFF8u32.to_le_bytes()); // Cluster 7 EOC
        dev.write_sectors(32, 1, &fat_sec).unwrap();
        dev.write_sectors(42, 1, &fat_sec).unwrap();

        // Root directory at Cluster 2 (LBA 52)
        let mut root_dir_sec = [0u8; SECTOR_SIZE];
        root_dir_sec[0..11].copy_from_slice(b"FRAG    DAT");
        root_dir_sec[11] = 0x20;
        root_dir_sec[20..22].copy_from_slice(&0u16.to_le_bytes());
        root_dir_sec[26..28].copy_from_slice(&3u16.to_le_bytes()); // Starts at Cluster 3
        root_dir_sec[28..32].copy_from_slice(&1024u32.to_le_bytes()); // Size 1024 (2 sectors)
        dev.write_sectors(52, 1, &root_dir_sec).unwrap();

        // Write cluster 3 (LBA 53) and cluster 7 (LBA 57)
        let part1 = [0x11u8; SECTOR_SIZE];
        let part2 = [0x22u8; SECTOR_SIZE];
        dev.write_sectors(53, 1, &part1).unwrap();
        dev.write_sectors(57, 1, &part2).unwrap();

        let mut fat_fs = Fat32FileSystem::mount(&mut dev, 0).unwrap();
        let file = fat_fs.lookup("FRAG.DAT").unwrap();
        assert_eq!(file.size_bytes, 1024);

        // Read across fragmentation boundary
        let mut full_buf = [0u8; 1024];
        let r = fat_fs.read(&file, 0, &mut full_buf).unwrap();
        assert_eq!(r, 1024);
        assert_eq!(&full_buf[..512], &part1[..]);
        assert_eq!(&full_buf[512..], &part2[..]);

        // Write new pattern across fragmented cluster chain
        let mut write_payload = [0x77u8; 1024];
        write_payload[512..].fill(0x88);
        let w = fat_fs.write(&file, 0, &write_payload).unwrap();
        assert_eq!(w, 1024);

        // Read back to confirm data spanned non-contiguous clusters accurately
        let mut verify_buf = [0u8; 1024];
        fat_fs.read(&file, 0, &mut verify_buf).unwrap();
        assert_eq!(&verify_buf[..], &write_payload[..]);

        // Flush block device
        assert_eq!(dev.flush(), Ok(()));
    }

    #[test]
    fn test_ext4_fail_closed_write_rejection() {
        let mut dev = MemBlockDevice::<{ 1024 * SECTOR_SIZE }>::new();

        // Construct valid Ext4 superblock at LBA 2 (1024 bytes)
        let mut raw_sb = [0u8; 1024];
        raw_sb[0x38] = 0x53; // Magic 0xEF53
        raw_sb[0x39] = 0xEF;
        raw_sb[0x00..0x04].copy_from_slice(&100u32.to_le_bytes()); // Inodes
        raw_sb[0x04..0x08].copy_from_slice(&1024u32.to_le_bytes()); // Blocks
        raw_sb[0x18..0x1C].copy_from_slice(&2u32.to_le_bytes()); // 4096B block size
        raw_sb[0x4C..0x50].copy_from_slice(&1u32.to_le_bytes());
        raw_sb[0x58..0x5A].copy_from_slice(&256u16.to_le_bytes());
        dev.write_sectors(2, 2, &raw_sb).unwrap();

        let mut ext4_fs = Ext4FileSystem::mount(&mut dev, 0).unwrap();
        assert_eq!(ext4_fs.superblock.magic, EXT4_MAGIC);

        let root_node = ext4_fs.lookup("/").unwrap();

        // Write to root node MUST fail closed with ReadOnly error
        let payload = [0xFFu8; 128];
        assert_eq!(
            ext4_fs.write(&root_node, 0, &payload),
            Err(VfsError::ReadOnly)
        );

        // File creation MUST fail closed with ReadOnly error
        assert_eq!(
            ext4_fs.create_file(&root_node, "test.txt"),
            Err(VfsError::ReadOnly)
        );

        // Directory creation MUST fail closed with ReadOnly error
        assert_eq!(
            ext4_fs.create_dir(&root_node, "test_dir"),
            Err(VfsError::ReadOnly)
        );
    }

    #[test]
    fn test_gpt_crc_corruption_and_invalid_header_rejection() {
        let mut dev = MemBlockDevice::<{ 1024 * SECTOR_SIZE }>::new();

        let mut gpt_hdr = [0u8; SECTOR_SIZE];
        gpt_hdr[0..8].copy_from_slice(GPT_SIGNATURE);
        gpt_hdr[12..16].copy_from_slice(&92u32.to_le_bytes());
        gpt_hdr[24..32].copy_from_slice(&1u64.to_le_bytes());
        gpt_hdr[40..48].copy_from_slice(&34u64.to_le_bytes());
        gpt_hdr[48..56].copy_from_slice(&1000u64.to_le_bytes());
        gpt_hdr[72..80].copy_from_slice(&2u64.to_le_bytes());
        gpt_hdr[80..84].copy_from_slice(&32u32.to_le_bytes());
        gpt_hdr[84..88].copy_from_slice(&128u32.to_le_bytes());

        // Put a bogus non-matching CRC
        gpt_hdr[16..20].copy_from_slice(&0xDEADBEEFu32.to_le_bytes());
        dev.write_sectors(1, 1, &gpt_hdr).unwrap();

        // Must reject corrupted GPT CRC
        assert_eq!(
            PartitionTable::parse(&mut dev).err(),
            Some(StorageError::CorruptedMetadata)
        );

        // Test invalid usable LBA bounds (first > last)
        let computed = partition::crc32(&gpt_hdr[..92]);
        gpt_hdr[16..20].copy_from_slice(&computed.to_le_bytes());
        gpt_hdr[40..48].copy_from_slice(&9999u64.to_le_bytes()); // first > last (out of bounds)
        let computed_bad = partition::crc32(&gpt_hdr[..92]);
        gpt_hdr[16..20].copy_from_slice(&computed_bad.to_le_bytes());
        dev.write_sectors(1, 1, &gpt_hdr).unwrap();

        assert_eq!(
            PartitionTable::parse(&mut dev).err(),
            Some(StorageError::CorruptedMetadata)
        );
    }

    #[test]
    fn test_ata_timeout_and_error_handling() {
        let driver = AtaPioDriver::new(AtaChannel::Primary, false);
        // On host test environment (without bare metal I/O port mapping or device),
        // timeout or busy poll will fail safely with Timeout
        assert_eq!(driver.io_base, ATA_PRIMARY_IO_BASE);
        assert_eq!(driver.ctrl_base, ATA_PRIMARY_CTRL_BASE);
        assert_eq!(driver.device_type, AtaDeviceType::None);
        assert_eq!(driver.total_sectors(), 0);
    }

    #[test]
    fn test_vfs_missing_device_and_unmounted_safety() {
        let mut vfs = Vfs::new();
        assert_eq!(vfs.is_mounted(), false);

        // Accessing unmounted VFS must return NotFound without panicking
        assert_eq!(vfs.open("/"), Err(VfsError::NotFound));

        let dummy_node = VfsNode {
            node_type: VfsNodeType::File,
            size_bytes: 10,
            inode: 1,
        };
        let mut buf = [0u8; 10];
        assert_eq!(vfs.read(&dummy_node, 0, &mut buf), Err(VfsError::NotFound));
        assert_eq!(vfs.write(&dummy_node, 0, &buf), Err(VfsError::NotFound));
    }

    #[test]
    fn test_raid0_creation_validation_and_rejections() {
        let mut d1 = MemBlockDevice::<{ 64 * SECTOR_SIZE }>::new();
        let mut d2 = MemBlockDevice::<{ 64 * SECTOR_SIZE }>::new();
        let mut d3 = MemBlockDevice::<{ 16 * SECTOR_SIZE }>::new();
        let mut d4 = MemBlockDevice::<{ 64 * SECTOR_SIZE }>::new();

        // 1. Reject less than 2 members
        let members1: [&mut dyn BlockDevice; MAX_RAID_MEMBERS] =
            [&mut d1, &mut d2, &mut d3, &mut d4];
        assert_eq!(
            Raid0Volume::new(members1, 1, 16).err(),
            Some(StorageError::InvalidParameter)
        );

        // 2. Reject zero or non-power-of-two stripe size
        let mut d1_b = MemBlockDevice::<{ 64 * SECTOR_SIZE }>::new();
        let mut d2_b = MemBlockDevice::<{ 64 * SECTOR_SIZE }>::new();
        let mut d3_b = MemBlockDevice::<{ 16 * SECTOR_SIZE }>::new();
        let mut d4_b = MemBlockDevice::<{ 64 * SECTOR_SIZE }>::new();
        let members2: [&mut dyn BlockDevice; MAX_RAID_MEMBERS] =
            [&mut d1_b, &mut d2_b, &mut d3_b, &mut d4_b];
        assert_eq!(
            Raid0Volume::new(members2, 2, 0).err(),
            Some(StorageError::InvalidParameter)
        );

        let mut d1_c = MemBlockDevice::<{ 64 * SECTOR_SIZE }>::new();
        let mut d2_c = MemBlockDevice::<{ 64 * SECTOR_SIZE }>::new();
        let mut d3_c = MemBlockDevice::<{ 16 * SECTOR_SIZE }>::new();
        let mut d4_c = MemBlockDevice::<{ 64 * SECTOR_SIZE }>::new();
        let members3: [&mut dyn BlockDevice; MAX_RAID_MEMBERS] =
            [&mut d1_c, &mut d2_c, &mut d3_c, &mut d4_c];
        assert_eq!(
            Raid0Volume::new(members3, 2, 15).err(),
            Some(StorageError::InvalidParameter)
        ); // 15 is not power of 2

        // 3. Valid creation with two members of 64 sectors and stripe size 16
        let mut d1_v = MemBlockDevice::<{ 64 * SECTOR_SIZE }>::new();
        let mut d2_v = MemBlockDevice::<{ 64 * SECTOR_SIZE }>::new();
        let mut d3_v = MemBlockDevice::<{ 16 * SECTOR_SIZE }>::new();
        let mut d4_v = MemBlockDevice::<{ 64 * SECTOR_SIZE }>::new();
        let members_valid: [&mut dyn BlockDevice; MAX_RAID_MEMBERS] =
            [&mut d1_v, &mut d2_v, &mut d3_v, &mut d4_v];
        let r0 = Raid0Volume::new(members_valid, 2, 16).unwrap();
        assert_eq!(r0.member_count(), 2);
        assert_eq!(r0.stripe_sectors(), 16);
        assert_eq!(r0.member_usable_sectors(), 64);
        assert_eq!(r0.total_sectors(), 128); // 64 * 2 = 128 sectors
    }

    #[test]
    fn test_raid0_striping_sector_mapping_across_boundaries() {
        let mut d1 = MemBlockDevice::<{ 32 * SECTOR_SIZE }>::new();
        let mut d2 = MemBlockDevice::<{ 32 * SECTOR_SIZE }>::new();
        let mut d3 = MemBlockDevice::<{ 16 * SECTOR_SIZE }>::new();
        let mut d4 = MemBlockDevice::<{ 16 * SECTOR_SIZE }>::new();

        // 2 members, stripe size = 4 sectors (2048 bytes per stripe chunk)
        let members: [&mut dyn BlockDevice; MAX_RAID_MEMBERS] =
            [&mut d1, &mut d2, &mut d3, &mut d4];
        let mut r0 = Raid0Volume::new(members, 2, 4).unwrap();
        assert_eq!(r0.total_sectors(), 64);

        // Fill pattern spanning 3 stripes (10 sectors: LBA 2..12)
        // LBA 2..4: in stripe 0 (on disk 0, offset 2..4) -> 2 sectors
        // LBA 4..8: in stripe 1 (on disk 1, offset 0..4) -> 4 sectors
        // LBA 8..12: in stripe 2 (on disk 0, offset 4..8) -> 4 sectors
        let mut write_data = [0u8; 10 * SECTOR_SIZE];
        for i in 0..write_data.len() {
            write_data[i] = (i & 0xFF) as u8;
        }

        r0.write_sectors(2, 10, &write_data).unwrap();

        // Read back via RAID 0 and verify
        let mut read_data = [0u8; 10 * SECTOR_SIZE];
        r0.read_sectors(2, 10, &mut read_data).unwrap();
        assert_eq!(read_data, write_data);

        // Verify out-of-bounds LBA rejected
        let mut err_buf = [0u8; 512];
        assert_eq!(
            r0.read_sectors(64, 1, &mut err_buf),
            Err(StorageError::InvalidLba)
        );
        let err_buf_5 = [0u8; 5 * 512];
        assert_eq!(
            r0.write_sectors(60, 5, &err_buf_5),
            Err(StorageError::InvalidLba)
        );
    }

    #[test]
    fn test_raid0_member_failure_propagation() {
        let mut d1 = MemBlockDevice::<{ 32 * SECTOR_SIZE }>::new();
        let mut d2 = MemBlockDevice::<{ 32 * SECTOR_SIZE }>::new();
        let mut d3 = MemBlockDevice::<{ 16 * SECTOR_SIZE }>::new();
        let mut d4 = MemBlockDevice::<{ 16 * SECTOR_SIZE }>::new();

        // Inject error on d2
        d2.set_inject_error(true);

        let members: [&mut dyn BlockDevice; MAX_RAID_MEMBERS] =
            [&mut d1, &mut d2, &mut d3, &mut d4];
        let mut r0 = Raid0Volume::new(members, 2, 4).unwrap();

        let mut buf = [0u8; 8 * SECTOR_SIZE]; // Spans across both d1 and d2
                                              // Must fail with IoError when accessing d2
        assert_eq!(r0.read_sectors(0, 8, &mut buf), Err(StorageError::IoError));
        assert_eq!(r0.write_sectors(0, 8, &buf), Err(StorageError::IoError));
    }

    #[test]
    fn test_raid1_mirror_write_readback_and_consistency() {
        let mut d1 = MemBlockDevice::<{ 64 * SECTOR_SIZE }>::new();
        let mut d2 = MemBlockDevice::<{ 64 * SECTOR_SIZE }>::new();
        let mut d3 = MemBlockDevice::<{ 16 * SECTOR_SIZE }>::new();
        let mut d4 = MemBlockDevice::<{ 16 * SECTOR_SIZE }>::new();

        let members: [&mut dyn BlockDevice; MAX_RAID_MEMBERS] =
            [&mut d1, &mut d2, &mut d3, &mut d4];
        let mut r1 = Raid1Volume::new(members, 2).unwrap();
        assert_eq!(r1.member_count(), 2);
        assert_eq!(r1.total_sectors(), 64);
        assert_eq!(r1.volume_state(), Raid1VolumeState::Optimal);

        let write_buf = [0x5Au8; 512];
        r1.write_sectors(10, 1, &write_buf).unwrap();

        // Read back through RAID 1 volume
        let mut read_buf = [0u8; 512];
        r1.read_sectors(10, 1, &mut read_buf).unwrap();
        assert_eq!(read_buf, write_buf);

        // Verify that BOTH underlying physical members received the exact identical data
        let mut d1_check = [0u8; 512];
        let mut d2_check = [0u8; 512];
        r1.members[0].read_sectors(10, 1, &mut d1_check).unwrap();
        r1.members[1].read_sectors(10, 1, &mut d2_check).unwrap();
        assert_eq!(d1_check, write_buf);
        assert_eq!(d2_check, write_buf);
    }

    #[test]
    fn test_raid1_degraded_mode_read_fallback_and_no_false_success() {
        let mut d1 = MemBlockDevice::<{ 64 * SECTOR_SIZE }>::new();
        let mut d2 = MemBlockDevice::<{ 64 * SECTOR_SIZE }>::new();
        let mut d3 = MemBlockDevice::<{ 16 * SECTOR_SIZE }>::new();
        let mut d4 = MemBlockDevice::<{ 16 * SECTOR_SIZE }>::new();

        let members: [&mut dyn BlockDevice; MAX_RAID_MEMBERS] =
            [&mut d1, &mut d2, &mut d3, &mut d4];
        let mut r1 = Raid1Volume::new(members, 2).unwrap();

        // Write initial data while optimal
        let initial_data = [0xC3u8; 512];
        r1.write_sectors(5, 1, &initial_data).unwrap();

        // Inject fault on primary member d1
        r1.members[0].set_inject_error(true);

        // Read operation must fallback to d2 and mark d1 as Failed
        let mut read_fallback = [0u8; 512];
        assert_eq!(r1.read_sectors(5, 1, &mut read_fallback), Ok(()));
        assert_eq!(read_fallback, initial_data);
        assert_eq!(r1.member_status(0), Some(Raid1MemberStatus::Failed));
        assert_eq!(r1.member_status(1), Some(Raid1MemberStatus::Healthy));
        assert_eq!(r1.volume_state(), Raid1VolumeState::Degraded);

        // Subsequent write in Degraded mode will only attempt to healthy members (d2), succeeding
        let new_data = [0xE5u8; 512];
        assert_eq!(r1.write_sectors(5, 1, &new_data), Ok(()));

        // Read back confirms d2 has updated data
        let mut read_degraded = [0u8; 512];
        assert_eq!(r1.read_sectors(5, 1, &mut read_degraded), Ok(()));
        assert_eq!(read_degraded, new_data);

        // Now inject fault on d2 as well -> All members offline
        r1.members[1].set_inject_error(true);
        assert_eq!(
            r1.read_sectors(5, 1, &mut read_degraded),
            Err(StorageError::IoError)
        );
        assert_eq!(r1.volume_state(), Raid1VolumeState::Offline);
        assert_eq!(
            r1.write_sectors(5, 1, &new_data),
            Err(StorageError::DeviceNotFound)
        );
    }

    #[test]
    fn test_raid1_partial_write_failure_policy() {
        let mut d1 = MemBlockDevice::<{ 64 * SECTOR_SIZE }>::new();
        let mut d2 = MemBlockDevice::<{ 64 * SECTOR_SIZE }>::new();
        let mut d3 = MemBlockDevice::<{ 16 * SECTOR_SIZE }>::new();
        let mut d4 = MemBlockDevice::<{ 16 * SECTOR_SIZE }>::new();

        let members: [&mut dyn BlockDevice; MAX_RAID_MEMBERS] =
            [&mut d1, &mut d2, &mut d3, &mut d4];
        let mut r1 = Raid1Volume::new(members, 2).unwrap();

        // Inject error on member 1 only
        r1.members[1].set_inject_error(true);

        // Attempting to write while both are marked healthy must FAIL with IoError
        // and must NOT return false-success Ok(())!
        let buf = [0x99u8; 512];
        assert_eq!(r1.write_sectors(0, 1, &buf), Err(StorageError::IoError));

        // Member 1 must now be marked Failed, transitioning volume to Degraded
        assert_eq!(r1.member_status(1), Some(Raid1MemberStatus::Failed));
        assert_eq!(r1.volume_state(), Raid1VolumeState::Degraded);
    }

    #[test]
    fn test_raid1_flush_failure_propagation() {
        let mut d1 = MemBlockDevice::<{ 64 * SECTOR_SIZE }>::new();
        let mut d2 = MemBlockDevice::<{ 64 * SECTOR_SIZE }>::new();
        let mut d3 = MemBlockDevice::<{ 16 * SECTOR_SIZE }>::new();
        let mut d4 = MemBlockDevice::<{ 16 * SECTOR_SIZE }>::new();

        let members: [&mut dyn BlockDevice; MAX_RAID_MEMBERS] =
            [&mut d1, &mut d2, &mut d3, &mut d4];
        let mut r1 = Raid1Volume::new(members, 2).unwrap();

        // Normal flush succeeds
        assert_eq!(r1.flush(), Ok(()));

        // Inject fault on member 0
        r1.members[0].set_inject_error(true);
        // Flush must propagate error and mark member 0 Failed
        assert_eq!(r1.flush(), Err(StorageError::IoError));
        assert_eq!(r1.member_status(0), Some(Raid1MemberStatus::Failed));
        assert_eq!(r1.volume_state(), Raid1VolumeState::Degraded);
    }

    #[test]
    fn test_vfs_mounted_on_raid1_volume() {
        let mut d1 = MemBlockDevice::<{ 256 * SECTOR_SIZE }>::new();
        let mut d2 = MemBlockDevice::<{ 256 * SECTOR_SIZE }>::new();
        let mut d3 = MemBlockDevice::<{ 16 * SECTOR_SIZE }>::new();
        let mut d4 = MemBlockDevice::<{ 16 * SECTOR_SIZE }>::new();

        let members: [&mut dyn BlockDevice; MAX_RAID_MEMBERS] =
            [&mut d1, &mut d2, &mut d3, &mut d4];
        let mut r1 = Raid1Volume::new(members, 2).unwrap();

        // Write FAT32 BPB on RAID 1 volume
        let mut bpb_sec = [0u8; SECTOR_SIZE];
        bpb_sec[510] = 0x55;
        bpb_sec[511] = 0xAA;
        bpb_sec[11..13].copy_from_slice(&512u16.to_le_bytes());
        bpb_sec[13] = 1;
        bpb_sec[14..16].copy_from_slice(&32u16.to_le_bytes());
        bpb_sec[16] = 2;
        bpb_sec[36..40].copy_from_slice(&10u32.to_le_bytes());
        bpb_sec[44..48].copy_from_slice(&2u32.to_le_bytes());
        r1.write_sectors(0, 1, &bpb_sec).unwrap();

        // Setup FAT
        let mut fat_sec = [0u8; SECTOR_SIZE];
        fat_sec[8..12].copy_from_slice(&0x0FFF_FFF8u32.to_le_bytes()); // Root dir EOC
        fat_sec[12..16].copy_from_slice(&0x0FFF_FFF8u32.to_le_bytes()); // File EOC
        r1.write_sectors(32, 1, &fat_sec).unwrap();
        r1.write_sectors(42, 1, &fat_sec).unwrap();

        // Root directory entry for "RAID1   TXT" pointing to cluster 3
        let mut root_dir_sec = [0u8; SECTOR_SIZE];
        root_dir_sec[0..11].copy_from_slice(b"RAID1   TXT");
        root_dir_sec[11] = 0x20;
        root_dir_sec[26..28].copy_from_slice(&3u16.to_le_bytes());
        root_dir_sec[28..32].copy_from_slice(&11u32.to_le_bytes()); // 11 bytes
        r1.write_sectors(52, 1, &root_dir_sec).unwrap();

        // Write payload to Cluster 3 (LBA 53)
        let mut payload_sec = [0u8; SECTOR_SIZE];
        payload_sec[..11].copy_from_slice(b"HELLO_RAID1");
        r1.write_sectors(53, 1, &payload_sec).unwrap();

        // Mount FAT32 filesystem directly on RAID 1 block device
        let mut fat_fs = Fat32FileSystem::mount(&mut r1, 0).unwrap();
        let file_node = fat_fs.lookup("RAID1.TXT").unwrap();
        assert_eq!(file_node.size_bytes, 11);

        let mut read_buf = [0u8; 11];
        let bytes_read = fat_fs.read(&file_node, 0, &mut read_buf).unwrap();
        assert_eq!(bytes_read, 11);
        assert_eq!(&read_buf, b"HELLO_RAID1");

        // Verify that VFS can manage the mounted FAT32 filesystem
        let mut vfs = Vfs::new();
        vfs.mount_root(&mut fat_fs);
        assert_eq!(vfs.is_mounted(), true);
        let vfs_node = vfs.open("RAID1.TXT").unwrap();
        let mut vfs_buf = [0u8; 11];
        let vfs_read = vfs.read(&vfs_node, 0, &mut vfs_buf).unwrap();
        assert_eq!(vfs_read, 11);
        assert_eq!(&vfs_buf, b"HELLO_RAID1");
    }

    #[test]
    fn test_raid1_member0_failure_behavior_symmetric() {
        let mut d1 = MemBlockDevice::<{ 64 * SECTOR_SIZE }>::new();
        let mut d2 = MemBlockDevice::<{ 64 * SECTOR_SIZE }>::new();
        let mut d3 = MemBlockDevice::<{ 16 * SECTOR_SIZE }>::new();
        let mut d4 = MemBlockDevice::<{ 16 * SECTOR_SIZE }>::new();

        let members: [&mut dyn BlockDevice; MAX_RAID_MEMBERS] =
            [&mut d1, &mut d2, &mut d3, &mut d4];
        let mut r1 = Raid1Volume::new(members, 2).unwrap();

        // Inject fault on member 0 (the primary member)
        r1.members[0].set_inject_error(true);

        // Writing while both are healthy should fail on member 0, propagate error,
        // and transition volume to Degraded with member 0 marked Failed
        let buf = [0x77u8; 512];
        assert_eq!(r1.write_sectors(0, 1, &buf), Err(StorageError::IoError));
        assert_eq!(r1.member_status(0), Some(Raid1MemberStatus::Failed));
        assert_eq!(r1.member_status(1), Some(Raid1MemberStatus::Healthy));
        assert_eq!(r1.volume_state(), Raid1VolumeState::Degraded);

        // Subsequent read must read from healthy member 1 without panicking or failing
        let mut read_buf = [0u8; 512];
        assert_eq!(r1.read_sectors(0, 1, &mut read_buf), Ok(()));
    }

    #[test]
    fn test_raid_boundary_heterogeneous_capacity_and_zero_sector_ops() {
        // Heterogeneous capacities: 64 sectors and 128 sectors
        let mut d1 = MemBlockDevice::<{ 64 * SECTOR_SIZE }>::new();
        let mut d2 = MemBlockDevice::<{ 128 * SECTOR_SIZE }>::new();
        let mut d3 = MemBlockDevice::<{ 16 * SECTOR_SIZE }>::new();
        let mut d4 = MemBlockDevice::<{ 16 * SECTOR_SIZE }>::new();

        let members_r0: [&mut dyn BlockDevice; MAX_RAID_MEMBERS] =
            [&mut d1, &mut d2, &mut d3, &mut d4];
        // In RAID 0, capacity is limited by minimum member capacity (64) aligned to stripe size (16)
        // 64 * 2 = 128 sectors
        let mut r0 = Raid0Volume::new(members_r0, 2, 16).unwrap();
        assert_eq!(r0.total_sectors(), 128);

        // Zero-sector read and write must be safe no-ops
        let mut dummy = [0u8; 512];
        assert_eq!(r0.read_sectors(0, 0, &mut dummy), Ok(()));
        assert_eq!(r0.write_sectors(0, 0, &dummy), Ok(()));

        // Buffer too small check
        let mut small = [0u8; 256];
        assert_eq!(
            r0.read_sectors(0, 1, &mut small),
            Err(StorageError::BufferTooSmall)
        );
        assert_eq!(
            r0.write_sectors(0, 1, &small),
            Err(StorageError::BufferTooSmall)
        );

        // RAID 1 heterogeneous capacity test
        let mut d1_m = MemBlockDevice::<{ 64 * SECTOR_SIZE }>::new();
        let mut d2_m = MemBlockDevice::<{ 128 * SECTOR_SIZE }>::new();
        let mut d3_m = MemBlockDevice::<{ 16 * SECTOR_SIZE }>::new();
        let mut d4_m = MemBlockDevice::<{ 16 * SECTOR_SIZE }>::new();
        let members_r1: [&mut dyn BlockDevice; MAX_RAID_MEMBERS] =
            [&mut d1_m, &mut d2_m, &mut d3_m, &mut d4_m];
        let mut r1 = Raid1Volume::new(members_r1, 2).unwrap();
        // RAID 1 capacity must be min(64, 128) = 64 sectors
        assert_eq!(r1.total_sectors(), 64);

        // Zero sector read and write no-op on RAID 1
        assert_eq!(r1.read_sectors(0, 0, &mut dummy), Ok(()));
        assert_eq!(r1.write_sectors(0, 0, &dummy), Ok(()));

        // Out of bounds on RAID 1
        assert_eq!(
            r1.read_sectors(64, 1, &mut dummy),
            Err(StorageError::InvalidLba)
        );
        assert_eq!(
            r1.write_sectors(64, 1, &dummy),
            Err(StorageError::InvalidLba)
        );
    }

    #[test]
    fn test_raid_metadata_crc32_known_answer_vector() {
        // Known-answer test (KAT) for IEEE 802.3 CRC32:
        // ASCII string b"123456789" CRC32 MUST be 0xCBF43926
        let kat_input = b"123456789";
        let computed = partition::crc32(kat_input);
        assert_eq!(computed, 0xCBF4_3926);
    }

    #[test]
    fn test_raid_metadata_roundtrip_raid0_and_raid1() {
        // Test RAID 0 metadata roundtrip
        let sb_r0 = RaidSuperblock {
            version: RAID_SUPERBLOCK_VERSION_1,
            raid_type: RaidType::Raid0,
            generation: 42,
            array_uuid: [0x11; 16],
            member_uuid: [0x22; 16],
            member_index: 0,
            member_count: 2,
            stripe_sectors: 16,
            array_capacity_sectors: 2048,
            member_usable_sectors: 1024,
            data_start_lba: 8,
            state: RaidMetadataState::ActiveClean,
        };

        let mut buf_r0 = [0u8; SECTOR_SIZE];
        sb_r0.encode(&mut buf_r0).unwrap();
        let decoded_r0 = RaidSuperblock::decode(&buf_r0).unwrap();
        assert_eq!(decoded_r0, sb_r0);

        // Test RAID 1 metadata roundtrip
        let sb_r1 = RaidSuperblock {
            version: RAID_SUPERBLOCK_VERSION_1,
            raid_type: RaidType::Raid1,
            generation: 100,
            array_uuid: [0xAA; 16],
            member_uuid: [0xBB; 16],
            member_index: 1,
            member_count: 2,
            stripe_sectors: 0,
            array_capacity_sectors: 512,
            member_usable_sectors: 512,
            data_start_lba: 8,
            state: RaidMetadataState::Degraded,
        };

        let mut buf_r1 = [0u8; SECTOR_SIZE];
        sb_r1.encode(&mut buf_r1).unwrap();
        let decoded_r1 = RaidSuperblock::decode(&buf_r1).unwrap();
        assert_eq!(decoded_r1, sb_r1);
    }

    #[test]
    fn test_raid_metadata_corruption_and_rejections() {
        let sb = RaidSuperblock {
            version: RAID_SUPERBLOCK_VERSION_1,
            raid_type: RaidType::Raid0,
            generation: 1,
            array_uuid: [0x01; 16],
            member_uuid: [0x02; 16],
            member_index: 0,
            member_count: 2,
            stripe_sectors: 16,
            array_capacity_sectors: 200,
            member_usable_sectors: 100,
            data_start_lba: 8,
            state: RaidMetadataState::ActiveClean,
        };

        let mut valid_buf = [0u8; SECTOR_SIZE];
        sb.encode(&mut valid_buf).unwrap();

        // 1. Corrupt magic
        let mut bad_magic = valid_buf;
        bad_magic[0] = b'X';
        assert_eq!(
            RaidSuperblock::decode(&bad_magic).err(),
            Some(StorageError::CorruptedMetadata)
        );

        // 2. Corrupt CRC32
        let mut bad_crc = valid_buf;
        bad_crc[16] ^= 0xFF;
        assert_eq!(
            RaidSuperblock::decode(&bad_crc).err(),
            Some(StorageError::CorruptedMetadata)
        );

        // 3. Unsupported version (recompute CRC so version check fails)
        let mut bad_ver = valid_buf;
        bad_ver[8] = 99;
        bad_ver[16..20].fill(0);
        let new_crc = partition::crc32(&bad_ver);
        bad_ver[16..20].copy_from_slice(&new_crc.to_le_bytes());
        assert_eq!(
            RaidSuperblock::decode(&bad_ver).err(),
            Some(StorageError::UnsupportedFeature)
        );

        // 4. Non-zero reserved bytes rejection
        let mut bad_reserved = valid_buf;
        bad_reserved[100] = 0xAA;
        bad_reserved[16..20].fill(0);
        let r_crc = partition::crc32(&bad_reserved);
        bad_reserved[16..20].copy_from_slice(&r_crc.to_le_bytes());
        assert_eq!(
            RaidSuperblock::decode(&bad_reserved).err(),
            Some(StorageError::CorruptedMetadata)
        );

        // 5. Invalid member count (< 2)
        let mut bad_count = valid_buf;
        bad_count[64..68].copy_from_slice(&1u32.to_le_bytes());
        bad_count[16..20].fill(0);
        let c_crc = partition::crc32(&bad_count);
        bad_count[16..20].copy_from_slice(&c_crc.to_le_bytes());
        assert_eq!(
            RaidSuperblock::decode(&bad_count).err(),
            Some(StorageError::CorruptedMetadata)
        );

        // 6. Invalid member index (>= member count)
        let mut bad_idx = valid_buf;
        bad_idx[60..64].copy_from_slice(&2u32.to_le_bytes()); // Index 2 on count 2
        bad_idx[16..20].fill(0);
        let i_crc = partition::crc32(&bad_idx);
        bad_idx[16..20].copy_from_slice(&i_crc.to_le_bytes());
        assert_eq!(
            RaidSuperblock::decode(&bad_idx).err(),
            Some(StorageError::CorruptedMetadata)
        );

        // 7. All-zero UUID rejection
        let mut zero_uuid = valid_buf;
        zero_uuid[28..44].fill(0);
        zero_uuid[16..20].fill(0);
        let z_crc = partition::crc32(&zero_uuid);
        zero_uuid[16..20].copy_from_slice(&z_crc.to_le_bytes());
        assert_eq!(
            RaidSuperblock::decode(&zero_uuid).err(),
            Some(StorageError::CorruptedMetadata)
        );
    }

    #[test]
    fn test_raid_metadata_generation_counter_wraparound() {
        // Equal
        assert_eq!(
            RaidSuperblock::compare_generation(10, 10),
            GenerationOrder::Equal
        );

        // Simple progression
        assert_eq!(
            RaidSuperblock::compare_generation(11, 10),
            GenerationOrder::Newer
        );
        assert_eq!(
            RaidSuperblock::compare_generation(10, 11),
            GenerationOrder::Older
        );

        // Wraparound edge: 5 is newer than u64::MAX
        assert_eq!(
            RaidSuperblock::compare_generation(5, u64::MAX),
            GenerationOrder::Newer
        );
        assert_eq!(
            RaidSuperblock::compare_generation(u64::MAX, 5),
            GenerationOrder::Older
        );

        // Ambiguous midpoint (distance exactly 2^63)
        let midpoint_a = 0u64;
        let midpoint_b = 1u64 << 63;
        assert_eq!(
            RaidSuperblock::compare_generation(midpoint_a, midpoint_b),
            GenerationOrder::Ambiguous
        );
    }

    #[test]
    fn test_raid_metadata_peer_compatibility() {
        let member0 = RaidSuperblock {
            version: RAID_SUPERBLOCK_VERSION_1,
            raid_type: RaidType::Raid1,
            generation: 1,
            array_uuid: [0x55; 16],
            member_uuid: [0x01; 16],
            member_index: 0,
            member_count: 2,
            stripe_sectors: 0,
            array_capacity_sectors: 1000,
            member_usable_sectors: 1000,
            data_start_lba: 2048,
            state: RaidMetadataState::ActiveClean,
        };

        let mut member1 = member0;
        member1.member_uuid = [0x02; 16];
        member1.member_index = 1;

        // Compatible peers
        assert_eq!(member0.validate_peer_compatibility(&member1), Ok(()));

        // Incompatible: duplicate member index
        let mut dup_member = member1;
        dup_member.member_index = 0;
        assert_eq!(
            member0.validate_peer_compatibility(&dup_member),
            Err(StorageError::AlreadyExists)
        );

        // Incompatible: different array UUID
        let mut diff_array = member1;
        diff_array.array_uuid = [0x99; 16];
        assert_eq!(
            member0.validate_peer_compatibility(&diff_array),
            Err(StorageError::CorruptedMetadata)
        );
    }

    #[test]
    fn test_raid_metadata_reservation_and_lba0_protection() {
        // Rule: partition_start_lba == 0 MUST be strictly rejected (LBA 0 is MBR / Protective MBR)
        assert_eq!(
            DedicatedPartitionReservation::new(0, 1000, 1, 8),
            Err(StorageError::InvalidParameter)
        );

        // Rule: zero sector count partition rejected
        assert_eq!(
            DedicatedPartitionReservation::new(2048, 0, 1, 8),
            Err(StorageError::InvalidParameter)
        );

        // Rule: metadata sector overlapping with data area rejected
        assert_eq!(
            DedicatedPartitionReservation::new(2048, 1000, 5, 5),
            Err(StorageError::InvalidParameter)
        );

        // Rule: data offset exceeding partition sector count rejected
        assert_eq!(
            DedicatedPartitionReservation::new(2048, 100, 1, 100),
            Err(StorageError::InvalidParameter)
        );

        // Rule: arithmetic overflow protection on partition bounds
        assert_eq!(
            DedicatedPartitionReservation::new(u64::MAX - 10, 20, 1, 8),
            Err(StorageError::InvalidParameter)
        );

        // Valid reservation: Dedicated partition starting at LBA 64, 100 sectors
        // metadata at offset 1 (LBA 65), data starts at offset 8 (LBA 72)
        let reservation = DedicatedPartitionReservation::new(64, 100, 1, 8).unwrap();
        assert_eq!(reservation.absolute_metadata_lba().unwrap(), 65);
        assert_eq!(reservation.absolute_data_start_lba().unwrap(), 72);
        assert_eq!(reservation.usable_member_sectors(), 92);

        // Set up test disk device (256 sectors = 128KB on stack)
        let mut disk = MemBlockDevice::<{ 256 * SECTOR_SIZE }>::new();

        // Populate LBA 0 (MBR) and LBA 1..34 (GPT header and partition entries) with mock bytes
        let mbr_pattern = [0x5Au8; SECTOR_SIZE];
        let gpt_pattern = [0xA5u8; SECTOR_SIZE];
        disk.write_sectors(0, 1, &mbr_pattern).unwrap();
        disk.write_sectors(1, 1, &gpt_pattern).unwrap();

        // Create initial disk snapshot
        let mut snapshot_lba0 = [0u8; SECTOR_SIZE];
        disk.read_sectors(0, 1, &mut snapshot_lba0).unwrap();
        assert_eq!(snapshot_lba0, mbr_pattern);

        let mut snapshot_lba1 = [0u8; SECTOR_SIZE];
        disk.read_sectors(1, 1, &mut snapshot_lba1).unwrap();
        assert_eq!(snapshot_lba1, gpt_pattern);

        // Superblock matching the explicit reservation
        let sb = RaidSuperblock {
            version: RAID_SUPERBLOCK_VERSION_1,
            raid_type: RaidType::Raid1,
            generation: 1,
            array_uuid: [0x42; 16],
            member_uuid: [0x11; 16],
            member_index: 0,
            member_count: 2,
            stripe_sectors: 0,
            array_capacity_sectors: 92,
            member_usable_sectors: 92,
            data_start_lba: 72,
            state: RaidMetadataState::ActiveClean,
        };

        // Write superblock to device using explicit reservation
        assert_eq!(sb.write_to_device(&mut disk, &reservation), Ok(()));

        // CRITICAL INVARIANCE CHECK: Verify LBA 0 and LBA 1 are 100% UNTOUCHED
        let mut check_lba0 = [0u8; SECTOR_SIZE];
        disk.read_sectors(0, 1, &mut check_lba0).unwrap();
        assert_eq!(check_lba0, mbr_pattern);

        let mut check_lba1 = [0u8; SECTOR_SIZE];
        disk.read_sectors(1, 1, &mut check_lba1).unwrap();
        assert_eq!(check_lba1, gpt_pattern);

        // Verify readback matches superblock exactly
        let readback = RaidSuperblock::read_from_device(&mut disk, &reservation).unwrap();
        assert_eq!(readback, sb);

        // Fail-Closed Check 1: Insufficient disk capacity fails closed
        let mut small_disk = MemBlockDevice::<{ 16 * SECTOR_SIZE }>::new(); // Only 16 sectors, needs 164
        assert_eq!(
            sb.write_to_device(&mut small_disk, &reservation),
            Err(StorageError::InvalidParameter)
        );

        // Fail-Closed Check 2: Mismatched data_start_lba in superblock vs reservation
        let mut mismatched_sb = sb;
        mismatched_sb.data_start_lba = 70; // Expected 72
        assert_eq!(
            mismatched_sb.write_to_device(&mut disk, &reservation),
            Err(StorageError::InvalidParameter)
        );

        // Fail-Closed Check 3: Error injection propagation on write
        disk.set_inject_error(true);
        assert_eq!(
            sb.write_to_device(&mut disk, &reservation),
            Err(StorageError::IoError)
        );
        assert_eq!(
            RaidSuperblock::read_from_device(&mut disk, &reservation),
            Err(StorageError::IoError)
        );
        disk.set_inject_error(false);

        // Fail-Closed Check 4: Corrupted metadata on disk fails closed
        let corrupt_data = [0xFFu8; SECTOR_SIZE];
        let meta_lba = reservation.absolute_metadata_lba().unwrap();
        disk.write_sectors(meta_lba, 1, &corrupt_data).unwrap();
        assert_eq!(
            RaidSuperblock::read_from_device(&mut disk, &reservation),
            Err(StorageError::CorruptedMetadata)
        );

        // Fail-Closed Check 5: from_verified_partition integration and full sector invariance
        let mut verified_disk = MemBlockDevice::<{ 256 * SECTOR_SIZE }>::new();
        // Fill all sectors with unique known byte patterns to verify strict invariance
        for s in 0..256u64 {
            let pattern = [(s & 0xFF) as u8; SECTOR_SIZE];
            verified_disk.write_sectors(s, 1, &pattern).unwrap();
        }

        // Negative Test 1: Reject Empty partition
        let empty_part = Partition {
            part_num: 1,
            part_type: PartitionType::Empty,
            start_lba: 100,
            sector_count: 50,
            bootable: false,
        };
        assert_eq!(
            DedicatedPartitionReservation::from_verified_partition(&empty_part, 1, 4),
            Err(StorageError::InvalidParameter)
        );

        // Negative Test 2: Reject EFI System partition (protecting bootloader)
        let efi_part = Partition {
            part_num: 1,
            part_type: PartitionType::EfiSystem,
            start_lba: 100,
            sector_count: 50,
            bootable: false,
        };
        assert_eq!(
            DedicatedPartitionReservation::from_verified_partition(&efi_part, 1, 4),
            Err(StorageError::InvalidParameter)
        );

        // Negative Test 3: Reject non-RAID data partitions (Fat32, LinuxNative, GptGenericData)
        // to prevent accidental overwrite of existing user filesystems!
        let fat_part = Partition {
            part_num: 2,
            part_type: PartitionType::Fat32,
            start_lba: 100,
            sector_count: 50,
            bootable: false,
        };
        assert_eq!(
            DedicatedPartitionReservation::from_verified_partition(&fat_part, 1, 4),
            Err(StorageError::InvalidParameter)
        );

        let data_part = Partition {
            part_num: 3,
            part_type: PartitionType::LinuxNative,
            start_lba: 100,
            sector_count: 50,
            bootable: false,
        };
        assert_eq!(
            DedicatedPartitionReservation::from_verified_partition(&data_part, 1, 4),
            Err(StorageError::InvalidParameter)
        );

        let gpt_generic_part = Partition {
            part_num: 4,
            part_type: PartitionType::GptGenericData,
            start_lba: 100,
            sector_count: 50,
            bootable: false,
        };
        assert_eq!(
            DedicatedPartitionReservation::from_verified_partition(&gpt_generic_part, 1, 4),
            Err(StorageError::InvalidParameter)
        );

        // Positive Test: Authorize dedicated RaidMember partition
        let raid_member_part = Partition {
            part_num: 1,
            part_type: PartitionType::RaidMember,
            start_lba: 100,
            sector_count: 50,
            bootable: false,
        };
        let verified_res = DedicatedPartitionReservation::from_verified_partition(
            &raid_member_part,
            1, // metadata at LBA 101
            4, // data at LBA 104
        )
        .unwrap();
        assert_eq!(verified_res.absolute_metadata_lba().unwrap(), 101);
        assert_eq!(verified_res.absolute_data_start_lba().unwrap(), 104);

        let v_sb = RaidSuperblock {
            version: RAID_SUPERBLOCK_VERSION_1,
            raid_type: RaidType::Raid1,
            generation: 1,
            array_uuid: [0x77; 16],
            member_uuid: [0x88; 16],
            member_index: 0,
            member_count: 2,
            stripe_sectors: 0,
            array_capacity_sectors: 46,
            member_usable_sectors: 46,
            data_start_lba: 104,
            state: RaidMetadataState::ActiveClean,
        };

        // Write superblock
        assert_eq!(
            v_sb.write_to_device(&mut verified_disk, &verified_res),
            Ok(())
        );

        // INVARIANCE VERIFICATION: Every single sector on disk EXCEPT LBA 101 MUST remain 100% unchanged!
        for s in 0..256u64 {
            let mut sec = [0u8; SECTOR_SIZE];
            verified_disk.read_sectors(s, 1, &mut sec).unwrap();
            let expected_byte = (s & 0xFF) as u8;
            if s == 101 {
                // Sektor 101 adalah target metadata yang ditulis
                assert_eq!(&sec[0..8], RAID_SUPERBLOCK_MAGIC);
            } else {
                // Seluruh sektor lain (termasuk LBA 0 MBR, LBA 1..34 GPT, dan data) TIDAK TERUBAH
                assert!(
                    sec.iter().all(|&b| b == expected_byte),
                    "Sektor {} tercemar!",
                    s
                );
            }
        }

        // Test RaidInitializationPolicy collision protection
        // Attempting to initialize over an already valid array member with CreateNewArraySafe MUST fail closed!
        assert_eq!(
            v_sb.initialize_member(
                &mut verified_disk,
                &verified_res,
                RaidInitializationPolicy::CreateNewArraySafe
            ),
            Err(StorageError::AlreadyExists)
        );

        // Operational Update Tests:
        // 1. Generation progression enforced: updating with equal generation fails closed
        assert_eq!(
            v_sb.update_operational_member(&mut verified_disk, &verified_res),
            Err(StorageError::InvalidParameter)
        );

        // 2. Mismatched Array UUID rejected
        let mut alien_sb = v_sb;
        alien_sb.array_uuid = [0x99; 16];
        alien_sb.generation = 2;
        assert_eq!(
            alien_sb.update_operational_member(&mut verified_disk, &verified_res),
            Err(StorageError::CorruptedMetadata)
        );

        // 3. Valid operational update (newer generation) succeeds with verified readback
        let mut updated_sb = v_sb;
        updated_sb.generation = 2;
        updated_sb.state = RaidMetadataState::Degraded;
        assert_eq!(
            updated_sb.update_operational_member(&mut verified_disk, &verified_res),
            Ok(())
        );
        let readback = RaidSuperblock::read_from_device(&mut verified_disk, &verified_res).unwrap();
        assert_eq!(readback, updated_sb);

        // 4. Fault Injection during Operational Update:
        // Error injected on write must fail closed and NOT report success
        verified_disk.set_inject_error(true);
        let mut next_sb = updated_sb;
        next_sb.generation = 3;
        assert_eq!(
            next_sb.update_operational_member(&mut verified_disk, &verified_res),
            Err(StorageError::IoError)
        );
        verified_disk.set_inject_error(false);
    }

    #[test]
    fn test_raid_metadata_operational_update_fault_injection_and_readback_tamper() {
        let mut disk = MemBlockDevice::<{ 256 * SECTOR_SIZE }>::new();
        let res = DedicatedPartitionReservation::new(64, 100, 1, 8).unwrap();

        let initial_sb = RaidSuperblock {
            version: RAID_SUPERBLOCK_VERSION_1,
            raid_type: RaidType::Raid1,
            generation: 10,
            array_uuid: [0x33; 16],
            member_uuid: [0x44; 16],
            member_index: 0,
            member_count: 2,
            stripe_sectors: 0,
            array_capacity_sectors: 92,
            member_usable_sectors: 92,
            data_start_lba: 72,
            state: RaidMetadataState::ActiveClean,
        };

        // Initialize clean member
        assert_eq!(
            initial_sb.initialize_member(
                &mut disk,
                &res,
                RaidInitializationPolicy::CreateNewArraySafe
            ),
            Ok(())
        );

        // Fault Injection 1: Flush failure propagation
        // Inject error right during flush, verify error returned and state not falsely claimed
        disk.set_inject_error(true);
        let mut update_sb = initial_sb;
        update_sb.generation = 11;
        assert_eq!(
            update_sb.update_operational_member(&mut disk, &res),
            Err(StorageError::IoError)
        );
        disk.set_inject_error(false);

        // Fault Injection 2: Read-back mismatch / sector tamper detection
        // Create an instrumented custom test device that writes a corrupted or old byte to disk
        // despite returning Ok from write_sectors, simulating media bit-rot during commit
        struct TamperingBlockDevice {
            inner: MemBlockDevice<{ 256 * SECTOR_SIZE }>,
            tamper_target_lba: u64,
        }
        impl BlockDevice for TamperingBlockDevice {
            fn total_sectors(&self) -> u64 {
                self.inner.total_sectors()
            }
            fn read_sectors(
                &mut self,
                start_lba: u64,
                sectors: u32,
                buf: &mut [u8],
            ) -> StorageResult<()> {
                self.inner.read_sectors(start_lba, sectors, buf)
            }
            fn write_sectors(
                &mut self,
                start_lba: u64,
                sectors: u32,
                buf: &[u8],
            ) -> StorageResult<()> {
                self.inner.write_sectors(start_lba, sectors, buf)?;
                // Corrupt the target sector immediately to simulate write verification failure
                if start_lba <= self.tamper_target_lba
                    && (start_lba + sectors as u64) > self.tamper_target_lba
                {
                    let offset = (self.tamper_target_lba as usize) * SECTOR_SIZE;
                    self.inner.storage[offset + 25] ^= 0xFF; // Invert a byte in generation field
                }
                Ok(())
            }
            fn flush(&mut self) -> StorageResult<()> {
                self.inner.flush()
            }
        }

        let mut tamper_dev = TamperingBlockDevice {
            inner: MemBlockDevice::<{ 256 * SECTOR_SIZE }>::new(),
            tamper_target_lba: res.absolute_metadata_lba().unwrap(),
        };

        // Initialize onto tamper_dev (tampering disabled during init by pointing target out of range)
        tamper_dev.tamper_target_lba = 999;
        assert_eq!(
            initial_sb.initialize_member(
                &mut tamper_dev,
                &res,
                RaidInitializationPolicy::CreateNewArraySafe
            ),
            Ok(())
        );

        // Enable tampering at the metadata LBA
        tamper_dev.tamper_target_lba = res.absolute_metadata_lba().unwrap();
        let mut update_fail = initial_sb;
        update_fail.generation = 12;

        // update_operational_member MUST detect the read-back discrepancy and reject with CorruptedMetadata!
        assert_eq!(
            update_fail.update_operational_member(&mut tamper_dev, &res),
            Err(StorageError::CorruptedMetadata)
        );

        // Fault Injection 3: Partial RAID 1 Member Update Simulation
        // Member 0 succeeds update, Member 1 fails I/O.
        // Verify that the error is propagated and no false global clean state is reported.
        let mut member0_disk = MemBlockDevice::<{ 256 * SECTOR_SIZE }>::new();
        let mut member1_disk = MemBlockDevice::<{ 256 * SECTOR_SIZE }>::new();
        let res0 = DedicatedPartitionReservation::new(64, 100, 1, 8).unwrap();
        let res1 = DedicatedPartitionReservation::new(64, 100, 1, 8).unwrap();

        let mut sb0 = initial_sb;
        sb0.member_index = 0;
        let mut sb1 = initial_sb;
        sb1.member_index = 1;
        sb1.member_uuid = [0x55; 16];

        assert_eq!(
            sb0.initialize_member(
                &mut member0_disk,
                &res0,
                RaidInitializationPolicy::CreateNewArraySafe
            ),
            Ok(())
        );
        assert_eq!(
            sb1.initialize_member(
                &mut member1_disk,
                &res1,
                RaidInitializationPolicy::CreateNewArraySafe
            ),
            Ok(())
        );

        // Inject error only on member 1
        member1_disk.set_inject_error(true);

        let mut next_gen_sb0 = sb0;
        next_gen_sb0.generation = 11;
        let mut next_gen_sb1 = sb1;
        next_gen_sb1.generation = 11;

        let res_update0 = next_gen_sb0.update_operational_member(&mut member0_disk, &res0);
        let res_update1 = next_gen_sb1.update_operational_member(&mut member1_disk, &res1);

        assert_eq!(res_update0, Ok(()));
        assert_eq!(res_update1, Err(StorageError::IoError));

        // Proves: Member 0 updated to generation 11, but Member 1 remains at generation 10.
        // A multi-disk coordinator must NOT declare the array universally clean when one member failed.
        let m0_current = RaidSuperblock::read_from_device(&mut member0_disk, &res0).unwrap();
        member1_disk.set_inject_error(false);
        let m1_current = RaidSuperblock::read_from_device(&mut member1_disk, &res1).unwrap();

        assert_eq!(m0_current.generation, 11);
        assert_eq!(m1_current.generation, 10);
        assert_eq!(
            RaidSuperblock::compare_generation(m0_current.generation, m1_current.generation),
            GenerationOrder::Newer
        );
    }

    #[test]
    fn test_gpt_raw_entry_linux_raid_guid_positive_and_negative() {
        let mut dev = MemBlockDevice::<{ 1024 * SECTOR_SIZE }>::new();

        let mut gpt_hdr = [0u8; SECTOR_SIZE];
        gpt_hdr[0..8].copy_from_slice(GPT_SIGNATURE);
        gpt_hdr[12..16].copy_from_slice(&92u32.to_le_bytes()); // Header size 92
        gpt_hdr[24..32].copy_from_slice(&1u64.to_le_bytes()); // My LBA 1
        gpt_hdr[40..48].copy_from_slice(&34u64.to_le_bytes()); // First usable LBA 34
        gpt_hdr[48..56].copy_from_slice(&1000u64.to_le_bytes()); // Last usable LBA 1000
        gpt_hdr[72..80].copy_from_slice(&2u64.to_le_bytes()); // Partition entries LBA 2
        gpt_hdr[80..84].copy_from_slice(&32u32.to_le_bytes()); // Num entries 32
        gpt_hdr[84..88].copy_from_slice(&128u32.to_le_bytes()); // Entry size 128

        // Compute and store valid CRC32
        let computed = partition::crc32(&gpt_hdr[..92]);
        gpt_hdr[16..20].copy_from_slice(&computed.to_le_bytes());

        // Construct 2 partition entries at LBA 2:
        // Entry 1 (0..128): Positive test with exact GPT_GUID_LINUX_RAID
        // Entry 2 (128..256): Negative test with generic Linux native data GUID
        let mut part_entries = [0u8; SECTOR_SIZE];

        // Entry 1: Linux RAID partition GUID
        part_entries[0..16].copy_from_slice(&GPT_GUID_LINUX_RAID);
        part_entries[32..40].copy_from_slice(&100u64.to_le_bytes()); // First LBA 100
        part_entries[40..48].copy_from_slice(&200u64.to_le_bytes()); // Last LBA 200

        // Entry 2: Different GUID (e.g. Linux native filesystem: 0FC63DAF-8483-4772-8E79-3D69D8477DE4)
        let linux_data_guid: [u8; 16] = [
            0xAF, 0x3D, 0xC6, 0x0F, 0x83, 0x84, 0x72, 0x47, 0x8E, 0x79, 0x3D, 0x69, 0xD8, 0x47,
            0x7D, 0xE4,
        ];
        part_entries[128..144].copy_from_slice(&linux_data_guid);
        part_entries[160..168].copy_from_slice(&201u64.to_le_bytes()); // First LBA 201
        part_entries[168..176].copy_from_slice(&300u64.to_le_bytes()); // Last LBA 300

        dev.write_sectors(1, 1, &gpt_hdr).unwrap();
        dev.write_sectors(2, 1, &part_entries).unwrap();

        let table = PartitionTable::parse(&mut dev).unwrap();
        assert_eq!(table.is_gpt, true);
        assert_eq!(table.count, 2);

        // Entry 1 MUST be recognized as PartitionType::RaidMember
        assert_eq!(table.partitions[0].part_type, PartitionType::RaidMember);
        assert_eq!(table.partitions[0].start_lba, 100);
        assert_eq!(table.partitions[0].sector_count, 101);

        // Entry 2 MUST be classified as generic data (NOT RaidMember)
        assert_eq!(table.partitions[1].part_type, PartitionType::GptGenericData);
        assert_eq!(table.partitions[1].start_lba, 201);
        assert_eq!(table.partitions[1].sector_count, 100);

        // Verify that from_verified_partition allows Entry 1 and rejects Entry 2
        assert!(
            DedicatedPartitionReservation::from_verified_partition(&table.partitions[0], 1, 4)
                .is_ok()
        );
        assert_eq!(
            DedicatedPartitionReservation::from_verified_partition(&table.partitions[1], 1, 4),
            Err(StorageError::InvalidParameter)
        );
    }
}
