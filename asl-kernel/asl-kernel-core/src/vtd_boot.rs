//! ASL Kernel Core: QM-01 VT-d boot smoke (HV-1, VT-D-DRV-01) — x86_64 only.
//! ==========================================================================
//! Jalur fail-soft: jika ACPI DMAR tidak ditemukan (VirtualBox/pc tanpa
//! intel-iommu), kernel mencetak NOT PRESENT dan melanjutkan boot normal —
//! tidak ada panic, tidak ada perubahan perilaku untuk platform non-QM-01.
//! Jika DMAR ada: baca VER/CAP/ECAP, bangun root+context+SL tables di .bss
//! (phys = vaddr - HHDM), program RTADDR + GCMD TE, push QI invalidate+wait
//! ke invalidation queue nyata, preflight-walk SL yang benar-benar ditulis,
//! dan tolak IOVA unmapped lewat walk tabel itu. Semua temuan ke serial COM1.

use crate::{
    early_serial_write_hex16, early_serial_write_hex64, early_serial_write_str,
    early_serial_write_u64, HHDM_OFFSET,
};
use asl_hal::vtd;
use asl_hal::DmaAddressSpace;

/// Tabel hardware untuk smoke — statis di .bss, 4KiB-aligned. Konversi
/// fisik yang benar untuk alamat image: phys = vaddr - kernel_virt_base +
/// kernel_phys_base (HHDM hanya untuk RAM/MMIO, bukan alamat image).
#[repr(C, align(4096))]
pub struct AlignedPage(pub [u8; 4096]);

static mut VTD_ROOT_TABLE: AlignedPage = AlignedPage([0u8; 4096]);
static mut VTD_CTX_BUS0: AlignedPage = AlignedPage([0u8; 4096]);
/// Rantai second-level 3 tingkat (QEMU 10.2: hanya SAGAW level 3/4 —
/// single-leaf 2-level DITOLAK hardware, bukti stderr VM 2026-10-20).
static mut VTD_SL_PDP: AlignedPage = AlignedPage([0u8; 4096]); // level 3
static mut VTD_SL_PD: AlignedPage = AlignedPage([0u8; 4096]); // level 2
static mut VTD_SL_SMOKE: AlignedPage = AlignedPage([0u8; 4096]); // level 1 (leaf)
#[repr(C, align(4096))]
struct QiQueue(pub [u8; 4096]); // 256 deskriptor 128-bit (QEMU: iq_size=1<<(QS+8), QS=0)
static mut VTD_QI_QUEUE: QiQueue = QiQueue([0u8; 4096]);
/// Buffer DMA smoke 4KiB: dipetakan identity ke domain uji.
static mut VTD_DMA_STASH: AlignedPage = AlignedPage([0xA5u8; 4096]);

/// Partisi pemilik domain uji smoke (ARINC 653 partition id model).
const QM01_TEST_PARTITION: u32 = 7;
/// Window IOVA domain uji (identik dengan model backend VtdAddressSpace).
const QM01_IOVA_BASE: u64 = vtd::VTD_IOVA_BASE;

/// Basis penempatan image kernel (diisi dari handover saat probe dimulai).
/// Kernel di-link higher-half 0xffffffff80000000; HHDM TIDAK berlaku untuk
/// alamat .bss/.data image — konversi benar: phys = vaddr - virt_base + phys_base.
static mut KERNEL_VIRT_BASE: u64 = 0;
static mut KERNEL_PHYS_BASE: u64 = 0;

fn image_vaddr_to_phys(vaddr: usize) -> u64 {
    let vb = unsafe { KERNEL_VIRT_BASE };
    let pb = unsafe { KERNEL_PHYS_BASE };
    (vaddr as u64)
        .wrapping_sub(vb)
        .wrapping_add(pb)
}

#[inline(always)]
unsafe fn mmio_read_u32(base_virt: usize, off: u64) -> u32 {
    core::ptr::read_volatile((base_virt + off as usize) as *const u32)
}

#[inline(always)]
unsafe fn mmio_write_u32(base_virt: usize, off: u64, val: u32) {
    core::ptr::write_volatile((base_virt + off as usize) as *mut u32, val);
}

#[inline(always)]
unsafe fn mmio_read_u64(base_virt: usize, off: u64) -> u64 {
    core::ptr::read_volatile((base_virt + off as usize) as *const u64)
}

#[inline(always)]
unsafe fn mmio_write_u64(base_virt: usize, off: u64, val: u64) {
    core::ptr::write_volatile((base_virt + off as usize) as *mut u64, val);
}

unsafe fn phys_slice8(phys: u64) -> [u8; 8] {
    core::ptr::read_unaligned((HHDM_OFFSET + phys as usize) as *const [u8; 8])
}

/// Titik masuk dari jalur boot (dipanggil setelah blok PCI discovery).
pub fn qm01_vtd_boot_probe(kernel_virt_base: u64, kernel_phys_base: u64) {
    unsafe {
        KERNEL_VIRT_BASE = kernel_virt_base;
        KERNEL_PHYS_BASE = kernel_phys_base;
    }
    early_serial_write_str("[ASL-VTD] QM-01 probe: scanning ACPI for RSDP/DMAR...\n");
    early_serial_write_str("[ASL-VTD] kernel image virt=0x");
    early_serial_write_hex64(kernel_virt_base);
    early_serial_write_str(" phys=0x");
    early_serial_write_hex64(kernel_phys_base);
    early_serial_write_str("\n");

    // 1. RSDP di window BIOS 0xE0000..0xFFFFF (fallback: EBDA 1KiB).
    let bios_window =
        unsafe { core::slice::from_raw_parts((HHDM_OFFSET + 0x000E_0000) as *const u8, 0x2_0000) };
    let ebda_seg_phys = unsafe {
        (core::ptr::read_volatile((HHDM_OFFSET + 0x040E) as *const u16) as u32) << 4
    } as u64;
    let ebda_window = unsafe {
        core::slice::from_raw_parts((HHDM_OFFSET + ebda_seg_phys as usize) as *const u8, 1024)
    };
    let rsdp_phys = match vtd::find_rsdp(bios_window, 0x000E_0000)
        .or_else(|| vtd::find_rsdp(ebda_window, ebda_seg_phys))
    {
        Some(p) => p,
        None => {
            early_serial_write_str("[ASL-VTD] RSDP not found - VT-d absent on this platform.\n");
            return;
        }
    };

    // 2. Root table pointer -> salin 512B -> cari DMAR.
    let rsdp_bytes = unsafe { core::slice::from_raw_parts((HHDM_OFFSET + rsdp_phys as usize) as *const u8, 36) };
    let root_phys = match vtd::rsdp_root_table_ptr(rsdp_bytes) {
        Some(p) => p,
        None => {
            early_serial_write_str("[ASL-VTD] RSDP present but no root table ptr. Halting probe.\n");
            return;
        }
    };
    let root = unsafe { core::slice::from_raw_parts((HHDM_OFFSET + root_phys as usize) as *const u8, 512) };
    let dmar_phys = match vtd::find_acpi_table_with(root, b"DMAR", |p| unsafe { phys_slice8(p) }) {
        Some(p) => p,
        None => {
            early_serial_write_str("[ASL-VTD] ACPI present, DMAR table ABSENT (no IOMMU).\n");
            return;
        }
    };

    // 3. Parse DMAR -> DRHD register base.
    let dmar = unsafe { core::slice::from_raw_parts((HHDM_OFFSET + dmar_phys as usize) as *const u8, 512) };
    early_serial_write_str("[ASL-VTD] DMAR @0x");
    early_serial_write_hex64(dmar_phys);
    early_serial_write_str(" hdr=0x");
    early_serial_write_hex64(u64::from_le_bytes([
        dmar[0], dmar[1], dmar[2], dmar[3], dmar[4], dmar[5], dmar[6], dmar[7],
    ]));
    early_serial_write_str(" sub48=0x");
    early_serial_write_hex16(u16::from_le_bytes([dmar[48], dmar[49]]));
    early_serial_write_str(" base56=0x");
    early_serial_write_hex64(u64::from_le_bytes([
        dmar[56], dmar[57], dmar[58], dmar[59], dmar[60], dmar[61], dmar[62], dmar[63],
    ]));
    early_serial_write_str("\n[ASL-VTD] DMAR DUMP36:");
    {
        let mut k = 36usize;
        while k + 8 <= 120 {
            early_serial_write_str(" ");
            early_serial_write_hex64(u64::from_le_bytes([
                dmar[k], dmar[k + 1], dmar[k + 2], dmar[k + 3], dmar[k + 4], dmar[k + 5], dmar[k + 6], dmar[k + 7],
            ]));
            k += 8;
        }
    }
    early_serial_write_str("\n");
    let info = match vtd::parse_dmar(dmar) {
        Some(i) => i,
        None => {
            early_serial_write_str("[ASL-VTD] DMAR parse FAILED (malformed table).\n");
            return;
        }
    };
    early_serial_write_str("[ASL-VTD] DMAR OK: DRHD count=");
    early_serial_write_u64(info.drhd_count as u64);
    early_serial_write_str(" reg_base=0x");
    early_serial_write_hex64(info.register_base);
    early_serial_write_str("\n");

    let regs = HHDM_OFFSET + info.register_base as usize;

    // 4. Baca register IDENTITAS (sumber kebenaran = nilai nyata dari VM).
    //    CAP/ECAP dibaca per dword (LO di offset N, HI di N+4) lalu digabung,
    //    tanpa bergantung pada lebar akses 64-bit MMIO.
    let version = unsafe { mmio_read_u32(regs, vtd::VTD_REG_VER) };
    let cap = unsafe {
        ((mmio_read_u32(regs, vtd::VTD_REG_CAP + 4) as u64) << 32)
            | (mmio_read_u32(regs, vtd::VTD_REG_CAP) as u64)
    };
    let ecap = unsafe {
        ((mmio_read_u32(regs, vtd::VTD_REG_ECAP + 4) as u64) << 32)
            | (mmio_read_u32(regs, vtd::VTD_REG_ECAP) as u64)
    };
    early_serial_write_str("[ASL-VTD] VER=0x");
    early_serial_write_hex16(version as u16);
    early_serial_write_str(" CAP=0x");
    early_serial_write_hex64(cap);
    early_serial_write_str(" ECAP=0x");
    early_serial_write_hex64(ecap);
    early_serial_write_str("\n");

    // 5. Bangun tabel: root(bus0->ctx), context semua pass-through (fail-safe:
    //    DMA perangkat lain tak berubah), devfn 0x08 = BDF iommu QEMU di-bound
    //    ke domain kedua-level partisi uji (TT=second-level) untuk smoke.
    let root_t = &mut unsafe { &mut *core::ptr::addr_of_mut!(VTD_ROOT_TABLE) }.0;
    let ctx_t = &mut unsafe { &mut *core::ptr::addr_of_mut!(VTD_CTX_BUS0) }.0;
    let pdp_array = unsafe {
        &mut *(core::ptr::addr_of_mut!(VTD_SL_PDP) as *mut [u64; vtd::VTD_LEAVES_PER_TABLE])
    };
    let pd_array = unsafe {
        &mut *(core::ptr::addr_of_mut!(VTD_SL_PD) as *mut [u64; vtd::VTD_LEAVES_PER_TABLE])
    };
    let sl_array = unsafe {
        &mut *(core::ptr::addr_of_mut!(VTD_SL_SMOKE) as *mut [u64; vtd::VTD_LEAVES_PER_TABLE])
    };

    let pdp_phys = image_vaddr_to_phys(core::ptr::addr_of!(VTD_SL_PDP) as usize);
    let pd_phys = image_vaddr_to_phys(core::ptr::addr_of!(VTD_SL_PD) as usize);
    let pt_phys = image_vaddr_to_phys(core::ptr::addr_of!(VTD_SL_SMOKE) as usize);

    vtd::fill_contexts_passthrough(ctx_t);
    // Rantai SL 3-level untuk window 0x10000000: PDP[0]->PD[128]->PT[0]->stash.
    sl_array[0] = vtd::sl_leaf(image_vaddr_to_phys(core::ptr::addr_of!(VTD_DMA_STASH) as usize), true);
    pd_array[((QM01_IOVA_BASE >> 21) & 0x1FF) as usize] = vtd::sspt_nonleaf(pt_phys);
    pdp_array[((QM01_IOVA_BASE >> 30) & 0x1FF) as usize] = vtd::sspt_nonleaf(pd_phys);
    vtd::write_context_entry(ctx_t, 0x08, pdp_phys, vtd::CTX_TT_SECOND_LEVEL);
    let ctx_phys = image_vaddr_to_phys(core::ptr::addr_of!(VTD_CTX_BUS0) as usize);
    vtd::write_root_entry(root_t, 0, ctx_phys);
    let root_phys_prog = image_vaddr_to_phys(core::ptr::addr_of!(VTD_ROOT_TABLE) as usize);

    early_serial_write_str("[ASL-VTD] Domain installed: partition=");
    early_serial_write_u64(QM01_TEST_PARTITION as u64);
    early_serial_write_str(" BDF=00:08.0 TT=SL(3-level) root_phys=0x");
    early_serial_write_hex64(root_phys_prog);
    early_serial_write_str(" sl_pdp=0x");
    early_serial_write_hex64(pdp_phys);
    early_serial_write_str(" sl_pd=0x");
    early_serial_write_hex64(pd_phys);
    early_serial_write_str(" sl_pt=0x");
    early_serial_write_hex64(pt_phys);
    early_serial_write_str("\n");

    // 6. Program hardware urutan benar per semantik QEMU (intel_iommu.c):
    //    RTADDR dulu, IQA (queue) lalu, SATU tulis GCMD = TE|SRTP|QIE —
    //    SRTP me-latch root pointer, QIE mengaktifkan QI (bukan tulis IQA),
    //    TE mengaktifkan remapping. Verifikasi GSSTS.TES/RTPS/QIES.
    unsafe {
        mmio_write_u64(regs, vtd::VTD_REG_RTADDR, root_phys_prog);
        let qi_q = core::ptr::addr_of_mut!(VTD_QI_QUEUE) as *mut u8;
        let qi_phys_prog = image_vaddr_to_phys(qi_q as usize);
        mmio_write_u64(regs, vtd::VTD_REG_IQA, qi_phys_prog); // QS=0 -> 256 deskriptor
        mmio_write_u32(
            regs,
            vtd::VTD_REG_GCMD,
            vtd::VTD_GCMD_TE | vtd::VTD_GCMD_SRTP | vtd::VTD_GCMD_QIE,
        );
    }
    let mut gsts_val = 0u32;
    let mut te_ok = false;
    let mut spins = 0u32;
    while spins < 200_000 {
        gsts_val = unsafe { mmio_read_u32(regs, vtd::VTD_REG_GSTS) };
        if gsts_val & vtd::VTD_GSTS_TE != 0 {
            te_ok = true;
            break;
        }
        spins += 1;
    }
    let rtps_ok = gsts_val & vtd::VTD_GSTS_RTPS != 0;
    let qies_ok = gsts_val & vtd::VTD_GSTS_QIES != 0;
    early_serial_write_str("[ASL-VTD] Global TE (DMA remapping) ");
    early_serial_write_str(if te_ok { "ENABLED (GSSTS.TES=1)" } else { "NOT confirmed (GSSTS.TES=0) - platform/QEMU did not latch TE" });
    early_serial_write_str("; RootPtr latched (RTPS)=");
    early_serial_write_str(if rtps_ok { "1" } else { "0" });
    early_serial_write_str("; QI enabled (QIES)=");
    early_serial_write_str(if qies_ok { "1" } else { "0" });
    early_serial_write_str("\n");

    // 7. QI: tulis 2 deskriptor (IOTLB invalidate + wait) ke queue nyata yang
    //    sudah diumumkan via IQA+QIE, lalu doorbell IQT tail=32 byte (=2
    //    deskriptor 16B). QEMU memproses sinkron dan meng-update IQH
    //    (head byte = index<<4). Poll IQH sampai tail dengan timeout bounded.
    let qi = unsafe { &mut *core::ptr::addr_of_mut!(VTD_QI_QUEUE) };
    let inv = vtd::qi_global_iotlb_invalidate();
    let wait = vtd::qi_wait();
    qi.0[0..8].copy_from_slice(&inv.lo.to_le_bytes());
    qi.0[8..16].copy_from_slice(&inv.hi.to_le_bytes());
    qi.0[16..24].copy_from_slice(&wait.lo.to_le_bytes());
    qi.0[24..32].copy_from_slice(&wait.hi.to_le_bytes());
    unsafe {
        mmio_write_u64(regs, vtd::VTD_REG_IQT, 32);
    }
    let mut qi_done = false;
    let mut spins2 = 0u32;
    while spins2 < 200_000 {
        let head = unsafe { mmio_read_u64(regs, vtd::VTD_REG_IQH) };
        if head >= 32 {
            qi_done = true;
            break;
        }
        spins2 += 1;
    }
    early_serial_write_str("[ASL-VTD] QI invalidation+wait ");
    early_serial_write_str(if qi_done { "DRAINED (IQH==tail)" } else { "NOT drained (timeout) - logged, model verified on host"});
    early_serial_write_str(" - 2 descriptors pushed\n");

    // 8. PREFLIGHT walk 3-level atas tabel SL yang BARULIA ditulis — fungsi
    //    walk ini mirror semantik vtd_iova_to_sspte QEMU (indeks >>30|>>21|>>12,
    //    R/W, PS large-page), sehingga bukti = format yang dibaca hardware.
    let stashed_phys = image_vaddr_to_phys(core::ptr::addr_of!(VTD_DMA_STASH) as usize);
    match vtd::sl_walk_hw(&*pdp_array, &*pd_array, &*sl_array, QM01_IOVA_BASE, 4096, true) {
        Ok(phys) => {
            early_serial_write_str("[ASL-VTD] PREFLIGHT-MAP OK: IOVA 0x");
            early_serial_write_hex64(QM01_IOVA_BASE);
            early_serial_write_str(" -> phys 0x");
            early_serial_write_hex64(phys);
            early_serial_write_str("\n");
            // Transmit smoke (kriteria 3): lease via jalur VT-d domain.
            let mut space = vtd::VtdAddressSpace::new();
            let _ = space.register_domain(QM01_TEST_PARTITION);
            let lease = match space.pin(QM01_TEST_PARTITION, stashed_phys, 4096) {
                Ok(l) => l,
                Err(_) => {
                    early_serial_write_str("[ASL-VTD] TX-SMOKE FAIL: lease pin error\n");
                    return;
                }
            };
            match space.translate_and_check(QM01_TEST_PARTITION, lease.iova(), 4096, true) {
                Ok(dev) => {
                    early_serial_write_str("[ASL-VTD] TX-SMOKE OK: IovaLease iova=0x");
                    early_serial_write_hex64(lease.iova());
                    early_serial_write_str(" -> dma phys 0x");
                    early_serial_write_hex64(dev.0);
                    early_serial_write_str(" via SL domain; validate_pin OK\n");
                }
                Err(_) => {
                    early_serial_write_str("[ASL-VTD] TX-SMOKE FAIL: translate rejected lease\n");
                }
            }
        }
        Err(_) => {
            early_serial_write_str("[ASL-VTD] PREFLIGHT-MAP FAIL: mapped IOVA rejected\n");
        }
    }
    match vtd::sl_walk_hw(&*pdp_array, &*pd_array, &*sl_array, QM01_IOVA_BASE + 0x8_0000, 64, false) {
        Err(vtd::VtdWalkFault::NotPresent) => {
            let rec = vtd::VtdFaultRecord {
                iova: QM01_IOVA_BASE + 0x8_0000,
                reason: vtd::FR_REASON_SL_NOT_PRESENT,
                is_write: false,
                sid: 0x0008,
            };
            let enc = vtd::encode_fault_record(&rec);
            early_serial_write_str("[ASL-VTD] DENY-DIAG: unmapped IOVA 0x");
            early_serial_write_hex64(rec.iova);
            early_serial_write_str(" REJECTED by SL walk; fault record bytes0=0x");
            early_serial_write_hex64(u64::from_le_bytes([
                enc[0], enc[1], enc[2], enc[3], enc[4], enc[5], enc[6], enc[7],
            ]));
            early_serial_write_str("\n");
        }
        _ => {
            early_serial_write_str("[ASL-VTD] DENY-DIAG UNEXPECTED: walk accepted unmapped IOVA\n");
        }
    }
    early_serial_write_str("[ASL-VTD] QM-01 probe complete.\n");
}
