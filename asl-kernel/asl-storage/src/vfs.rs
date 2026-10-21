//! Virtual File System (VFS) Abstraction
//! Standard POSIX / UNIX-like Virtual Tree with Inodes, Mount Points & Path Resolution

use crate::block::StorageError;

pub const MAX_NAME_LEN: usize = 64;
pub const MAX_PATH_LEN: usize = 128;
pub const MAX_DIR_ENTRIES: usize = 32;
pub const MAX_MOUNT_POINTS: usize = 4;

#[derive(Debug, Copy, Clone, PartialEq, Eq)]
pub enum VfsNodeType {
    File,
    Directory,
    BlockDevice,
    CharacterDevice,
}

#[derive(Debug, Copy, Clone, PartialEq, Eq)]
pub enum VfsError {
    NotFound,
    AlreadyExists,
    IsADirectory,
    NotADirectory,
    DirectoryNotEmpty,
    NoSpaceLeft,
    ReadOnly,
    InvalidPath,
    Storage(StorageError),
}

pub type VfsResult<T> = Result<T, VfsError>;

impl From<StorageError> for VfsError {
    fn from(err: StorageError) -> Self {
        match err {
            StorageError::WriteProtected => VfsError::ReadOnly,
            StorageError::NoSuchEntry => VfsError::NotFound,
            StorageError::AlreadyExists => VfsError::AlreadyExists,
            StorageError::IsADirectory => VfsError::IsADirectory,
            StorageError::NotADirectory => VfsError::NotADirectory,
            StorageError::DirectoryNotEmpty => VfsError::DirectoryNotEmpty,
            StorageError::NoSpaceLeft => VfsError::NoSpaceLeft,
            StorageError::InvalidParameter => VfsError::InvalidPath,
            other => VfsError::Storage(other),
        }
    }
}

/// Single directory entry in VFS
#[derive(Debug, Copy, Clone, PartialEq, Eq)]
pub struct DirectoryEntry {
    pub name: [u8; MAX_NAME_LEN],
    pub name_len: usize,
    pub node_type: VfsNodeType,
    pub size_bytes: u64,
    pub inode: u64,
}

impl DirectoryEntry {
    pub const fn empty() -> Self {
        Self {
            name: [0u8; MAX_NAME_LEN],
            name_len: 0,
            node_type: VfsNodeType::File,
            size_bytes: 0,
            inode: 0,
        }
    }

    pub fn new(name_str: &str, node_type: VfsNodeType, size_bytes: u64, inode: u64) -> Self {
        let mut entry = Self::empty();
        let bytes = name_str.as_bytes();
        let len = if bytes.len() > MAX_NAME_LEN {
            MAX_NAME_LEN
        } else {
            bytes.len()
        };
        for i in 0..len {
            entry.name[i] = bytes[i];
        }
        entry.name_len = len;
        entry.node_type = node_type;
        entry.size_bytes = size_bytes;
        entry.inode = inode;
        entry
    }

    pub fn name_as_str(&self) -> &str {
        core::str::from_utf8(&self.name[..self.name_len]).unwrap_or("")
    }
}

/// Represents an opened or referenced VFS node
#[derive(Debug, Copy, Clone, PartialEq, Eq)]
pub struct VfsNode {
    pub node_type: VfsNodeType,
    pub size_bytes: u64,
    pub inode: u64,
}

/// Generic File System Operations Trait
pub trait FileSystem {
    fn name(&self) -> &'static str;
    fn lookup(&mut self, path: &str) -> VfsResult<VfsNode>;
    fn read(&mut self, node: &VfsNode, offset: u64, buf: &mut [u8]) -> VfsResult<usize>;
    fn write(&mut self, node: &VfsNode, offset: u64, buf: &[u8]) -> VfsResult<usize>;
    fn read_dir(&mut self, node: &VfsNode, entries: &mut [DirectoryEntry]) -> VfsResult<usize>;
    fn create_file(&mut self, parent: &VfsNode, name: &str) -> VfsResult<VfsNode>;
    fn create_dir(&mut self, parent: &VfsNode, name: &str) -> VfsResult<VfsNode>;

    /// Hapus satu berkas pada direktori induk.
    ///
    /// Standar = `ReadOnly` (BUKAN `unimplemented!`) supaya backend pita-disk yang
    /// sudah ada (FAT32/Ext4) tetap kompilasi tanpa disentuh. Misi v1.0.2 Bagian 1.4
    /// hanya menjanjikan create/delete pada RAMDISK volatil; create/delete FAT32
    /// berstatus STRETCH dan sengaja tidak dijanjikan rilis ini.
    fn delete_file(&mut self, _parent: &VfsNode, _name: &str) -> VfsResult<()> {
        Err(VfsError::ReadOnly)
    }

    /// Hapus satu direktori kosong pada direktori induk. Standar = `ReadOnly`;
    /// lihat catatan `delete_file`.
    fn delete_dir(&mut self, _parent: &VfsNode, _name: &str) -> VfsResult<()> {
        Err(VfsError::ReadOnly)
    }

    /// Potong berkas menjadi `new_len` byte. Standar = `ReadOnly` (lihat `delete_file`).
    fn truncate(&mut self, _node: &VfsNode, _new_len: usize) -> VfsResult<()> {
        Err(VfsError::ReadOnly)
    }

    /// Kapasitas tertulis dalam byte; `0` = backend tidak melaporkannya.
    fn capacity_bytes(&self) -> usize {
        0
    }

    /// Byte yang sudah terpakai; `0` = backend tidak melaporkannya.
    fn used_bytes(&self) -> usize {
        0
    }

    /// Jumlah entri hidup (direktori + berkas) termasuk akar;
    /// `0` = backend tidak melaporkannya.
    fn entry_count(&self) -> usize {
        0
    }
}

/// Global Virtual File System Manager
pub struct Vfs<'a> {
    pub root_fs: Option<&'a mut dyn FileSystem>,
}

impl<'a> Vfs<'a> {
    pub const fn new() -> Self {
        Self { root_fs: None }
    }

    pub fn mount_root(&mut self, fs: &'a mut dyn FileSystem) {
        self.root_fs = Some(fs);
    }

    pub fn is_mounted(&self) -> bool {
        self.root_fs.is_some()
    }

    pub fn open(&mut self, path: &str) -> VfsResult<VfsNode> {
        let fs = self.root_fs.as_mut().ok_or(VfsError::NotFound)?;
        fs.lookup(path)
    }

    pub fn read(&mut self, node: &VfsNode, offset: u64, buf: &mut [u8]) -> VfsResult<usize> {
        let fs = self.root_fs.as_mut().ok_or(VfsError::NotFound)?;
        fs.read(node, offset, buf)
    }

    pub fn write(&mut self, node: &VfsNode, offset: u64, buf: &[u8]) -> VfsResult<usize> {
        let fs = self.root_fs.as_mut().ok_or(VfsError::NotFound)?;
        fs.write(node, offset, buf)
    }

    pub fn read_dir(&mut self, node: &VfsNode, entries: &mut [DirectoryEntry]) -> VfsResult<usize> {
        let fs = self.root_fs.as_mut().ok_or(VfsError::NotFound)?;
        fs.read_dir(node, entries)
    }

    pub fn mkdir(&mut self, parent_path: &str, name: &str) -> VfsResult<VfsNode> {
        let fs = self.root_fs.as_mut().ok_or(VfsError::NotFound)?;
        let parent = fs.lookup(parent_path)?;
        fs.create_dir(&parent, name)
    }

    pub fn touch(&mut self, parent_path: &str, name: &str) -> VfsResult<VfsNode> {
        let fs = self.root_fs.as_mut().ok_or(VfsError::NotFound)?;
        let parent = fs.lookup(parent_path)?;
        fs.create_file(&parent, name)
    }

    /// Hapus berkas lewat jalur induk-oleh-nama (bukan path penuh) agar konsisten
    /// dengan kontrak `FileSystem::delete_file`.
    pub fn rm(&mut self, parent_path: &str, name: &str) -> VfsResult<()> {
        let fs = self.root_fs.as_mut().ok_or(VfsError::NotFound)?;
        let parent = fs.lookup(parent_path)?;
        fs.delete_file(&parent, name)
    }

    /// Hapus direktori kosong lewat jalur induk-oleh-nama.
    pub fn rmdir(&mut self, parent_path: &str, name: &str) -> VfsResult<()> {
        let fs = self.root_fs.as_mut().ok_or(VfsError::NotFound)?;
        let parent = fs.lookup(parent_path)?;
        fs.delete_dir(&parent, name)
    }

    /// Listing satu direktori berdasarkan path (kombinasi `lookup` + `read_dir`).
    pub fn list_dir(&mut self, path: &str, entries: &mut [DirectoryEntry]) -> VfsResult<usize> {
        let fs = self.root_fs.as_mut().ok_or(VfsError::NotFound)?;
        let node = fs.lookup(path)?;
        fs.read_dir(&node, entries)
    }

    /// Isi satu berkas berdasarkan path ke `buf`; mengembalikan byte terbaca.
    pub fn read_file(&mut self, path: &str, buf: &mut [u8]) -> VfsResult<usize> {
        let fs = self.root_fs.as_mut().ok_or(VfsError::NotFound)?;
        let node = fs.lookup(path)?;
        if node.node_type != VfsNodeType::File {
            return Err(VfsError::IsADirectory);
        }
        fs.read(&node, 0, buf)
    }

    /// Tulis seluruh `buf` ke berkas berdasarkan path (memotong ukuran lama).
    pub fn write_file(&mut self, path: &str, buf: &[u8]) -> VfsResult<usize> {
        let fs = self.root_fs.as_mut().ok_or(VfsError::NotFound)?;
        let node = fs.lookup(path)?;
        if node.node_type != VfsNodeType::File {
            return Err(VfsError::IsADirectory);
        }
        let written = fs.write(&node, 0, buf)?;
        if (buf.len() as u64) < node.size_bytes {
            // Pangkas sisa byte lama supaya isi berkas = tepat `buf`.
            fs.truncate(&node, written)?;
        }
        Ok(written)
    }

    /// Potong satu berkas berdasarkan path.
    pub fn truncate(&mut self, path: &str, new_len: usize) -> VfsResult<()> {
        let fs = self.root_fs.as_mut().ok_or(VfsError::NotFound)?;
        let node = fs.lookup(path)?;
        if node.node_type != VfsNodeType::File {
            return Err(VfsError::IsADirectory);
        }
        fs.truncate(&node, new_len)
    }

    pub fn capacity_bytes(&self) -> usize {
        match &self.root_fs {
            Some(fs) => fs.capacity_bytes(),
            None => 0,
        }
    }

    pub fn used_bytes(&self) -> usize {
        match &self.root_fs {
            Some(fs) => fs.used_bytes(),
            None => 0,
        }
    }

    /// Entri hidup pada sistem berkas terpasang (0 bila belum terpasang).
    pub fn entry_count(&self) -> usize {
        match &self.root_fs {
            Some(fs) => fs.entry_count(),
            None => 0,
        }
    }

    pub fn fs_name(&self) -> &'static str {
        match &self.root_fs {
            Some(fs) => fs.name(),
            None => "(tidak terpasang)",
        }
    }
}

/// Kedalaman maksimum path yang bisa dinormalisasi (jumlah segmen nama).
pub const MAX_PATH_DEPTH: usize = 16;

/// Gabungkan direktori kerja `cwd` dengan `input` menjadi path absolut ternormalisasi.
///
/// Aturan: `.` diabaikan, `..` naik satu tingkat (tidak bisa naik dari root),
/// `input` yang diawali `/` mengabaikan `cwd`. Hasil selalu diawali `/`, tanpa
/// garis miring ganda, dan tanpa garis miring penutup kecuali root `"/"`.
/// Pengembalian meminjam `out`, jadi `out` harus tetap hidup saat dipakai.
pub fn resolve_path<'a>(
    cwd: &str,
    input: &str,
    out: &'a mut [u8; MAX_PATH_LEN],
) -> Option<&'a str> {
    let mut ctx = PathCtx::new();
    let base: &str = if input.starts_with('/') {
        ""
    } else {
        cwd
    };
    if !ctx.push_all(base, out) {
        return None;
    }
    if !ctx.push_all(input, out) {
        return None;
    }
    if ctx.len == 0 {
        out[0] = b'/';
        ctx.len = 1;
    }
    core::str::from_utf8(&out[..ctx.len]).ok()
}

struct PathCtx {
    len: usize,
    depth: usize,
    cuts: [usize; MAX_PATH_DEPTH],
}

impl PathCtx {
    fn new() -> Self {
        Self {
            len: 0,
            depth: 0,
            cuts: [0usize; MAX_PATH_DEPTH],
        }
    }

    fn push_all(&mut self, src: &str, out: &mut [u8; MAX_PATH_LEN]) -> bool {
        for seg in src.split('/') {
            if seg.is_empty() || seg == "." {
                continue;
            }
            if seg == ".." {
                if self.depth > 0 {
                    self.depth -= 1;
                    self.len = self.cuts[self.depth];
                } else {
                    self.len = 0;
                }
                continue;
            }
            if self.depth >= MAX_PATH_DEPTH {
                return false;
            }
            let sb = seg.as_bytes();
            let need = 1 + sb.len();
            if self.len + need > MAX_PATH_LEN {
                return false;
            }
            self.cuts[self.depth] = self.len;
            out[self.len] = b'/';
            let mut i = 0;
            while i < sb.len() {
                out[self.len + 1 + i] = sb[i];
                i += 1;
            }
            self.len += need;
            self.depth += 1;
        }
        true
    }
}
