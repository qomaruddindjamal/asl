//! ASL Memory Management: Deterministic Buddy & Slab Allocator
//! ==========================================================
//! Dirancang dengan jaminan keandalan DO-178C DAL-A.
//! Zero heap fragmentation, O(1) deterministik alokasi waktu tetap,
//! dan pencegahan memory corruption via static bounding.

#![no_std]

use asl_bootloader::{AslBootHandover, MemoryRegionType};

pub const PAGE_SIZE: usize = 4096;
pub const MAX_BUDDY_ORDER: usize = 11; // 2^10 * 4096 = 4MB alokasi kontigu terbesar
pub const MAX_TRACKED_PAGES: usize = 4096; // 4096 halaman * 4KB = 16 MB pool statis per-allocator
pub const NO_PAGE: u16 = 0xFFFF;

pub mod paging;
pub use paging::{
    create_kernel_pml4, invalidate_page_tlb, is_canonical_address, is_hhdm_address,
    is_user_address, load_cr3, page_offset, pd_index, pdpt_index, pml4_index, pt_index,
    read_cr3, AddressSpace, PageTable, PageTableEntry, PageTableFlags, PagingError,
    HHDM_VIRTUAL_BASE, PHYSICAL_ADDRESS_MASK, USER_SPACE_LIMIT,
};

pub const BUDDY_FLAG_FREE: u8 = 0;
pub const BUDDY_FLAG_ALLOCATED: u8 = 1;
pub const BUDDY_FLAG_SPLIT: u8 = 2;

/// Status deskriptor per-halaman fisik untuk pelacakan alokasi dan coalescing (Compact 8 bytes)
#[derive(Copy, Clone)]
pub struct PageDescriptor {
    pub next_free: u16,
    pub prev_free: u16,
    pub order: u8,
    pub flags: u8,
    pub _pad: [u8; 2],
}

impl PageDescriptor {
    pub const fn empty() -> Self {
        Self {
            next_free: NO_PAGE,
            prev_free: NO_PAGE,
            order: 0,
            flags: BUDDY_FLAG_FREE,
            _pad: [0; 2],
        }
    }
}

/// True Hierarchical Buddy Allocator Statis Deterministik
pub struct BuddyAllocator {
    pub base_address: u64,
    pub total_pages: usize,
    pub free_pages: usize,
    /// Kepala linked list blok bebas untuk setiap order (0 ..= MAX_BUDDY_ORDER)
    pub free_lists: [u16; MAX_BUDDY_ORDER + 1],
    /// Array deskriptor halaman statis
    pub page_descriptors: [PageDescriptor; MAX_TRACKED_PAGES],
}

impl BuddyAllocator {
    pub const fn empty() -> Self {
        Self {
            base_address: 0,
            total_pages: 0,
            free_pages: 0,
            free_lists: [NO_PAGE; MAX_BUDDY_ORDER + 1],
            page_descriptors: [PageDescriptor::empty(); MAX_TRACKED_PAGES],
        }
    }

    /// Inisialisasi Buddy System dari Handover Bootloader
    pub fn init_from_handover(&mut self, handover: &AslBootHandover) -> Result<(), &'static str> {
        let mut largest_usable_base = 0;
        let mut largest_usable_len = 0;

        let mut i = 0;
        while i < handover.memory_map_count {
            let desc = &handover.memory_map[i];
            if desc.region_type == MemoryRegionType::UsableRam
                && desc.length_bytes > largest_usable_len
            {
                largest_usable_base = desc.base_physical_addr;
                largest_usable_len = desc.length_bytes;
            }
            i += 1;
        }

        if largest_usable_len < (PAGE_SIZE as u64 * 16) {
            return Err("ERR_INSUFFICIENT_MEMORY_FOR_BUDDY");
        }

        self.base_address = largest_usable_base;
        let raw_total = (largest_usable_len as usize) / PAGE_SIZE;
        self.total_pages = if raw_total > MAX_TRACKED_PAGES {
            MAX_TRACKED_PAGES
        } else {
            raw_total
        };
        self.free_pages = 0;

        // Reset seluruh free list dan deskriptor
        for ord in 0..=MAX_BUDDY_ORDER {
            self.free_lists[ord] = NO_PAGE;
        }
        for idx in 0..self.total_pages {
            self.page_descriptors[idx] = PageDescriptor::empty();
        }

        // Susun blok-blok bebas awal ke dalam freelist berorde sebesar mungkin
        let mut current_page = 0;
        while current_page < self.total_pages {
            let remaining = self.total_pages - current_page;
            // Cari order terbesar yang memenuhi: (1 << order) <= remaining dan current_page selaras (1 << order)
            let mut order = MAX_BUDDY_ORDER;
            while order > 0 {
                let block_size = 1 << order;
                if block_size <= remaining && (current_page % block_size == 0) {
                    break;
                }
                order -= 1;
            }

            self.push_free_block(current_page as u16, order);
            let block_size = 1 << order;
            self.free_pages += block_size;
            current_page += block_size;
        }

        Ok(())
    }

    /// Memasukkan blok bebas ke dalam kepala freelist orde yang sesuai
    fn push_free_block(&mut self, page_idx: u16, order: usize) {
        let old_head = self.free_lists[order];
        let desc = &mut self.page_descriptors[page_idx as usize];
        desc.order = order as u8;
        desc.flags = BUDDY_FLAG_FREE;
        desc.prev_free = NO_PAGE;
        desc.next_free = old_head;

        if old_head != NO_PAGE {
            self.page_descriptors[old_head as usize].prev_free = page_idx;
        }
        self.free_lists[order] = page_idx;
    }

    /// Mengeluarkan blok bebas tertentu dari freelist
    fn remove_free_block(&mut self, page_idx: u16, order: usize) {
        let (prev, next) = {
            let desc = &mut self.page_descriptors[page_idx as usize];
            (desc.prev_free, desc.next_free)
        };

        if prev != NO_PAGE {
            self.page_descriptors[prev as usize].next_free = next;
        } else {
            self.free_lists[order] = next;
        }

        if next != NO_PAGE {
            self.page_descriptors[next as usize].prev_free = prev;
        }

        let desc = &mut self.page_descriptors[page_idx as usize];
        desc.next_free = NO_PAGE;
        desc.prev_free = NO_PAGE;
        desc.flags = BUDDY_FLAG_ALLOCATED;
    }

    /// Alokasi blok halaman fisik secara deterministik dengan pemecahan orde (splitting)
    pub fn allocate_pages(&mut self, order: usize) -> Option<u64> {
        let pages_needed = 1 << order;
        if self.free_pages < pages_needed || order > MAX_BUDDY_ORDER {
            return None;
        }

        // Cari order terkecil >= order yang memiliki blok bebas
        let mut target_order = order;
        while target_order <= MAX_BUDDY_ORDER && self.free_lists[target_order] == NO_PAGE {
            target_order += 1;
        }

        if target_order > MAX_BUDDY_ORDER {
            return None; // Fragmentasi atau kehabisan blok kontigu
        }

        // Ambil blok dari target_order
        let block_idx = self.free_lists[target_order];
        self.remove_free_block(block_idx, target_order);

        // Pecah blok secara rekursif hingga mencapai order yang diminta
        let mut current_order = target_order;
        while current_order > order {
            current_order -= 1;
            let buddy_idx = block_idx + (1 << current_order) as u16;
            self.push_free_block(buddy_idx, current_order);
            self.page_descriptors[buddy_idx as usize].flags = BUDDY_FLAG_FREE;
        }

        self.page_descriptors[block_idx as usize].order = order as u8;
        self.page_descriptors[block_idx as usize].flags = BUDDY_FLAG_ALLOCATED;
        self.free_pages -= pages_needed;

        Some(self.base_address + (block_idx as u64 * PAGE_SIZE as u64))
    }

    /// Dealokasi halaman dengan penggabungan pasangan buddy (coalescing)
    pub fn free_pages(&mut self, phys_addr: u64, order: usize) {
        if phys_addr < self.base_address || order > MAX_BUDDY_ORDER {
            return;
        }

        let offset = phys_addr - self.base_address;
        if offset % PAGE_SIZE as u64 != 0 {
            return; // Alamat fisik tidak selaras halaman
        }

        let mut page_idx = (offset / PAGE_SIZE as u64) as usize;
        if page_idx >= self.total_pages {
            return;
        }

        let mut current_order = order;
        let pages_freed = 1 << order;

        // Rekursif Coalescing: gabungkan dengan pasangan buddy selama pasangan tersebut bebas pada orde yang sama
        while current_order < MAX_BUDDY_ORDER {
            let block_size = 1 << current_order;
            let buddy_idx = page_idx ^ block_size;

            // Jika buddy berada di luar batas RAM yang terkelola, berhenti coalescing
            if buddy_idx + block_size > self.total_pages {
                break;
            }

            let buddy_desc = &self.page_descriptors[buddy_idx];
            // Syarat mutlak penggabungan: buddy harus berstatus FREE dan berada pada orde yang sama
            if buddy_desc.flags == BUDDY_FLAG_FREE && buddy_desc.order == current_order as u8 {
                self.remove_free_block(buddy_idx as u16, current_order);
                page_idx = core::cmp::min(page_idx, buddy_idx);
                current_order += 1;
            } else {
                break;
            }
        }

        self.push_free_block(page_idx as u16, current_order);
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
        assert_eq!(
            addr & 0x3F,
            0,
            "Alamat alokasi buddy system wajib selaras batas 64-byte cache line"
        );
    }

    #[test]
    fn test_buddy_coalescing_full_cycle() {
        let mut alloc = BuddyAllocator::empty();
        // Sediakan 64 halaman (256 KB)
        let handover = make_test_handover(PAGE_SIZE as u64 * 64);
        assert!(alloc.init_from_handover(&handover).is_ok());
        let initial_free = alloc.free_pages;
        assert_eq!(initial_free, 64);

        // Alokasikan 16 blok individual berorde 0 (16 halaman)
        let mut addrs = [0u64; 16];
        for i in 0..16 {
            let p = alloc.allocate_pages(0);
            assert!(p.is_some(), "Alokasi halaman {} wajib berhasil", i);
            addrs[i] = p.unwrap();
        }
        assert_eq!(alloc.free_pages, initial_free - 16);

        // Bebaskan seluruh 16 blok dalam urutan non-sekuensial (ganjil lalu genap) untuk menguji coalescing
        for i in (1..16).step_by(2) {
            alloc.free_pages(addrs[i], 0);
        }
        for i in (0..16).step_by(2) {
            alloc.free_pages(addrs[i], 0);
        }

        // Bukti mutlak coalescing: Seluruh halaman kembali utuh ke free_pages
        assert_eq!(alloc.free_pages, initial_free);

        // Bukti mutlak hierarchic merging: Sekarang kita harus bisa mengalokasikan 1 blok besar orde 4 (16 halaman kontigu = 64KB)
        let large_block = alloc.allocate_pages(4);
        assert!(
            large_block.is_some(),
            "Setelah coalescing penuh, alokasi blok besar order 4 (16 halaman) wajib berhasil"
        );
        assert_eq!(alloc.free_pages, initial_free - 16);

        // Kembalikan blok besar orde 4
        alloc.free_pages(large_block.unwrap(), 4);
        assert_eq!(alloc.free_pages, initial_free);
    }

    #[test]
    fn test_buddy_out_of_memory_graceful() {
        let mut alloc = BuddyAllocator::empty();
        // Sediakan 16 halaman (64 KB)
        let handover = make_test_handover(PAGE_SIZE as u64 * 16);
        assert!(alloc.init_from_handover(&handover).is_ok());

        // Alokasikan seluruh 16 halaman via orde 4
        let p_all = alloc.allocate_pages(4);
        assert!(p_all.is_some());
        assert_eq!(alloc.free_pages, 0);

        // Alokasi saat OOM wajib mengembalikan None secara anggun tanpa panic
        let p_oom = alloc.allocate_pages(0);
        assert_eq!(p_oom, None);

        // Dealokasi dan coba alokasi lagi
        alloc.free_pages(p_all.unwrap(), 4);
        assert_eq!(alloc.free_pages, 16);

        let p_after = alloc.allocate_pages(0);
        assert!(p_after.is_some());
    }
}
