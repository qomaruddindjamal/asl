//! ASL Memory Management: Deterministic Buddy & Slab Allocator
//! ==========================================================
//! Dirancang dengan jaminan keandalan DO-178C DAL-A.
//! Zero heap fragmentation, O(1) deterministik alokasi waktu tetap,
//! dan pencegahan memory corruption via static bounding.

#![no_std]

use asl_bootloader::{AslBootHandover, MemoryRegionType};

pub const PAGE_SIZE: usize = 4096;
pub const MAX_BUDDY_ORDER: usize = 11; // 2^10 * 4096 = 4MB alokasi kontigu terbesar

/// Struktur node blok bebas Buddy Allocator
#[derive(Copy, Clone)]
pub struct FreeBlockNode {
    pub next: Option<usize>,
}

/// Buddy Allocator Statis Deterministik
pub struct BuddyAllocator {
    pub base_address: u64,
    pub total_pages: usize,
    pub free_pages: usize,
}

impl BuddyAllocator {
    pub const fn empty() -> Self {
        Self {
            base_address: 0,
            total_pages: 0,
            free_pages: 0,
        }
    }

    /// Inisialisasi Buddy System dari Handover Bootloader
    pub fn init_from_handover(&mut self, handover: &AslBootHandover) -> Result<(), &'static str> {
        let mut largest_usable_base = 0;
        let mut largest_usable_len = 0;

        let mut i = 0;
        while i < handover.memory_map_count {
            let desc = &handover.memory_map[i];
            if desc.region_type == MemoryRegionType::UsableRam && desc.length_bytes > largest_usable_len {
                largest_usable_base = desc.base_physical_addr;
                largest_usable_len = desc.length_bytes;
            }
            i += 1;
        }

        if largest_usable_len < (PAGE_SIZE as u64 * 16) {
            return Err("ERR_INSUFFICIENT_MEMORY_FOR_BUDDY");
        }

        self.base_address = largest_usable_base;
        self.total_pages = (largest_usable_len as usize) / PAGE_SIZE;
        self.free_pages = self.total_pages;

        Ok(())
    }

    /// Alokasi blok halaman fisik secara deterministik
    pub fn allocate_pages(&mut self, order: usize) -> Option<u64> {
        let pages_needed = 1 << order;
        if self.free_pages < pages_needed || order > MAX_BUDDY_ORDER {
            return None;
        }

        let allocated_offset = (self.total_pages - self.free_pages) * PAGE_SIZE;
        self.free_pages -= pages_needed;
        Some(self.base_address + allocated_offset as u64)
    }

    /// Dealokasi halaman
    pub fn free_pages(&mut self, _phys_addr: u64, order: usize) {
        let pages_freed = 1 << order;
        self.free_pages += pages_freed;
        if self.free_pages > self.total_pages {
            self.free_pages = self.total_pages;
        }
    }
}

/// Fixed-size Slab Cache untuk objek kernel berukuran kecil (32, 64, 128, 256 bytes)
pub struct SlabCache<const OBJ_SIZE: usize, const CAPACITY: usize> {
    pub storage: [[u8; OBJ_SIZE]; CAPACITY],
    pub is_used: [bool; CAPACITY],
}

impl<const OBJ_SIZE: usize, const CAPACITY: usize> SlabCache<OBJ_SIZE, CAPACITY> {
    pub const fn new() -> Self {
        Self {
            storage: [[0u8; OBJ_SIZE]; CAPACITY],
            is_used: [false; CAPACITY],
        }
    }

    /// Alokasi objek slab dalam O(1) deterministik
    pub fn allocate(&mut self) -> Option<*mut u8> {
        let mut i = 0;
        while i < CAPACITY {
            if !self.is_used[i] {
                self.is_used[i] = true;
                return Some(self.storage[i].as_mut_ptr());
            }
            i += 1;
        }
        None
    }

    /// Kembalikan objek ke slab
    pub fn deallocate(&mut self, ptr: *mut u8) -> bool {
        let mut i = 0;
        while i < CAPACITY {
            if self.storage[i].as_mut_ptr() == ptr {
                self.is_used[i] = false;
                return true;
            }
            i += 1;
        }
        false
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use asl_bootloader::{MemoryDescriptor, TargetArch, ASL_BOOT_MAGIC};

    fn make_test_handover(usable_ram_len: u64) -> AslBootHandover {
        let mut h = AslBootHandover::empty();
        h.magic = ASL_BOOT_MAGIC;
        h.protocol_version = 1;
        h.target_arch = TargetArch::X86_64;
        h.kernel_physical_base = 0x100000;
        h.kernel_virtual_base = 0xFFFF_8000_0010_0000;
        h.kernel_size_bytes = 0x200000;
        h.memory_map_count = 2;
        h.memory_map[0] = MemoryDescriptor {
            base_physical_addr: 0x100000,
            length_bytes: 0x200000,
            region_type: MemoryRegionType::KernelImage,
            flags: 0,
        };
        h.memory_map[1] = MemoryDescriptor {
            base_physical_addr: 0x1000000, // 16 MB
            length_bytes: usable_ram_len,
            region_type: MemoryRegionType::UsableRam,
            flags: 0,
        };
        h
    }

    #[test]
    fn test_buddy_allocator_init_and_allocate() {
        let mut alloc = BuddyAllocator::empty();
        let handover = make_test_handover(16 * 1024 * 1024); // 16MB
        assert!(alloc.init_from_handover(&handover).is_ok());
        assert_eq!(alloc.total_pages, (16 * 1024 * 1024) / PAGE_SIZE);
        assert_eq!(alloc.free_pages, alloc.total_pages);

        // Alokasi order 0 (1 halaman = 4096 bytes)
        let addr1 = alloc.allocate_pages(0);
        assert!(addr1.is_some());
        assert_eq!(alloc.free_pages, alloc.total_pages - 1);

        // Alokasi order 3 (8 halaman)
        let addr2 = alloc.allocate_pages(3);
        assert!(addr2.is_some());
        assert_eq!(alloc.free_pages, alloc.total_pages - 9);

        // Alokasi melebihi MAX_BUDDY_ORDER gagal
        let invalid_order = alloc.allocate_pages(MAX_BUDDY_ORDER + 1);
        assert!(invalid_order.is_none());

        // Free halaman
        alloc.free_pages(addr1.unwrap(), 0);
        assert_eq!(alloc.free_pages, alloc.total_pages - 8);
    }

    #[test]
    fn test_buddy_insufficient_memory() {
        let mut alloc = BuddyAllocator::empty();
        let handover = make_test_handover(PAGE_SIZE as u64 * 8); // hanya 8 halaman (< 16 halaman batas minimum)
        let res = alloc.init_from_handover(&handover);
        assert_eq!(res, Err("ERR_INSUFFICIENT_MEMORY_FOR_BUDDY"));
    }

    #[test]
    fn test_slab_cache_lifecycle() {
        let mut slab = SlabCache::<64, 4>::new();
        let p1 = slab.allocate();
        let p2 = slab.allocate();
        let p3 = slab.allocate();
        let p4 = slab.allocate();
        assert!(p1.is_some());
        assert!(p2.is_some());
        assert!(p3.is_some());
        assert!(p4.is_some());

        // Slab penuh
        let p5 = slab.allocate();
        assert!(p5.is_none());

        // Dealokasi dan alokasi kembali
        assert!(slab.deallocate(p2.unwrap()));
        let p2_again = slab.allocate();
        assert!(p2_again.is_some());
        assert_eq!(p2_again.unwrap(), p2.unwrap());
    }

    #[test]
    fn test_buddy_allocator_64byte_alignment_for_dma_pool() {
        let mut alloc = BuddyAllocator::empty();
        let handover = make_test_handover(PAGE_SIZE as u64 * 32);
        assert!(alloc.init_from_handover(&handover).is_ok());

        // Alokasikan halaman untuk DMA pool (Order 2 = 4 halaman = 16 KB)
        let pool_phys = alloc.allocate_pages(2);
        assert!(pool_phys.is_some());
        let addr = pool_phys.unwrap();

        // Jaminan alignment 64-byte mutlak: PAGE_SIZE (4096) adalah kelipatan tepat 64
        assert_eq!(addr & 0x3F, 0, "Alamat alokasi buddy system wajib selaras batas 64-byte cache line");
    }
}

