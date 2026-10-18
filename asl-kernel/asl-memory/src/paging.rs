//! ASL x86_64 Virtual Memory Manager & 4-Level Paging (PML4)
//! =========================================================
//! Standar DO-178C DAL-A deterministik tanpa alokasi heap tak terkontrol.
//! Menyediakan:
//! - PageTableEntry & PageTableFlags (PRESENT, WRITABLE, USER, NO_EXECUTE, dll)
//! - 4-Level Page Table: PML4 -> PDPT -> PD -> PT
//! - Address Space Layout: Higher Half Direct Map (HHDM) & User space boundary
//! - Memory mapping API: map_page, unmap_page, translate_virtual_address, query_flags
//! - Deterministic TLB Invalidation via `invlpg`
//! - Secondary Kernel Independent CR3 Architecture Isolation

pub const PAGE_SIZE: usize = 4096;
pub const ENTRY_COUNT: usize = 512;
pub const PHYSICAL_ADDRESS_MASK: u64 = 0x000F_FFFF_FFFF_F000;

pub const HHDM_VIRTUAL_BASE: u64 = 0xFFFF_8000_0000_0000;
pub const USER_SPACE_LIMIT: u64 = 0x0000_7FFF_FFFF_FFFF;

/// Flags untuk Page Table Entry x86_64
#[derive(Copy, Clone, Debug, PartialEq, Eq)]
pub struct PageTableFlags(pub u64);

impl PageTableFlags {
    pub const NONE: Self = Self(0);
    pub const PRESENT: Self = Self(1 << 0);
    pub const WRITABLE: Self = Self(1 << 1);
    pub const USER: Self = Self(1 << 2);
    pub const WRITE_THROUGH: Self = Self(1 << 3);
    pub const CACHE_DISABLE: Self = Self(1 << 4);
    pub const ACCESSED: Self = Self(1 << 5);
    pub const DIRTY: Self = Self(1 << 6);
    pub const HUGE_PAGE: Self = Self(1 << 7);
    pub const GLOBAL: Self = Self(1 << 8);
    pub const NO_EXECUTE: Self = Self(1 << 63);

    #[inline(always)]
    pub const fn contains(&self, other: Self) -> bool {
        (self.0 & other.0) == other.0
    }

    #[inline(always)]
    pub const fn union(&self, other: Self) -> Self {
        Self(self.0 | other.0)
    }

    #[inline(always)]
    pub const fn bits(&self) -> u64 {
        self.0
    }
}

/// 64-bit Page Table Entry
#[repr(transparent)]
#[derive(Copy, Clone, Debug, PartialEq, Eq)]
pub struct PageTableEntry(pub u64);

impl PageTableEntry {
    #[inline(always)]
    pub const fn empty() -> Self {
        Self(0)
    }

    #[inline(always)]
    pub const fn is_present(&self) -> bool {
        (self.0 & PageTableFlags::PRESENT.0) != 0
    }

    #[inline(always)]
    pub const fn is_user(&self) -> bool {
        (self.0 & PageTableFlags::USER.0) != 0
    }

    #[inline(always)]
    pub const fn is_writable(&self) -> bool {
        (self.0 & PageTableFlags::WRITABLE.0) != 0
    }

    #[inline(always)]
    pub const fn is_no_execute(&self) -> bool {
        (self.0 & PageTableFlags::NO_EXECUTE.0) != 0
    }

    #[inline(always)]
    pub const fn physical_address(&self) -> u64 {
        self.0 & PHYSICAL_ADDRESS_MASK
    }

    #[inline(always)]
    pub const fn flags(&self) -> PageTableFlags {
        PageTableFlags(self.0 & !PHYSICAL_ADDRESS_MASK)
    }

    #[inline(always)]
    pub fn set(&mut self, phys_addr: u64, flags: PageTableFlags) {
        self.0 = (phys_addr & PHYSICAL_ADDRESS_MASK) | flags.0;
    }

    #[inline(always)]
    pub fn clear(&mut self) {
        self.0 = 0;
    }
}

/// 512-entry x86_64 Page Table (align 4096 bytes)
#[repr(C, align(4096))]
#[derive(Copy, Clone)]
pub struct PageTable {
    pub entries: [PageTableEntry; ENTRY_COUNT],
}

impl PageTable {
    pub const fn empty() -> Self {
        Self {
            entries: [PageTableEntry::empty(); ENTRY_COUNT],
        }
    }

    pub fn zero(&mut self) {
        for entry in self.entries.iter_mut() {
            entry.clear();
        }
    }
}

/// Status kesalahan pemetaan halaman virtual
#[derive(Copy, Clone, Debug, PartialEq, Eq)]
pub enum PagingError {
    AddressNotAligned,
    NonCanonicalAddress,
    AlreadyMapped,
    NotMapped,
    HugePageNotSupported,
    AllocationFailed,
    FrameDeallocationFailed,
}

/// Pemecah indeks 4-level paging untuk alamat virtual 48-bit kanonikal
#[inline(always)]
pub fn pml4_index(virt: u64) -> usize {
    ((virt >> 39) & 0x1FF) as usize
}

#[inline(always)]
pub fn pdpt_index(virt: u64) -> usize {
    ((virt >> 30) & 0x1FF) as usize
}

#[inline(always)]
pub fn pd_index(virt: u64) -> usize {
    ((virt >> 21) & 0x1FF) as usize
}

#[inline(always)]
pub fn pt_index(virt: u64) -> usize {
    ((virt >> 12) & 0x1FF) as usize
}

#[inline(always)]
pub fn page_offset(virt: u64) -> u64 {
    virt & 0xFFF
}

/// Validasi apakah alamat virtual berada dalam representasi kanonikal x86_64 48-bit
#[inline(always)]
pub fn is_canonical_address(virt: u64) -> bool {
    let sign_extension = virt >> 47;
    sign_extension == 0 || sign_extension == 0x1FFFF
}

/// Validasi batas user space
#[inline(always)]
pub fn is_user_address(virt: u64) -> bool {
    virt <= USER_SPACE_LIMIT
}

/// Validasi Higher-Half Direct Map (HHDM)
#[inline(always)]
pub fn is_hhdm_address(virt: u64) -> bool {
    virt >= HHDM_VIRTUAL_BASE
}

/// Invalidate TLB untuk alamat virtual tertentu via instruksi `invlpg`
#[inline(always)]
pub unsafe fn invalidate_page_tlb(virt: u64) {
    #[cfg(all(target_arch = "x86_64", target_os = "none"))]
    core::arch::asm!(
        "invlpg [{}]",
        in(reg) virt,
        options(nostack, preserves_flags)
    );
    #[cfg(not(all(target_arch = "x86_64", target_os = "none")))]
    let _ = virt;
}

/// Baca register CR3 saat ini (Physical base address of active PML4)
#[inline(always)]
pub unsafe fn read_cr3() -> u64 {
    #[cfg(all(target_arch = "x86_64", target_os = "none"))]
    {
        let cr3: u64;
        core::arch::asm!("mov {}, cr3", out(reg) cr3, options(nomem, nostack, preserves_flags));
        cr3
    }
    #[cfg(not(all(target_arch = "x86_64", target_os = "none")))]
    0
}

/// Muat register CR3 dengan alamat fisik PML4 baru
#[inline(always)]
pub unsafe fn load_cr3(pml4_phys: u64) {
    #[cfg(all(target_arch = "x86_64", target_os = "none"))]
    core::arch::asm!(
        "mov cr3, {}",
        in(reg) pml4_phys,
        options(nostack, preserves_flags)
    );
    #[cfg(not(all(target_arch = "x86_64", target_os = "none")))]
    let _ = pml4_phys;
}

/// Virtual Address Space Manager x86_64
pub struct AddressSpace {
    pub pml4_phys: u64,
    pub hhdm_offset: u64,
}

impl AddressSpace {
    /// Membuat handle AddressSpace dari physical address PML4 dan HHDM offset
    pub const fn from_pml4_phys(pml4_phys: u64, hhdm_offset: u64) -> Self {
        Self {
            pml4_phys,
            hhdm_offset,
        }
    }

    /// Mendapatkan pointer virtual ke suatu struktur PageTable dari alamat fisiknya via HHDM
    #[inline(always)]
    pub unsafe fn table_at_phys<'a>(&self, phys: u64) -> &'a mut PageTable {
        let virt = self.hhdm_offset + (phys & PHYSICAL_ADDRESS_MASK);
        &mut *(virt as *mut PageTable)
    }

    /// Memetakan satu halaman virtual 4KB ke halaman fisik dengan flags yang ditentukan
    /// Menggunakan closure `alloc_frame` untuk mengalokasikan page table baru jika belum ada
    pub unsafe fn map_page<F>(
        &mut self,
        virt: u64,
        phys: u64,
        flags: PageTableFlags,
        mut alloc_frame: F,
    ) -> Result<(), PagingError>
    where
        F: FnMut() -> Option<u64>,
    {
        if (virt & 0xFFF) != 0 || (phys & 0xFFF) != 0 {
            return Err(PagingError::AddressNotAligned);
        }
        if !is_canonical_address(virt) {
            return Err(PagingError::NonCanonicalAddress);
        }

        let pml4 = self.table_at_phys(self.pml4_phys);
        let pml4_idx = pml4_index(virt);

        // 1. Traverse / Alokasikan PDPT
        if !pml4.entries[pml4_idx].is_present() {
            let new_frame = alloc_frame().ok_or(PagingError::AllocationFailed)?;
            let new_table = self.table_at_phys(new_frame);
            new_table.zero();
            // Entry intermediate mewarisi writable & user jika flags memerlukannya
            let intermediate_flags = PageTableFlags::PRESENT
                .union(PageTableFlags::WRITABLE)
                .union(if flags.contains(PageTableFlags::USER) {
                    PageTableFlags::USER
                } else {
                    PageTableFlags::NONE
                });
            pml4.entries[pml4_idx].set(new_frame, intermediate_flags);
        } else if flags.contains(PageTableFlags::USER) && !pml4.entries[pml4_idx].is_user() {
            let cur_flags = pml4.entries[pml4_idx].flags().union(PageTableFlags::USER);
            pml4.entries[pml4_idx].set(pml4.entries[pml4_idx].physical_address(), cur_flags);
        }

        let pdpt_phys = pml4.entries[pml4_idx].physical_address();
        let pdpt = self.table_at_phys(pdpt_phys);
        let pdpt_idx = pdpt_index(virt);

        // 2. Traverse / Alokasikan PD
        if !pdpt.entries[pdpt_idx].is_present() {
            let new_frame = alloc_frame().ok_or(PagingError::AllocationFailed)?;
            let new_table = self.table_at_phys(new_frame);
            new_table.zero();
            let intermediate_flags = PageTableFlags::PRESENT
                .union(PageTableFlags::WRITABLE)
                .union(if flags.contains(PageTableFlags::USER) {
                    PageTableFlags::USER
                } else {
                    PageTableFlags::NONE
                });
            pdpt.entries[pdpt_idx].set(new_frame, intermediate_flags);
        } else if flags.contains(PageTableFlags::USER) && !pdpt.entries[pdpt_idx].is_user() {
            let cur_flags = pdpt.entries[pdpt_idx].flags().union(PageTableFlags::USER);
            pdpt.entries[pdpt_idx].set(pdpt.entries[pdpt_idx].physical_address(), cur_flags);
        }

        let pd_phys = pdpt.entries[pdpt_idx].physical_address();
        let pd = self.table_at_phys(pd_phys);
        let pd_idx = pd_index(virt);

        // 3. Traverse / Alokasikan PT
        if !pd.entries[pd_idx].is_present() {
            let new_frame = alloc_frame().ok_or(PagingError::AllocationFailed)?;
            let new_table = self.table_at_phys(new_frame);
            new_table.zero();
            let intermediate_flags = PageTableFlags::PRESENT
                .union(PageTableFlags::WRITABLE)
                .union(if flags.contains(PageTableFlags::USER) {
                    PageTableFlags::USER
                } else {
                    PageTableFlags::NONE
                });
            pd.entries[pd_idx].set(new_frame, intermediate_flags);
        } else if flags.contains(PageTableFlags::USER) && !pd.entries[pd_idx].is_user() {
            let cur_flags = pd.entries[pd_idx].flags().union(PageTableFlags::USER);
            pd.entries[pd_idx].set(pd.entries[pd_idx].physical_address(), cur_flags);
        }

        let pt_phys = pd.entries[pd_idx].physical_address();
        let pt = self.table_at_phys(pt_phys);
        let pt_idx = pt_index(virt);

        // 4. Map Halaman 4KB pada Level 1 PT
        if pt.entries[pt_idx].is_present() {
            return Err(PagingError::AlreadyMapped);
        }

        pt.entries[pt_idx].set(phys, flags.union(PageTableFlags::PRESENT));

        // Invalidate TLB untuk mencegah stale translation
        invalidate_page_tlb(virt);

        Ok(())
    }

    /// Membatalkan pemetaan halaman virtual 4KB dan mengembalikan physical address-nya
    pub unsafe fn unmap_page(&mut self, virt: u64) -> Result<u64, PagingError> {
        if (virt & 0xFFF) != 0 {
            return Err(PagingError::AddressNotAligned);
        }
        if !is_canonical_address(virt) {
            return Err(PagingError::NonCanonicalAddress);
        }

        let pml4 = self.table_at_phys(self.pml4_phys);
        let pml4_idx = pml4_index(virt);
        if !pml4.entries[pml4_idx].is_present() {
            return Err(PagingError::NotMapped);
        }

        let pdpt = self.table_at_phys(pml4.entries[pml4_idx].physical_address());
        let pdpt_idx = pdpt_index(virt);
        if !pdpt.entries[pdpt_idx].is_present() {
            return Err(PagingError::NotMapped);
        }

        let pd = self.table_at_phys(pdpt.entries[pdpt_idx].physical_address());
        let pd_idx = pd_index(virt);
        if !pd.entries[pd_idx].is_present() {
            return Err(PagingError::NotMapped);
        }

        let pt = self.table_at_phys(pd.entries[pd_idx].physical_address());
        let pt_idx = pt_index(virt);
        if !pt.entries[pt_idx].is_present() {
            return Err(PagingError::NotMapped);
        }

        let phys = pt.entries[pt_idx].physical_address();
        pt.entries[pt_idx].clear();

        // Flush TLB
        invalidate_page_tlb(virt);

        Ok(phys)
    }

    /// Menerjemahkan alamat virtual ke alamat fisik
    pub unsafe fn translate_virtual_address(&self, virt: u64) -> Result<u64, PagingError> {
        if !is_canonical_address(virt) {
            return Err(PagingError::NonCanonicalAddress);
        }

        let pml4 = self.table_at_phys(self.pml4_phys);
        let pml4_idx = pml4_index(virt);
        if !pml4.entries[pml4_idx].is_present() {
            return Err(PagingError::NotMapped);
        }

        let pdpt = self.table_at_phys(pml4.entries[pml4_idx].physical_address());
        let pdpt_idx = pdpt_index(virt);
        if !pdpt.entries[pdpt_idx].is_present() {
            return Err(PagingError::NotMapped);
        }

        let pd = self.table_at_phys(pdpt.entries[pdpt_idx].physical_address());
        let pd_idx = pd_index(virt);
        if !pd.entries[pd_idx].is_present() {
            return Err(PagingError::NotMapped);
        }

        let pt = self.table_at_phys(pd.entries[pd_idx].physical_address());
        let pt_idx = pt_index(virt);
        if !pt.entries[pt_idx].is_present() {
            return Err(PagingError::NotMapped);
        }

        let phys_base = pt.entries[pt_idx].physical_address();
        Ok(phys_base + page_offset(virt))
    }

    /// Memeriksa flags izin untuk alamat virtual tertentu
    pub unsafe fn query_flags(&self, virt: u64) -> Result<PageTableFlags, PagingError> {
        if !is_canonical_address(virt) {
            return Err(PagingError::NonCanonicalAddress);
        }

        let pml4 = self.table_at_phys(self.pml4_phys);
        let pml4_idx = pml4_index(virt);
        if !pml4.entries[pml4_idx].is_present() {
            return Err(PagingError::NotMapped);
        }

        let pdpt = self.table_at_phys(pml4.entries[pml4_idx].physical_address());
        let pdpt_idx = pdpt_index(virt);
        if !pdpt.entries[pdpt_idx].is_present() {
            return Err(PagingError::NotMapped);
        }

        let pd = self.table_at_phys(pdpt.entries[pdpt_idx].physical_address());
        let pd_idx = pd_index(virt);
        if !pd.entries[pd_idx].is_present() {
            return Err(PagingError::NotMapped);
        }

        let pt = self.table_at_phys(pd.entries[pd_idx].physical_address());
        let pt_idx = pt_index(virt);
        if !pt.entries[pt_idx].is_present() {
            return Err(PagingError::NotMapped);
        }

        Ok(pt.entries[pt_idx].flags())
    }

    /// Kloning separuh ruang alamat kernel (indeks PML4 256..512) ke dalam PML4 baru.
    /// Pola ini mematuhi arsitektur isolasi CR3 untuk proses user dan secondary failover core,
    /// di mana kernel space (HHDM, text, data) dibagi secara terproteksi, sementara lower half
    /// (indeks 0..256) tetap terisolasi penuh.
    pub unsafe fn clone_kernel_space(&self, dest_pml4_phys: u64) {
        let src_pml4 = self.table_at_phys(self.pml4_phys);
        let dest_pml4 = self.table_at_phys(dest_pml4_phys);

        // Bersihkan lower half (User space 0..256)
        for i in 0..256 {
            dest_pml4.entries[i].clear();
        }

        // Kloning higher half (Kernel space 256..512)
        for i in 256..512 {
            dest_pml4.entries[i] = src_pml4.entries[i];
        }
    }
}

/// Membangun Kernel PML4 AddressSpace deterministik:
/// Memetakan Kernel Code, Kernel Data, HHDM, Framebuffer, dan Serial MMIO.
/// Mengkloning pemetaan higher half dari PML4 aktif saat ini (disediakan oleh Limine bootloader)
/// ke dalam PML4 baru yang terkelola penuh oleh ASL kernel.
pub unsafe fn create_kernel_pml4(
    dest_pml4_phys: u64,
    hhdm_offset: u64,
) -> AddressSpace {
    let current_cr3 = read_cr3();
    let current_pml4_phys = current_cr3 & PHYSICAL_ADDRESS_MASK;

    let dest_space = AddressSpace::from_pml4_phys(dest_pml4_phys, hhdm_offset);

    if current_pml4_phys != 0 {
        let current_space = AddressSpace::from_pml4_phys(current_pml4_phys, hhdm_offset);
        current_space.clone_kernel_space(dest_pml4_phys);
    }

    dest_space
}

/// Static Kernel Page Table Buffers untuk pembuatan bootstrap PML4 deterministik
pub static mut STATIC_KERNEL_PML4: PageTable = PageTable::empty();
pub static mut STATIC_KERNEL_PDPT: PageTable = PageTable::empty();
pub static mut STATIC_KERNEL_PD: PageTable = PageTable::empty();

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_canonical_address_validation() {
        // Lower half canonical
        assert!(is_canonical_address(0x0000_0000_0000_0000));
        assert!(is_canonical_address(0x0000_7FFF_FFFF_FFFF));
        assert!(is_user_address(0x0000_7FFF_FFFF_FFFF));

        // Non-canonical gap
        assert!(!is_canonical_address(0x0000_8000_0000_0000));
        assert!(!is_canonical_address(0xFFFF_7FFF_FFFF_FFFF));

        // Higher half canonical
        assert!(is_canonical_address(0xFFFF_8000_0000_0000));
        assert!(is_canonical_address(0xFFFF_FFFF_FFFF_FFFF));
        assert!(is_hhdm_address(0xFFFF_8000_0000_0000));
        assert!(!is_user_address(0xFFFF_8000_0000_0000));
    }

    #[test]
    fn test_page_table_indices_decomposition() {
        let virt = 0xFFFF_8000_0000_0000u64;
        assert_eq!(pml4_index(virt), 256);
        assert_eq!(pdpt_index(virt), 0);
        assert_eq!(pd_index(virt), 0);
        assert_eq!(pt_index(virt), 0);
        assert_eq!(page_offset(virt), 0);

        let sample_virt = 0x0000_7FFF_A012_3456u64;
        assert_eq!(page_offset(sample_virt), 0x456);
    }

    #[test]
    fn test_page_table_entry_flags_and_address() {
        let mut entry = PageTableEntry::empty();
        assert!(!entry.is_present());
        assert!(!entry.is_writable());
        assert!(!entry.is_user());
        assert!(!entry.is_no_execute());

        let phys = 0x1000_0000u64;
        let flags = PageTableFlags::PRESENT
            .union(PageTableFlags::WRITABLE)
            .union(PageTableFlags::NO_EXECUTE);

        entry.set(phys, flags);
        assert!(entry.is_present());
        assert!(entry.is_writable());
        assert!(!entry.is_user());
        assert!(entry.is_no_execute());
        assert_eq!(entry.physical_address(), phys);
        assert_eq!(entry.flags(), flags);
    }

    // Mock simulated page table allocator untuk unit test
    struct SyncCell<T>(core::cell::UnsafeCell<T>);
    unsafe impl<T> Sync for SyncCell<T> {}

    struct SimulatedRam {
        tables: [PageTable; 16],
    }
    static SIMULATED_RAM: SyncCell<SimulatedRam> = SyncCell(core::cell::UnsafeCell::new(SimulatedRam {
        tables: [PageTable::empty(); 16],
    }));

    #[test]
    fn test_address_space_mapping_and_translation() {
        let ram_ptr = SIMULATED_RAM.0.get();
        unsafe {
            for tbl in (*ram_ptr).tables.iter_mut() {
                tbl.zero();
            }
        }

        let pml4_phys = 0;
        let hhdm_offset = unsafe { (*ram_ptr).tables.as_ptr() as u64 };

        let mut space = AddressSpace::from_pml4_phys(pml4_phys, hhdm_offset);

        let mut next_alloc_idx = 1;
        let mut alloc_frame = || {
            if next_alloc_idx < 16 {
                let phys = (next_alloc_idx * core::mem::size_of::<PageTable>()) as u64;
                next_alloc_idx += 1;
                Some(phys)
            } else {
                None
            }
        };

        let target_virt = 0x0000_0000_0040_0000u64;
        let target_phys = 0x0000_0000_1000_0000u64;
        let flags = PageTableFlags::PRESENT
            .union(PageTableFlags::WRITABLE)
            .union(PageTableFlags::USER);

        // 1. Map page
        let res = unsafe { space.map_page(target_virt, target_phys, flags, &mut alloc_frame) };
        assert_eq!(res, Ok(()));

        // 2. Translate virtual address
        let translated = unsafe { space.translate_virtual_address(target_virt + 0x123) };
        assert_eq!(translated, Ok(target_phys + 0x123));

        // 3. Query flags
        let query_res = unsafe { space.query_flags(target_virt) };
        assert_eq!(query_res, Ok(flags));

        // 4. Double map rejected
        let double_map = unsafe { space.map_page(target_virt, target_phys, flags, &mut alloc_frame) };
        assert_eq!(double_map, Err(PagingError::AlreadyMapped));

        // 5. Unmap page
        let unmapped = unsafe { space.unmap_page(target_virt) };
        assert_eq!(unmapped, Ok(target_phys));

        // 6. Translate unmapped faults
        let trans_after = unsafe { space.translate_virtual_address(target_virt) };
        assert_eq!(trans_after, Err(PagingError::NotMapped));
    }

    #[test]
    fn test_clone_kernel_space_isolation() {
        let ram_ptr = SIMULATED_RAM.0.get();
        unsafe {
            for tbl in (*ram_ptr).tables.iter_mut() {
                tbl.zero();
            }
        }

        let pml4_src_phys = 0;
        let pml4_dest_phys = core::mem::size_of::<PageTable>() as u64;
        let hhdm_offset = unsafe { (*ram_ptr).tables.as_ptr() as u64 };

        let src_space = AddressSpace::from_pml4_phys(pml4_src_phys, hhdm_offset);

        // Pasang satu entri pada user space (lower half: index 0)
        let src_pml4 = unsafe { src_space.table_at_phys(pml4_src_phys) };
        src_pml4.entries[0].set(0x1000, PageTableFlags::PRESENT.union(PageTableFlags::USER));

        // Pasang satu entri pada kernel space (higher half: index 256)
        src_pml4.entries[256].set(0x2000, PageTableFlags::PRESENT.union(PageTableFlags::WRITABLE));

        // Lakukan kloning kernel space ke dest
        unsafe {
            src_space.clone_kernel_space(pml4_dest_phys);
        }

        let dest_pml4 = unsafe { src_space.table_at_phys(pml4_dest_phys) };

        // Lower half pada destination WAJIB bersih (zero) demi isolasi user process / secondary kernel
        assert_eq!(dest_pml4.entries[0].physical_address(), 0);
        assert!(!dest_pml4.entries[0].is_present());

        // Higher half pada destination terkloning sempurna
        assert!(dest_pml4.entries[256].is_present());
        assert_eq!(dest_pml4.entries[256].physical_address(), 0x2000);
        assert!(dest_pml4.entries[256].is_writable());
        assert!(!dest_pml4.entries[256].is_user());
    }

    #[test]
    fn test_paging_boundary_and_alignment_rejections() {
        let ram_ptr = SIMULATED_RAM.0.get();
        let hhdm_offset = unsafe { (*ram_ptr).tables.as_ptr() as u64 };
        let mut space = AddressSpace::from_pml4_phys(0, hhdm_offset);
        let mut dummy_alloc = || Some(0x1000);

        // Alamat unaligned
        let unaligned_virt = 0x1234_5001u64;
        let aligned_phys = 0x2000u64;
        let res1 = unsafe {
            space.map_page(
                unaligned_virt,
                aligned_phys,
                PageTableFlags::PRESENT,
                &mut dummy_alloc,
            )
        };
        assert_eq!(res1, Err(PagingError::AddressNotAligned));

        // Alamat non-canonical
        let non_canonical = 0x0000_8000_0000_0000u64;
        let res2 = unsafe {
            space.map_page(
                non_canonical,
                aligned_phys,
                PageTableFlags::PRESENT,
                &mut dummy_alloc,
            )
        };
        assert_eq!(res2, Err(PagingError::NonCanonicalAddress));
    }
}
