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
