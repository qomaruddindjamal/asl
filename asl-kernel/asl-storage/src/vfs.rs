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
}
