//! In-Memory Deterministic Block Device for unit, property, and regression testing

use crate::block::{BlockDevice, StorageError, StorageResult, SECTOR_SIZE};
use crate::vfs::{
    DirectoryEntry, FileSystem, VfsError, VfsNode, VfsNodeType, VfsResult, MAX_NAME_LEN,
};

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

    fn flush(&mut self) -> StorageResult<()> {
        if self.inject_error {
            return Err(StorageError::IoError);
        }
        Ok(())
    }

    fn set_inject_error(&mut self, err: bool) {
        self.inject_error = err;
    }
}

// ============================================================================
// RAMDISK FILE SYSTEM (misi v1.0.2 Bagian 1.4 + 1.7)
// ----------------------------------------------------------------------------
// Berkas sistem VOLATIL tanpa alokasi heap: seluruh struktur tinggal di .bss
// dan usianya sama dengan satu boot. Persistensi ke pita-disk sungguhan
// sengaja BUKAN lingkup rilis ini (Bagian 1.4).
//
// Model penyimpanan: satu indeks simpul = satu slot data berukuran tetap,
// sehingga tidak ada alokator, tidak ada fragmentasi, dan pemakaian memori
// dapat dihitung sebelum boot (sifat yang dicari untuk DAL-A).
// ============================================================================

/// Batas maksimum simpul (direktori + berkas) satu instans RAMDISK.
pub const RAMDISK_MAX_NODES: usize = 64;

/// Jumlah simpul yang dipakai default.
///
/// Awalnya 64 (512 KiB) sesuai Bagian 1.7, tetapi terbukti bertabrakan dengan
/// gerbang boot `qemu -m 3M`: dengan 512 KiB serial tetap kosong pada
/// 3 MB padahal ISO pra-ramdisk boot normal. Bagian 1.7 mengizinkan
/// penurunan default ke minimum 256 KiB saat tabrakan, jadi default = 32 simpul.
pub const RAMDISK_DEFAULT_NODES: usize = 32;

/// Ukuran satu slot berkas dalam byte. Slot simpul ke-i = indeks i.
pub const RAMDISK_FILE_SLOT_BYTES: usize = 8 * 1024;

/// Kapasitas default RAMDISK sekarang: 32 x 8 KiB = 256 KiB (minimum Bagian 1.7).
pub const RAMDISK_CAPACITY_DEFAULT_BYTES: usize = RAMDISK_DEFAULT_NODES * RAMDISK_FILE_SLOT_BYTES;

/// Kapasitas minimum yang diizinkan Bagian 1.7 (256 KiB).
pub const RAMDISK_CAPACITY_MIN_BYTES: usize = RAMDISK_DEFAULT_NODES * RAMDISK_FILE_SLOT_BYTES;

/// Kapasitas maksimum bila tabrakan memori boot tidak terjadi (64 x 8 KiB = 512 KiB).
pub const RAMDISK_CAPACITY_MAX_BYTES: usize = RAMDISK_MAX_NODES * RAMDISK_FILE_SLOT_BYTES;

/// Inode akar selalu 1 dan menempati indeks simpul 0.
pub const RAMDISK_ROOT_INODE: u64 = 1;

/// Satu entri tabel simpul RAMDISK.
#[derive(Debug, Copy, Clone)]
pub struct RamNode {
    pub in_use: bool,
    pub kind: VfsNodeType,
    pub parent: u64,
    pub inode: u64,
    pub name: [u8; MAX_NAME_LEN],
    pub name_len: usize,
    pub size: usize,
}

impl RamNode {
    /// Simpul kosong (indeks bebas) dengan nama ternol.
    pub const fn free() -> Self {
        Self {
            in_use: false,
            kind: VfsNodeType::File,
            parent: 0,
            inode: 0,
            name: [0u8; MAX_NAME_LEN],
            name_len: 0,
            size: 0,
        }
    }

    pub fn name_as_str(&self) -> &str {
        core::str::from_utf8(&self.name[..self.name_len]).unwrap_or("")
    }
}

/// Berkas sistem RAMDISK volatil yang memenuhi kontrak `FileSystem`.
///
/// `SLOTS` = jumlah simpul (sekaligus jumlah slot data), `SLOT_BYTES` = ukuran
/// satu berkas. Jejak memori = `SLOTS * SLOT_BYTES` byte dan TIDAK berubah oleh
/// isi direktori, sehingga mudah dibukukan pada gerbang memori.
pub struct RamFileSystem<
    const SLOTS: usize = RAMDISK_DEFAULT_NODES,
    const SLOT_BYTES: usize = RAMDISK_FILE_SLOT_BYTES,
> {
    pub nodes: [RamNode; SLOTS],
    pub data: [[u8; SLOT_BYTES]; SLOTS],
    pub next_inode: u64,
}

impl<const SLOTS: usize, const SLOT_BYTES: usize> RamFileSystem<SLOTS, SLOT_BYTES> {
    /// Simpul akar: direktori, inode 1, tanpa nama.
    const fn root_node() -> RamNode {
        RamNode {
            in_use: true,
            kind: VfsNodeType::Directory,
            parent: RAMDISK_ROOT_INODE,
            inode: RAMDISK_ROOT_INODE,
            name: [0u8; MAX_NAME_LEN],
            name_len: 0,
            size: 0,
        }
    }

    /// Buat RAMDISK baru berisi satu direktori akar kosong.
    pub const fn new() -> Self {
        let mut nodes = [RamNode::free(); SLOTS];
        nodes[0] = Self::root_node();
        Self {
            nodes,
            data: [[0u8; SLOT_BYTES]; SLOTS],
            next_inode: RAMDISK_ROOT_INODE + 1,
        }
    }

    /// Konstruktur untuk statis kernel: setiap byte bernilai nol sehingga
    /// penempatan jatuh ke `.bss` dan image kernel tidak perlu membawa
    /// `SLOTS * SLOT_BYTES` byte nol. Akar dipasang `ensure_root()` sebelum pakai.
    pub const fn new_bss_zeroed() -> Self {
        Self {
            nodes: [RamNode::free(); SLOTS],
            data: [[0u8; SLOT_BYTES]; SLOTS],
            next_inode: 0,
        }
    }

    /// Pasang simpul akar bila belum terpasang. Idempoten: aman dipanggil pada
    /// setiap perintah dan tidak pernah menimpa isi yang sudah ada.
    pub fn ensure_root(&mut self) {
        if self.nodes[0].in_use {
            return;
        }
        self.nodes[0] = Self::root_node();
        self.next_inode = RAMDISK_ROOT_INODE + 1;
    }

    /// Indeks simpul pemilik inode, atau `None` bila inode tidak dikenal.
    fn index_of_inode(&self, inode: u64) -> Option<usize> {
        if inode == RAMDISK_ROOT_INODE {
            return Some(0);
        }
        let mut i = 1;
        while i < SLOTS {
            if self.nodes[i].in_use && self.nodes[i].inode == inode {
                return Some(i);
            }
            i += 1;
        }
        None
    }

    /// Indeks anak langsung dari `parent_inode` yang namanya cocok.
    fn find_child(&self, parent_inode: u64, name: &str) -> Option<usize> {
        if name.is_empty() || name.len() > MAX_NAME_LEN {
            return None;
        }
        let mut i = 1;
        while i < SLOTS {
            let n = &self.nodes[i];
            if n.in_use && n.parent == parent_inode && n.name_as_str() == name {
                return Some(i);
            }
            i += 1;
        }
        None
    }

    /// Indeks simpul bebas pertama setelah akar.
    fn find_free(&self) -> Option<usize> {
        let mut i = 1;
        while i < SLOTS {
            if !self.nodes[i].in_use {
                return Some(i);
            }
            i += 1;
        }
        None
    }

    /// Jumlah anak langsung (untuk uji direktori tidak kosong).
    fn child_count(&self, parent_inode: u64) -> usize {
        let mut c = 0;
        let mut i = 1;
        while i < SLOTS {
            if self.nodes[i].in_use && self.nodes[i].parent == parent_inode {
                c += 1;
            }
            i += 1;
        }
        c
    }

    fn node_view(&self, idx: usize) -> VfsNode {
        let n = &self.nodes[idx];
        VfsNode {
            node_type: n.kind,
            size_bytes: n.size as u64,
            inode: n.inode,
        }
    }

    /// Total entri hidup termasuk akar (dipakai tampilan `status`).
    pub fn total_entries(&self) -> usize {
        let mut c = 1;
        let mut i = 1;
        while i < SLOTS {
            if self.nodes[i].in_use {
                c += 1;
            }
            i += 1;
        }
        c
    }

    /// Buat satu entri (berkas atau direktori) di bawah `parent`.
    fn create_entry(
        &mut self,
        parent: &VfsNode,
        name: &str,
        kind: VfsNodeType,
    ) -> VfsResult<VfsNode> {
        let pidx = self.index_of_inode(parent.inode).ok_or(VfsError::NotFound)?;
        if self.nodes[pidx].kind != VfsNodeType::Directory {
            return Err(VfsError::NotADirectory);
        }
        if name.is_empty() || name == "." || name == ".." {
            return Err(VfsError::InvalidPath);
        }
        if name.len() > MAX_NAME_LEN || name.contains('/') {
            return Err(VfsError::InvalidPath);
        }
        let parent_inode = self.nodes[pidx].inode;
        if self.find_child(parent_inode, name).is_some() {
            return Err(VfsError::AlreadyExists);
        }
        let idx = self.find_free().ok_or(VfsError::NoSpaceLeft)?;
        let mut node = RamNode::free();
        node.in_use = true;
        node.kind = kind;
        node.parent = parent_inode;
        node.inode = self.next_inode;
        node.name_len = name.len();
        let mut i = 0;
        while i < name.len() {
            node.name[i] = name.as_bytes()[i];
            i += 1;
        }
        self.nodes[idx] = node;
        self.next_inode += 1;
        if kind == VfsNodeType::File {
            let mut s = 0;
            while s < SLOT_BYTES {
                self.data[idx][s] = 0;
                s += 1;
            }
        }
        Ok(self.node_view(idx))
    }
}

impl<const SLOTS: usize, const SLOT_BYTES: usize> FileSystem for RamFileSystem<SLOTS, SLOT_BYTES> {
    fn name(&self) -> &'static str {
        "ramdisk"
    }

    fn lookup(&mut self, path: &str) -> VfsResult<VfsNode> {
        let trimmed = path.trim();
        if trimmed.is_empty() || trimmed == "/" {
            return Ok(self.node_view(0));
        }
        let mut current_inode = RAMDISK_ROOT_INODE;
        for seg in trimmed.split('/') {
            if seg.is_empty() || seg == "." {
                continue;
            }
            if seg == ".." {
                let idx = self.index_of_inode(current_inode).ok_or(VfsError::NotFound)?;
                current_inode = self.nodes[idx].parent;
                continue;
            }
            let idx = self.find_child(current_inode, seg).ok_or(VfsError::NotFound)?;
            current_inode = self.nodes[idx].inode;
        }
        let idx = self.index_of_inode(current_inode).ok_or(VfsError::NotFound)?;
        Ok(self.node_view(idx))
    }

    fn read(&mut self, node: &VfsNode, offset: u64, buf: &mut [u8]) -> VfsResult<usize> {
        let idx = self.index_of_inode(node.inode).ok_or(VfsError::NotFound)?;
        if self.nodes[idx].kind != VfsNodeType::File {
            return Err(VfsError::IsADirectory);
        }
        let size = self.nodes[idx].size;
        let start = offset as usize;
        if start >= size {
            return Ok(0);
        }
        let avail = size - start;
        let n = if buf.len() < avail { buf.len() } else { avail };
        let slot = &self.data[idx];
        buf[..n].copy_from_slice(&slot[start..start + n]);
        Ok(n)
    }

    fn write(&mut self, node: &VfsNode, offset: u64, buf: &[u8]) -> VfsResult<usize> {
        let idx = self.index_of_inode(node.inode).ok_or(VfsError::NotFound)?;
        if self.nodes[idx].kind != VfsNodeType::File {
            return Err(VfsError::IsADirectory);
        }
        let start = offset as usize;
        let need = start.checked_add(buf.len()).ok_or(VfsError::NoSpaceLeft)?;
        if need > SLOT_BYTES {
            return Err(VfsError::NoSpaceLeft);
        }
        let mut i = 0;
        while i < buf.len() {
            self.data[idx][start + i] = buf[i];
            i += 1;
        }
        if need > self.nodes[idx].size {
            self.nodes[idx].size = need;
        }
        Ok(buf.len())
    }

    fn read_dir(&mut self, node: &VfsNode, entries: &mut [DirectoryEntry]) -> VfsResult<usize> {
        let idx = self.index_of_inode(node.inode).ok_or(VfsError::NotFound)?;
        if self.nodes[idx].kind != VfsNodeType::Directory {
            return Err(VfsError::NotADirectory);
        }
        let parent_inode = self.nodes[idx].inode;
        let mut out = 0;
        let mut i = 1;
        while i < SLOTS {
            let n = &self.nodes[i];
            if n.in_use && n.parent == parent_inode {
                if out >= entries.len() {
                    break;
                }
                entries[out] = DirectoryEntry::new(
                    n.name_as_str(),
                    n.kind,
                    n.size as u64,
                    n.inode,
                );
                out += 1;
            }
            i += 1;
        }
        Ok(out)
    }

    fn create_file(&mut self, parent: &VfsNode, name: &str) -> VfsResult<VfsNode> {
        self.create_entry(parent, name, VfsNodeType::File)
    }

    fn create_dir(&mut self, parent: &VfsNode, name: &str) -> VfsResult<VfsNode> {
        self.create_entry(parent, name, VfsNodeType::Directory)
    }

    fn delete_file(&mut self, parent: &VfsNode, name: &str) -> VfsResult<()> {
        let pidx = self.index_of_inode(parent.inode).ok_or(VfsError::NotFound)?;
        if self.nodes[pidx].kind != VfsNodeType::Directory {
            return Err(VfsError::NotADirectory);
        }
        let idx = self
            .find_child(self.nodes[pidx].inode, name)
            .ok_or(VfsError::NotFound)?;
        if self.nodes[idx].kind != VfsNodeType::File {
            return Err(VfsError::IsADirectory);
        }
        self.nodes[idx] = RamNode::free();
        let mut s = 0;
        while s < SLOT_BYTES {
            self.data[idx][s] = 0;
            s += 1;
        }
        Ok(())
    }

    fn delete_dir(&mut self, parent: &VfsNode, name: &str) -> VfsResult<()> {
        let pidx = self.index_of_inode(parent.inode).ok_or(VfsError::NotFound)?;
        if self.nodes[pidx].kind != VfsNodeType::Directory {
            return Err(VfsError::NotADirectory);
        }
        let idx = self
            .find_child(self.nodes[pidx].inode, name)
            .ok_or(VfsError::NotFound)?;
        if self.nodes[idx].kind != VfsNodeType::Directory {
            return Err(VfsError::NotADirectory);
        }
        let inode = self.nodes[idx].inode;
        if self.child_count(inode) > 0 {
            return Err(VfsError::DirectoryNotEmpty);
        }
        self.nodes[idx] = RamNode::free();
        Ok(())
    }

    fn truncate(&mut self, node: &VfsNode, new_len: usize) -> VfsResult<()> {
        let idx = self.index_of_inode(node.inode).ok_or(VfsError::NotFound)?;
        if self.nodes[idx].kind != VfsNodeType::File {
            return Err(VfsError::IsADirectory);
        }
        if new_len > SLOT_BYTES {
            return Err(VfsError::NoSpaceLeft);
        }
        let mut s = new_len;
        while s < self.nodes[idx].size {
            self.data[idx][s] = 0;
            s += 1;
        }
        self.nodes[idx].size = new_len;
        Ok(())
    }

    fn capacity_bytes(&self) -> usize {
        SLOTS * SLOT_BYTES
    }

    fn used_bytes(&self) -> usize {
        let mut sum = 0usize;
        let mut i = 1;
        while i < SLOTS {
            if self.nodes[i].in_use && self.nodes[i].kind == VfsNodeType::File {
                sum += self.nodes[i].size;
            }
            i += 1;
        }
        sum
    }

    fn entry_count(&self) -> usize {
        self.total_entries()
    }
}

#[cfg(test)]
mod fs_tests {
    use super::*;
    use crate::vfs::{resolve_path, MAX_PATH_LEN, Vfs, MAX_DIR_ENTRIES};

    type TinyFs = RamFileSystem<8, 64>;

    fn root_node(fs: &mut TinyFs) -> VfsNode {
        fs.lookup("/").expect("akar harus ada")
    }

    #[test]
    fn test_ram_filesystem_empty_root_and_lookup() {
        let mut fs = TinyFs::new();
        assert_eq!(fs.name(), "ramdisk");
        let root = root_node(&mut fs);
        assert_eq!(root.node_type, VfsNodeType::Directory);
        assert_eq!(root.inode, RAMDISK_ROOT_INODE);
        assert_eq!(fs.lookup("/tidakada").unwrap_err(), VfsError::NotFound);
        let mut entries = [DirectoryEntry::empty(); MAX_DIR_ENTRIES];
        assert_eq!(fs.read_dir(&root, &mut entries).unwrap(), 0);
    }

    #[test]
    fn test_ram_filesystem_write_read_truncate() {
        let mut fs = TinyFs::new();
        let root = root_node(&mut fs);
        let file = fs.create_file(&root, "catat.txt").unwrap();
        assert_eq!(file.node_type, VfsNodeType::File);
        assert_eq!(file.size_bytes, 0);

        assert_eq!(fs.write(&file, 0, b"HELLO RAMDISK").unwrap(), 13);
        assert_eq!(fs.lookup("/catat.txt").unwrap().size_bytes, 13);

        let mut buf = [0u8; 32];
        assert_eq!(fs.read(&file, 0, &mut buf).unwrap(), 13);
        assert_eq!(&buf[..13], b"HELLO RAMDISK");
        assert_eq!(fs.read(&file, 99, &mut buf).unwrap(), 0);

        // Tulis di tengah menimpa byte lama TANPA memotong: ukuran tetap 13,
        // sehingga "RAMDISK" yang tersisa masih terbaca sebagai "SK".
        assert_eq!(fs.write(&file, 6, b"WORLD").unwrap(), 5);
        assert_eq!(fs.read(&file, 0, &mut buf).unwrap(), 13);
        assert_eq!(&buf[..13], b"HELLO WORLDSK");

        // Potong: sisa byte lama ternol, ukuran mengecil.
        fs.truncate(&file, 5).unwrap();
        assert_eq!(fs.lookup("/catat.txt").unwrap().size_bytes, 5);
        assert_eq!(fs.read(&file, 0, &mut buf).unwrap(), 5);

        // Melebihi slot berkas -> NoSpaceLeft (fail-closed, bukan panik).
        let big = [0u8; 65];
        assert_eq!(
            fs.write(&file, 0, &big).unwrap_err(),
            VfsError::NoSpaceLeft
        );
    }

    #[test]
    fn test_ram_filesystem_delete_and_slot_reclaim() {
        let mut fs = TinyFs::new();
        let root = root_node(&mut fs);
        let dir = fs.create_dir(&root, "data").unwrap();
        fs.create_file(&dir, "dalam.txt").unwrap();

        // Direktori berisi tidak boleh dihapus.
        assert_eq!(
            fs.delete_dir(&root, "data").unwrap_err(),
            VfsError::DirectoryNotEmpty
        );
        // Berkas tidak boleh dihapus lewat jalur direktori, dan sebaliknya.
        assert_eq!(
            fs.delete_file(&root, "data").unwrap_err(),
            VfsError::IsADirectory
        );
        fs.delete_file(&dir, "dalam.txt").unwrap();
        assert_eq!(fs.delete_file(&dir, "dalam.txt").unwrap_err(), VfsError::NotFound);
        fs.delete_dir(&root, "data").unwrap();
        assert_eq!(fs.lookup("/data").unwrap_err(), VfsError::NotFound);
        assert_eq!(fs.total_entries(), 1);
    }

    #[test]
    fn test_ram_filesystem_duplicate_name_full_and_invalid() {
        let mut fs = TinyFs::new();
        let root = root_node(&mut fs);
        fs.create_file(&root, "satu").unwrap();
        assert_eq!(
            fs.create_file(&root, "satu").unwrap_err(),
            VfsError::AlreadyExists
        );
        // Nama kembar di direktori berbeda boleh.
        let d = fs.create_dir(&root, "dir").unwrap();
        fs.create_file(&d, "satu").unwrap();
        assert_eq!(fs.lookup("/dir/satu").unwrap().node_type, VfsNodeType::File);

        assert_eq!(fs.create_file(&root, "").unwrap_err(), VfsError::InvalidPath);
        assert_eq!(
            fs.create_file(&root, "ada/garis").unwrap_err(),
            VfsError::InvalidPath
        );
        // Tabel simpul penuh -> NoSpaceLeft, bukan panik.
        let mut full = RamFileSystem::<4, 16>::new();
        let r = full.lookup("/").unwrap();
        full.create_file(&r, "a").unwrap();
        full.create_file(&r, "b").unwrap();
        full.create_file(&r, "c").unwrap();
        assert_eq!(
            full.create_file(&r, "d").unwrap_err(),
            VfsError::NoSpaceLeft
        );
    }

    #[test]
    fn test_ram_filesystem_vfs_full_path_journey() {
        let mut fs = TinyFs::new();
        let mut vfs = Vfs::new();
        vfs.mount_root(&mut fs);
        assert!(vfs.is_mounted());

        vfs.mkdir("/", "log").unwrap();
        vfs.touch("/log", "err.txt").unwrap();
        let mut entries = [DirectoryEntry::empty(); MAX_DIR_ENTRIES];
        let dir = vfs.open("/log").unwrap();
        assert_eq!(vfs.read_dir(&dir, &mut entries).unwrap(), 1);
        assert_eq!(entries[0].name_as_str(), "err.txt");

        vfs.write_file("/log/err.txt", b"kali ini panjang").unwrap();
        let mut buf = [0u8; 40];
        assert_eq!(vfs.read_file("/log/err.txt", &mut buf).unwrap(), 16);
        assert_eq!(&buf[..16], b"kali ini panjang");
        // Penggantian dengan teks lebih pendek harus memotong sisa lama.
        vfs.write_file("/log/err.txt", b"pendek").unwrap();
        assert_eq!(vfs.read_file("/log/err.txt", &mut buf).unwrap(), 6);
        assert_eq!(vfs.used_bytes(), 6);
        assert_eq!(vfs.capacity_bytes(), 8 * 64);
        assert_eq!(vfs.entry_count(), 3);
        assert_eq!(vfs.fs_name(), "ramdisk");

        vfs.rm("/log", "err.txt").unwrap();
        vfs.rmdir("/log", "").unwrap_err();
        vfs.rmdir("/", "log").unwrap();
        assert_eq!(vfs.open("/log").unwrap_err(), VfsError::NotFound);

        // Backend default (hapus belum diimplementasikan) tidak dipakai di sini,
        // jadi `rm` di atas harus sukses, bukan ReadOnly.
    }

    #[test]
    fn test_resolve_path_normalization() {
        let mut out = [0u8; MAX_PATH_LEN];
        assert_eq!(resolve_path("/", "/", &mut out), Some("/"));
        assert_eq!(resolve_path("/a/b", "", &mut out), Some("/a/b"));
        assert_eq!(resolve_path("/a/b", "c", &mut out), Some("/a/b/c"));
        assert_eq!(resolve_path("/a/b", "../c", &mut out), Some("/a/c"));
        assert_eq!(resolve_path("/a/b", "../../..", &mut out), Some("/"));
        assert_eq!(resolve_path("/a/b", "/x/y/", &mut out), Some("/x/y"));
        assert_eq!(resolve_path("/a", "./b/./c", &mut out), Some("/a/b/c"));
        assert_eq!(resolve_path("/", "..", &mut out), Some("/"));
        // Melebihi kapasitas penampung harus Galat, bukan menulis keluar batas.
        let long_path = ["zzzzzzzzzz"; 20].join("/");
        assert_eq!(resolve_path("/", long_path.as_str(), &mut out), None);
    }

    #[test]
    fn test_ram_filesystem_capacity_bounds() {
        // Angka Bagian 1.7 diuji, bukan diasumsikan. Default sekarang duduk di
        // MINIMUM 256 KiB karena tabrakan gerbang boot `qemu -m 3M`.
        assert_eq!(RAMDISK_FILE_SLOT_BYTES, 8 * 1024);
        assert_eq!(RAMDISK_DEFAULT_NODES, 32);
        assert_eq!(RAMDISK_MAX_NODES, 64);
        assert_eq!(RAMDISK_CAPACITY_DEFAULT_BYTES, 256 * 1024);
        assert_eq!(RAMDISK_CAPACITY_MIN_BYTES, 256 * 1024);
        assert_eq!(RAMDISK_CAPACITY_MAX_BYTES, 512 * 1024);
        // Instans default (dipakai kernel) = kapasitas default. Anotasi tipe
        // diperlukan karena nilai default parameter const hanya dipakai pada
        // posisi tipe, bukan pada posisi pemanggilan.
        let default_fs: RamFileSystem = RamFileSystem::new();
        assert_eq!(default_fs.capacity_bytes(), RAMDISK_CAPACITY_DEFAULT_BYTES);
        // Batas maksimum tetap tersedia bila gerbang memori suatu hari longgar.
        assert_eq!(
            RamFileSystem::<RAMDISK_MAX_NODES, RAMDISK_FILE_SLOT_BYTES>::new().capacity_bytes(),
            RAMDISK_CAPACITY_MAX_BYTES
        );
    }

    #[test]
    fn test_zeroed_static_requires_ensure_root() {
        // Jalur statis kernel: nilai awal seluruhnya nol agar `.bss`, dan akar
        // baru ada sesudah `ensure_root()`.
        let mut fs = TinyFs::new_bss_zeroed();
        assert_eq!(fs.lookup("/").unwrap().node_type, VfsNodeType::File);
        fs.ensure_root();
        let root = fs.lookup("/").unwrap();
        assert_eq!(root.node_type, VfsNodeType::Directory);
        assert_eq!(root.inode, RAMDISK_ROOT_INODE);
        assert_eq!(fs.total_entries(), 1);

        // ensure_root idempoten: tidak menghapus isi yang sudah dibuat.
        fs.create_dir(&root, "log").unwrap();
        fs.ensure_root();
        assert_eq!(fs.total_entries(), 2);
        assert_eq!(fs.lookup("/log").unwrap().node_type, VfsNodeType::Directory);

        // Konstructor biasa tetap langsung ber-akar dan tetap aman dipanggil lagi.
        let mut plain = TinyFs::new();
        assert!(plain.lookup("/").is_ok());
        plain.ensure_root();
        assert_eq!(plain.total_entries(), 1);
    }
}
