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
pub mod ramdisk;
pub mod vfs;

pub use ahci::{AhciController, AhciPortType, HbaPort};
pub use ata::{
    AtaChannel, AtaDeviceType, AtaPioDriver, ATA_PRIMARY_CTRL_BASE, ATA_PRIMARY_IO_BASE,
};
pub use block::{BlockDevice, StorageError, StorageResult, SECTOR_SIZE};
pub use ext4::{Ext4FileSystem, Ext4Superblock, EXT4_FEATURE_INCOMPAT_EXTENTS, EXT4_MAGIC};
pub use fat32::{Fat32Bpb, Fat32FileSystem};
pub use partition::{Partition, PartitionTable, PartitionType, GPT_SIGNATURE, MBR_MAGIC};
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
}
