//! ASL HAL: Minimal Intel VT-d (DMA remapping) path — VT-D-DRV-01 / HV-1 (QM-01)
//! ============================================================================
//! Scope jujur fase 5B: jalur VT-d MINIMUM untuk smoke QEMU q35 `-device intel-iommu`.
//!  - Parsing ACPI: scan RSDP -> root table (RSDT/XSDT) -> DMAR -> DRHD register base.
//!  - Encoder tabel hardware format (root entry, context entry, second-level leaf),
//!    QI descriptor, dan fault record — pure functions atas byte array sehingga
//!    terverifikasi di host test TANPA hardware, dan dipakai identik oleh runtime
//!    (runtime menulis tabel static yang dialamatkan lewat HHDM).
//!  - Backend `VtdAddressSpace` (kontrak lease RND_08): domain per partisi,
//!    pin = pasang leaf SL + QI invalidate, translate = walk tabel SL,
//!    fault tercatat di ring (mode FRCD).
//!
//! BUKAN cakupan (dinyatakan eksplisit di register): interrupt remapping,
//! scalable mode/PASID, ATS, SR-IOV VF, fault recovery ISR. Device-DMA fault
//! injection via perangkat nyata = stretch goal QM-01 runtime.
//!
//! Konstanta register/opcode mengikuti Intel VT-d register layout & tabel fault
//! reason umum; nilai diverifikasi ulang lewat log probe QM-01 pada QEMU 10.2 —
//! jika probe runtime berbeda, log serial adalah sumber kebenaran.

use crate::dma_space::{
    DmaAddressFault, DmaAddressSpace, DmaDeviceAddress, DmaError, IovaLease,
};

/// ============================== REGISTER MAP ==============================
/// Offset MMIO relatif terhadap DRHD register_base.
/// SUMBER KEBENARAN: hw/i386/intel_iommu_internal.h QEMU upstream (dicocokkan
/// byte-for-byte terhadap dump register runtime QM-01, 2026-10-20) yang
/// mengikuti Intel VT-d register layout: VER 0x00, CAP 0x08, ECAP 0x10,
/// GCMD 0x18, GSTS 0x1C, RTADDR 0x20, CCMD 0x28, FSTS 0x34, FECTL 0x38,
/// IQH 0x80, IQT 0x88, IQA 0x90, ICS 0x9C.
pub const VTD_REG_VER: u64 = 0x00;
pub const VTD_REG_CAP: u64 = 0x08;
pub const VTD_REG_ECAP: u64 = 0x10;
pub const VTD_REG_GCMD: u64 = 0x18;
pub const VTD_REG_GSTS: u64 = 0x1c;
pub const VTD_REG_RTADDR: u64 = 0x20;
pub const VTD_REG_CCMD: u64 = 0x28;
pub const VTD_REG_FSTS: u64 = 0x34;
pub const VTD_REG_FECTL: u64 = 0x38;
pub const VTD_REG_FECT: u64 = 0x38; // alias kompat (deprecated)
pub const VTD_REG_IQH: u64 = 0x80;
pub const VTD_REG_IQT: u64 = 0x88;
pub const VTD_REG_IQA: u64 = 0x90;
pub const VTD_REG_ICS: u64 = 0x9c;

/// GCMD bit31 = global TE (DMA remapping) enable; GSSTS bit31 = status TE.
/// GCMD bit30 = SRTP (latch RTADDR ke root ptr); bit26 = QIE (aktifkan QI).
/// Sumber kebenaran: include/hw/i386/intel_iommu_internal.h QEMU upstream
/// (VTD_GCMD_TE/SRTP/QIE, VTD_GSTS_TES/RTPS/QIES) — cocok per-bit.
pub const VTD_GCMD_TE: u32 = 1 << 31;
pub const VTD_GCMD_SRTP: u32 = 1 << 30;
pub const VTD_GCMD_QIE: u32 = 1 << 26;
pub const VTD_GSTS_TE: u32 = 1 << 31;
pub const VTD_GSTS_RTPS: u32 = 1 << 30;
pub const VTD_GSTS_QIES: u32 = 1 << 26;

pub const VTD_PAGE_SIZE: u64 = 0x1000;
pub const VTD_PAGE_SHIFT: u32 = 12;
pub const VTD_LEAVES_PER_TABLE: usize = 512; // 1 page-table level: 512 x 4KiB = 2MiB

/// Translation-type field context entry, posisi raw bit lo[3:2].
/// SUMBER KEBENARAN: intel_iommu_internal.h QEMU upstream (VTD_CONTEXT_ENTRY_TT
/// = 3<<2; VTD_CONTEXT_TT_MULTI_LEVEL=0 = second-level walk,
/// VTD_CONTEXT_TT_PASS_THROUGH = 2<<2 = 8). Unmap diekspresikan P-bit=0.
pub const CTX_TT_SECOND_LEVEL: u8 = 0;
pub const CTX_TT_PASS_THROUGH: u8 = 8;
/// AGAW field hi[2:0]: 1 = 39-bit => SL level 3 (satu-satunya level minimum
/// yang diiklankan CAP.SAGAW QEMU; level 2 TIDAK didukung — terbukti dari
/// log stderr VM "level 2 not supported").
pub const CTX_AGAW_39BIT: u64 = 1;

/// Fault reason model (dipetakan ke field Reason record FRCD saat runtime).
pub const FR_REASON_SL_NOT_PRESENT: u8 = 0x04;
pub const FR_REASON_SL_PERMISSION: u8 = 0x08;

// -----------------------------------------------------------------------------
// 1. PARSING ACPI: RSDP -> RSDT/XSDT -> DMAR (pure atas slice; runtime memakai
//    slice yang sama lewat HHDM — tidak ada asumsi alokator)
// -----------------------------------------------------------------------------

/// Hasil parse DRHD pertama (segment 0) dari tabel DMAR.
#[derive(Debug, Copy, Clone, PartialEq, Eq)]
pub struct DmarInfo {
    pub register_base: u64,
    pub drhd_count: u8,
}

/// ACPI checksum: seluruh byte tabel harus menjumlah ke 0 mod 256.
pub fn acpi_checksum_ok(bytes: &[u8]) -> bool {
    let mut sum: u8 = 0;
    let mut i = 0;
    while i < bytes.len() {
        sum = sum.wrapping_add(bytes[i]);
        i += 1;
    }
    sum == 0
}

/// Cari RSDP pada window BIOS (umumnya 0xE0000..0xFFFFF, scan stride 16 byte).
/// `window_phys` adalah alamat fisik byte pertama window (untuk report fisik).
pub fn find_rsdp(window: &[u8], window_phys: u64) -> Option<u64> {
    let sig = *b"RSD PTR ";
    let mut off = 0usize;
    while off + 20 <= window.len() {
        if window[off..off + 8] == sig[..] && acpi_checksum_ok(&window[off..off + 20]) {
            return Some(window_phys + off as u64);
        }
        off += 16;
    }
    None
}

/// Root table pointer (RSDT untuk rev<2, XSDT untuk rev>=2) dari 36 byte RSDP.
pub fn rsdp_root_table_ptr(rsdp: &[u8]) -> Option<u64> {
    if rsdp.len() < 20 || rsdp[0..8] != *b"RSD PTR " {
        return None;
    }
    let rev = rsdp[15];
    if rev >= 2 && rsdp.len() >= 36 {
        let xsdt = rd_u64(rsdp, 24);
        if xsdt != 0 {
            return Some(xsdt);
        }
    }
    let rsdt = rd_u32(rsdp, 16);
    if rsdt != 0 {
        Some(rsdt as u64)
    } else {
        None
    }
}

/// Temukan entri tabel ber-signature `sig` dalam root table (RSDT 4-byte / XSDT
/// 8-byte pointers). Deteksi lebar entri dari signature header root table.
/// `read8` membaca 8 byte fisik pada address (runtime: slice HHDM; host test:
/// closure atas tabel signature statis) — pemurnikan fungsi agar host-testable.
pub fn find_acpi_table_with<F: Fn(u64) -> [u8; 8]>(
    root: &[u8],
    sig: &[u8; 4],
    read8: F,
) -> Option<u64> {
    if root.len() < 36 {
        return None;
    }
    let wide = root[0..4] == *b"XSDT";
    if !wide && root[0..4] != *b"RSDT" {
        return None;
    }
    let len = core::cmp::min(rd_u32(root, 4) as usize, root.len());
    let mut off = 36usize;
    while off + if wide { 8 } else { 4 } <= len {
        let ptr = if wide { rd_u64(root, off) } else { rd_u32(root, off) as u64 };
        if ptr != 0 && &read8(ptr)[0..4] == sig {
            return Some(ptr);
        }
        off += if wide { 8 } else { 4 };
    }
    None
}

/// Parse tabel DMAR: validasi signature, telusuri subtable DRHD, ambil
/// register_base DRHD pertama dengan segment 0.
///
/// Toleransi layout (empiris QEMU q35 acpi-build build_dmar_q35, 2026-10-20):
/// QEMU menulis Type sbg 2 byte lalu Length 2 byte (b48..51 = 00 00 48 00);
/// tabel classic/legacy menulis Length u8 di b49. Parser menerima keduanya:
/// bila byte+1 nol, length dibaca sebagai u16 di offset+2.
pub fn parse_dmar(dmar: &[u8]) -> Option<DmarInfo> {
    if dmar.len() < 48 || dmar[0..4] != *b"DMAR" {
        return None;
    }
    let total = core::cmp::min(rd_u32(dmar, 4) as usize, dmar.len());
    let mut off = 48usize;
    let mut found: Option<DmarInfo> = None;
    let mut count: u8 = 0;
    while off + 4 <= total {
        let stype = dmar[off];
        let slen = if dmar[off + 1] != 0 {
            dmar[off + 1] as usize
        } else if off + 4 <= total {
            rd_u16(dmar, off + 2) as usize
        } else {
            0
        };
        if slen < 4 || off + slen > total {
            break;
        }
        if stype == 0 {
            // DRHD: type, length, (spec/revision), flags, reserved, segment,
            // register base u64 @ off+8.
            count += 1;
            let segment = rd_u16(dmar, off + 6);
            let base = rd_u64(dmar, off + 8);
            if segment == 0 && found.is_none() && base != 0 {
                found = Some(DmarInfo { register_base: base, drhd_count: 0 });
            }
        }
        off += slen;
    }
    found.map(|mut d| {
        d.drhd_count = count;
        d
    })
}

// -----------------------------------------------------------------------------
// 2. ENCODER TABEL HARDWARE (pure; runtime menulis tabel static dengan fn sama)
//    Layout diverifikasi byte-level terhadap QEMU upstream (VTD_CONTEXT_ENTRY_*,
//    VTD_ROOT_ENTRY_*, vtd_iova_to_sspte) — lihat komentar tiap konstanta.
// -----------------------------------------------------------------------------

/// Root table entry: lo = (context_table_phys & ~0xFFF) | present(bit0); hi=0
/// (QEMU menuntut re->hi == 0 pada legacy mode, reserved check 1438).
pub fn root_entry_lo(context_table_phys: u64) -> u64 {
    (context_table_phys & !0xFFF) | 1
}

/// Tulis root entry (bus index) — stride entry 32 byte (QEMU: bus_num << 5),
/// bytes 16..32 tetap nol (bss) dan tidak dibaca pada legacy mode.
pub fn write_root_entry(table: &mut [u8; 4096], bus: usize, context_table_phys: u64) {
    let off = bus * 32;
    let lo = root_entry_lo(context_table_phys);
    table[off..off + 8].copy_from_slice(&lo.to_le_bytes());
    table[off + 8..off + 16].copy_from_slice(&0u64.to_le_bytes());
}

/// Context entry lo (QEMU 10.2): SSPTPTR = second-level ROOT table ptr di
/// lo[63:12] (VTD_CONTEXT_ENTRY_SSPTPTR = ~0xfff), TT di lo[3:2], P di bit0.
pub fn context_entry_lo(sl_root_table_phys: u64, tt: u8) -> u64 {
    (sl_root_table_phys & !0xFFF) | ((tt as u64) & 0xC) | 1
}

/// Context entry hi: AGAW di hi[2:0]; DID hi[23:8] (diisi 0 — smoke ini
/// memakai invalidasi GLOBAL sehingga DID tidak dipakai); bit7 + hi[63:40]
/// reserved harus 0 (VTD_CONTEXT_ENTRY_RSVD_HI).
pub fn context_entry_hi(did: u16, agaw: u64) -> u64 {
    (((did as u64) & 0xFF) << 8) | (agaw & 0x7)
}

/// Dekode field konteks untuk validasi walk-time/preflight.
/// tt yang dikembalikan = raw lo[3:2] sehingga langsung cocok dgn CTX_TT_*.
pub fn context_entry_fields(lo: u64) -> (bool, u8, u64) {
    (lo & 1 != 0, (lo & 0xC) as u8, lo & !0xFFF)
}

/// Tulis context entry utk devfn (0..256 = dev*8+func), 16B stride, SL root
/// pointer di lo, AGAW 39-bit di hi.
pub fn write_context_entry(table: &mut [u8; 4096], devfn: usize, sl_table_phys: u64, tt: u8) {
    let off = devfn * 16;
    let lo = context_entry_lo(sl_table_phys, tt);
    let hi = context_entry_hi(0, CTX_AGAW_39BIT);
    table[off..off + 8].copy_from_slice(&lo.to_le_bytes());
    table[off + 8..off + 16].copy_from_slice(&hi.to_le_bytes());
}

/// Isi seluruh 256 entri konteks dengan present|pass-through — fail-safe:
/// setiap perangkat yang tidak jadi subjek uji tetap DMA identity seperti
/// sebelum VT-d. Passthrough pun wajib AGAW valid (level check QEMU jalan
/// sebelum type check).
pub fn fill_contexts_passthrough(table: &mut [u8; 4096]) {
    for devfn in 0..256 {
        write_context_entry(table, devfn, 0, CTX_TT_PASS_THROUGH);
    }
}

/// SL leaf: (phys & ~0xFFF) | R(bit0) | W(bit1). present = R|W != 0.
pub fn sl_leaf(phys: u64, writable: bool) -> u64 {
    let mut v = (phys & !0xFFF) | 1;
    if writable {
        v |= 2;
    }
    v
}

pub fn sl_leaf_present(pte: u64) -> bool {
    pte & 0x3 != 0
}

pub fn sl_leaf_writable(pte: u64) -> bool {
    pte & 2 != 0
}

pub fn sl_leaf_phys(pte: u64) -> u64 {
    pte & !0xFFF
}

// -----------------------------------------------------------------------------
// 3. QUEUED INVALIDATION descriptor + FAULT record (model FRCD)
// -----------------------------------------------------------------------------

/// QI opcode (VTD_INV_DESC_* upstream QEMU / VT-d spec):
/// 1 = context-cache invalidate, 2 = IOTLB invalidate, 5 = invalidation wait.
/// Field granularitas IOTLB ada di lo[5:4]: 1=GLOBAL, 2=domain, 3=page
/// (VTD_INV_DESC_IOTLB_GLOBAL, QEMU) — tanpanya deskriptor DITOLAK hardware
/// ("type mismatch: 0x0" pada stderr VM run 2026-10-20).
pub const QI_OP_CONTEXT_INV: u64 = 1;
pub const QI_OP_IOTLB_GLOBAL_INV: u64 = 2;
pub const QI_OP_WAIT: u64 = 5;
pub const QI_INV_IOTLB_GLOBAL: u64 = 1 << 4;
/// WAIT descriptor wajib punya salah satu SW(bit5)/IF(bit4)/FN(bit6);
/// FN saja = no-op valid ("SW=0, IF=0, FN=1 is also a valid descriptor",
/// vtd_process_wait_desc QEMU — tanpa itu descriptor DITOLAK hardware).
pub const QI_WAIT_FN: u64 = 1 << 6;

#[derive(Debug, Copy, Clone, PartialEq, Eq)]
pub struct QiDesc {
    pub lo: u64,
    pub hi: u64,
}

pub const fn qi_global_iotlb_invalidate() -> QiDesc {
    QiDesc {
        lo: QI_OP_IOTLB_GLOBAL_INV | QI_INV_IOTLB_GLOBAL,
        hi: 0,
    }
}

pub const fn qi_context_invalidate() -> QiDesc {
    QiDesc { lo: QI_OP_CONTEXT_INV, hi: 0 }
}

pub const fn qi_wait() -> QiDesc {
    QiDesc {
        lo: QI_OP_WAIT | QI_WAIT_FN,
        hi: 0,
    }
}

#[derive(Debug, Copy, Clone, PartialEq, Eq)]
pub struct VtdFaultRecord {
    pub iova: u64,
    pub reason: u8,
    pub is_write: bool,
    pub sid: u16,
}

/// Enkode ke FRCD 16 byte: DW0/DW1 = iova; DW2 = reason<<4 | write<<3 | present;
/// DW3 = SID<<16 | count(1). Layout model — dikroscek terhadap QEMU saat runtime.
pub fn encode_fault_record(r: &VtdFaultRecord) -> [u8; 16] {
    let mut b = [0u8; 16];
    b[0..8].copy_from_slice(&r.iova.to_le_bytes());
    let dw2: u32 = ((r.reason as u32) << 4) | ((r.is_write as u32) << 3) | (1 << 2);
    b[8..12].copy_from_slice(&dw2.to_le_bytes());
    let dw3: u32 = (r.sid as u32) << 16 | 1;
    b[12..16].copy_from_slice(&dw3.to_le_bytes());
    b
}

pub fn decode_fault_record(b: &[u8; 16]) -> VtdFaultRecord {
    let iova = rd_u64(b, 0);
    let dw2 = rd_u32(b, 8);
    VtdFaultRecord {
        iova,
        reason: ((dw2 >> 4) & 0xFF) as u8,
        is_write: dw2 & (1 << 3) != 0,
        sid: (rd_u32(b, 12) >> 16) as u16,
    }
}

// -----------------------------------------------------------------------------
// 4. WALK SL table murni (dipakai backend + preflight runtime atas tabel nyata)
// -----------------------------------------------------------------------------

#[derive(Debug, Copy, Clone, PartialEq, Eq)]
pub enum VtdWalkFault {
    NotPresent,
    Permission,
    RangeOutOfBounds,
}

/// Walk satu level leaf table (domain window = base_iova..base_iova+2MiB).
/// Return alamat fisik untuk IOVA+size bila semua leaf present (dan W bila write).
pub fn sl_walk(
    table: &[u64; VTD_LEAVES_PER_TABLE],
    base_iova: u64,
    iova: u64,
    size: usize,
    is_write: bool,
) -> Result<u64, VtdWalkFault> {
    if size == 0 {
        return Err(VtdWalkFault::RangeOutOfBounds);
    }
    if iova < base_iova {
        return Err(VtdWalkFault::RangeOutOfBounds);
    }
    let off = iova - base_iova;
    let end = match off.checked_add(size as u64 - 1) {
        Some(e) => e,
        None => return Err(VtdWalkFault::RangeOutOfBounds),
    };
    if end >= (VTD_LEAVES_PER_TABLE as u64) * VTD_PAGE_SIZE {
        return Err(VtdWalkFault::RangeOutOfBounds);
    }
    let first = (off >> VTD_PAGE_SHIFT) as usize;
    let last = (end >> VTD_PAGE_SHIFT) as usize;
    let mut i = first;
    while i <= last {
        let pte = table[i];
        if !sl_leaf_present(pte) {
            return Err(VtdWalkFault::NotPresent);
        }
        if is_write && !sl_leaf_writable(pte) {
            return Err(VtdWalkFault::Permission);
        }
        i += 1;
    }
    Ok(sl_leaf_phys(table[first]) + (off & (VTD_PAGE_SIZE - 1)))
}

/// ===================== WALK 3-LEVEL MIRROR QEMU (HV-1 runtime) =============
/// QEMU 10.2 hanya mendukung SL level 3/4 (CAP.SAGAW); walk hardware nyata
/// memakai indeks (iova>>30|>>21|>>12)&0x1FF seperti vtd_iova_to_sspte.
/// Semantik present/permission: R(bit0) utk read, W(bit1) utk write; PS(bit7)
/// di L3/L1 => large page 1GiB/2MiB. `pdp`/`pd`/`pt` adalah TIGA halaman yang
/// benar-benar ditulis runtime; dipakai untuk preflight atas memori tabel
/// nyata sehingga bukti walk = bukti format yang dibaca hardware.
pub fn sl_walk_hw(
    pdp: &[u64; VTD_LEAVES_PER_TABLE],
    pd: &[u64; VTD_LEAVES_PER_TABLE],
    pt: &[u64; VTD_LEAVES_PER_TABLE],
    iova: u64,
    size: usize,
    is_write: bool,
) -> Result<u64, VtdWalkFault> {
    if size == 0 {
        return Err(VtdWalkFault::RangeOutOfBounds);
    }
    let end = match iova.checked_add(size as u64 - 1) {
        Some(e) => e,
        None => return Err(VtdWalkFault::RangeOutOfBounds),
    };
    let need: u64 = if is_write { 2 } else { 1 };
    let mut cur = iova & !(VTD_PAGE_SIZE - 1);
    let last = end & !(VTD_PAGE_SIZE - 1);
    let mut first_phys = 0u64;
    loop {
        let e3 = pdp[((cur >> 30) & 0x1FF) as usize];
        if e3 == 0 {
            return Err(VtdWalkFault::NotPresent);
        }
        if e3 & need == 0 {
            return Err(VtdWalkFault::Permission);
        }
        let phys = if e3 & SSPT_PS != 0 {
            (e3 & !(0x4000_0000 - 1)) + (cur & (0x4000_0000 - 1))
        } else {
            let e2 = pd[((cur >> 21) & 0x1FF) as usize];
            if e2 == 0 {
                return Err(VtdWalkFault::NotPresent);
            }
            if e2 & need == 0 {
                return Err(VtdWalkFault::Permission);
            }
            if e2 & SSPT_PS != 0 {
                (e2 & !(0x20_0000 - 1)) + (cur & (0x20_0000 - 1))
            } else {
                let e1 = pt[((cur >> 12) & 0x1FF) as usize];
                if e1 == 0 {
                    return Err(VtdWalkFault::NotPresent);
                }
                if e1 & need == 0 {
                    return Err(VtdWalkFault::Permission);
                }
                (e1 & !(VTD_PAGE_SIZE - 1)) + (cur & (VTD_PAGE_SIZE - 1))
            }
        };
        if cur == iova & !(VTD_PAGE_SIZE - 1) {
            first_phys = phys;
        }
        if cur >= last {
            break;
        }
        cur += VTD_PAGE_SIZE;
    }
    Ok(first_phys + (iova & (VTD_PAGE_SIZE - 1)))
}

/// PS (Page Size) bit entri second-level — large page di L3 (1GiB) / L2 (2MiB).
pub const SSPT_PS: u64 = 1 << 7;

/// Entri non-leaf L3/L2: pointer tabel level berikutnya + R|W.
pub fn sspt_nonleaf(next_table_phys: u64) -> u64 {
    (next_table_phys & !(VTD_PAGE_SIZE - 1)) | 3
}

/// Leaf 2MiB (L2 dengan PS=1); phys wajib 2MiB-aligned.
pub fn sspt_leaf_2m(phys: u64, writable: bool) -> u64 {
    let mut v = (phys & !(0x20_0000 - 1)) | SSPT_PS | 1;
    if writable {
        v |= 2;
    }
    v
}

// -----------------------------------------------------------------------------
// 5. BACKEND VtdAddressSpace — domain per partisi (kontrak lease RND_08)
// -----------------------------------------------------------------------------

pub const VTD_DOMAIN_COUNT: usize = 8;
pub const VTD_IOVA_BASE: u64 = 0x1000_0000;
pub const VTD_DOMAIN_WINDOW: u64 = (VTD_LEAVES_PER_TABLE as u64) * VTD_PAGE_SIZE; // 2MiB
pub const VTD_SLOT_COUNT: usize = 8;
pub const VTD_QI_RING: usize = 32;
pub const VTD_FAULT_RING: usize = 16;

#[derive(Debug, Copy, Clone)]
struct VtdDomain {
    in_use: bool,
    partition_id: u32,
    base_iova: u64,
    next_off: u64, // bump allocator window
    table: [u64; VTD_LEAVES_PER_TABLE],
}

const fn empty_domain() -> VtdDomain {
    VtdDomain {
        in_use: false,
        partition_id: 0,
        base_iova: 0,
        next_off: 0,
        table: [0; VTD_LEAVES_PER_TABLE],
    }
}

#[derive(Debug, Copy, Clone)]
struct VtdSlot {
    in_use: bool,
    domain: usize,
    iova: u64,
    len: u64,
    generation: u32,
    read_only: bool,
}

/// Backend VT-d model: menerjemahkan lewat second-level page table nyata
/// (bump allocator IOVA per domain, pin = pasang leaf + push QI invalidate).
pub struct VtdAddressSpace {
    domains: [VtdDomain; VTD_DOMAIN_COUNT],
    slots: [VtdSlot; VTD_SLOT_COUNT],
    qi_ring: [QiDesc; VTD_QI_RING],
    qi_count: usize,
    faults: [Option<VtdFaultRecord>; VTD_FAULT_RING],
    fault_count: usize,
    /// BDF (bus:dev.func <<16 | ...) sumber perangkat untuk fault record;
    /// runtime mengisinya dari context entry perangkat yang di-bound.
    origin_sid: u16,
}

impl VtdAddressSpace {
    pub const fn new() -> Self {
        Self {
            domains: [empty_domain(); VTD_DOMAIN_COUNT],
            slots: [VtdSlot {
                in_use: false,
                domain: 0,
                iova: 0,
                len: 0,
                generation: 0,
                read_only: false,
            }; VTD_SLOT_COUNT],
            qi_ring: [QiDesc { lo: 0, hi: 0 }; VTD_QI_RING],
            qi_count: 0,
            faults: [None; VTD_FAULT_RING],
            fault_count: 0,
            origin_sid: 0,
        }
    }

    pub fn set_origin_sid(&mut self, sid: u16) {
        self.origin_sid = sid;
    }

    /// Daftarkan domain second-level milik partisi (first-come-first-served).
    pub fn register_domain(&mut self, partition_id: u32) -> Result<usize, DmaError> {
        for i in 0..self.domains.len() {
            if self.domains[i].in_use {
                if self.domains[i].partition_id == partition_id {
                    return Ok(i); // idempotent
                }
                continue;
            }
            self.domains[i] = empty_domain();
            self.domains[i].in_use = true;
            self.domains[i].partition_id = partition_id;
            self.domains[i].base_iova = VTD_IOVA_BASE + (i as u64) * VTD_DOMAIN_WINDOW;
            return Ok(i);
        }
        Err(DmaError::OutOfSlots)
    }

    fn domain_of(&self, partition_id: u32) -> Option<usize> {
        (0..self.domains.len()).find(|&i| self.domains[i].in_use && self.domains[i].partition_id == partition_id)
    }

    pub fn domain_window(&self, domain: usize) -> Option<(u64, u64)> {
        if domain < self.domains.len() && self.domains[domain].in_use {
            Some((self.domains[domain].base_iova, VTD_DOMAIN_WINDOW))
        } else {
            None
        }
    }

    fn take_slot(&mut self) -> Result<usize, DmaError> {
        for i in 0..self.slots.len() {
            if !self.slots[i].in_use {
                return Ok(i);
            }
        }
        Err(DmaError::OutOfSlots)
    }

    fn push_qi(&mut self, desc: QiDesc) {
        if self.qi_count < VTD_QI_RING {
            self.qi_ring[self.qi_count] = desc;
        }
        self.qi_count = self.qi_count.wrapping_add(1);
    }

    /// Descriptor QI terakhir yang dipush (untuk test/telemetri).
    pub fn qi_pushed(&self) -> usize {
        self.qi_count
    }

    pub fn qi_desc_at(&self, idx: usize) -> QiDesc {
        self.qi_ring[core::cmp::min(idx, VTD_QI_RING - 1)]
    }

    /// Catat fault pada ring model FRCD (dipakai jalur fault runtime & test
    /// injeksi; translate via &self tidak boleh mutasi — pemanggil runtime
    /// wrapper `&mut` yang mencatat, sama seperti hardware mencatat FRCD).
    pub fn record_fault(&mut self, iova: u64, reason: u8, is_write: bool) {
        let r = VtdFaultRecord {
            iova: iova & !0x3F, // FRCD menyimpan address-granular
            reason,
            is_write,
            sid: self.origin_sid,
        };
        if self.fault_count < VTD_FAULT_RING {
            self.faults[self.fault_count] = Some(r);
        }
        self.fault_count = self.fault_count.wrapping_add(1);
    }

    pub fn fault_count(&self) -> usize {
        core::cmp::min(self.fault_count, VTD_FAULT_RING)
    }

    pub fn fault_at(&self, idx: usize) -> Option<VtdFaultRecord> {
        if idx < VTD_FAULT_RING {
            self.faults[idx]
        } else {
            None
        }
    }

    /// Pin RO untuk jalur RX (fill descriptor tidak boleh writable DMA).
    pub fn pin_ro(&mut self, partition_id: u32, phys: u64, len: u64) -> Result<IovaLease, DmaError> {
        self.pin_internal(partition_id, phys, len, false)
    }

    fn pin_internal(
        &mut self,
        partition_id: u32,
        phys: u64,
        len: u64,
        writable: bool,
    ) -> Result<IovaLease, DmaError> {
        // Kontrak lease: 64B bus-aligned. Kedua level: second-level page table
        // hardware menuntut base fisik 4KiB-aligned (granularitas leaf).
        if len == 0 || (phys & 0x3F) != 0 || (len & 0x3F) != 0 {
            return Err(DmaError::Misaligned);
        }
        if (phys & (VTD_PAGE_SIZE - 1)) != 0 {
            return Err(DmaError::Misaligned);
        }
        let d = match self.domain_of(partition_id) {
            Some(d) => d,
            None => return Err(DmaError::PermissionDenied),
        };
        let pages = match len.checked_add(VTD_PAGE_SIZE - 1) {
            Some(x) => x >> VTD_PAGE_SHIFT,
            None => return Err(DmaError::RangeOutOfBounds),
        };
        let span_bytes = match pages.checked_mul(VTD_PAGE_SIZE) {
            Some(s) => s,
            None => return Err(DmaError::RangeOutOfBounds),
        };
        let base_iova = self.domains[d].base_iova;
        let off = self.domains[d].next_off;
        let end = match off.checked_add(span_bytes) {
            Some(e) => e,
            None => return Err(DmaError::RangeOutOfBounds),
        };
        if end > VTD_DOMAIN_WINDOW {
            return Err(DmaError::RangeOutOfBounds);
        }
        let slot = self.take_slot()?;
        let first = (off >> VTD_PAGE_SHIFT) as usize;
        let mut p = phys;
        let mut i = first;
        let last = first + pages as usize;
        while i < last {
            self.domains[d].table[i] = sl_leaf(p, writable);
            p += VTD_PAGE_SIZE;
            i += 1;
        }
        self.domains[d].next_off = end;
        let gen = self.slots[slot].generation;
        self.slots[slot] = VtdSlot {
            in_use: true,
            domain: d,
            iova: base_iova + off,
            len,
            generation: gen,
            read_only: !writable,
        };
        self.push_qi(qi_global_iotlb_invalidate());
        self.push_qi(qi_wait());
        Ok(IovaLease::new(base_iova + off, len, partition_id, slot, gen))
    }

    pub fn validate_lease(&self, lease: &IovaLease) -> Result<(), DmaError> {
        let slot = lease.slot();
        if slot >= VTD_SLOT_COUNT {
            return Err(DmaError::GenerationMismatch);
        }
        let s = self.slots[slot];
        if !s.in_use || s.generation != lease.generation() || s.iova != lease.iova() {
            return Err(DmaError::GenerationMismatch);
        }
        if self.domains[s.domain].partition_id != lease.partition_id() {
            return Err(DmaError::PermissionDenied);
        }
        Ok(())
    }

    /// Telemetri slot: (iova, len, read_only) untuk validasi test/serial.
    pub fn slot_info(&self, idx: usize) -> Option<(u64, u64, bool)> {
        if idx < VTD_SLOT_COUNT && self.slots[idx].in_use {
            Some((self.slots[idx].iova, self.slots[idx].len, self.slots[idx].read_only))
        } else {
            None
        }
    }
}

impl DmaAddressSpace for VtdAddressSpace {
    fn translate_and_check(
        &self,
        partition_id: u32,
        buffer_addr: u64,
        size_bytes: usize,
        is_write: bool,
    ) -> Result<DmaDeviceAddress, DmaAddressFault> {
        // Domain mana yang mengklaim window ini?
        let owner = (0..self.domains.len()).find(|&i| {
            self.domains[i].in_use
                && buffer_addr >= self.domains[i].base_iova
                && (buffer_addr + size_bytes as u64) <= self.domains[i].base_iova + VTD_DOMAIN_WINDOW
        });
        let d = match owner {
            Some(d) => d,
            None => return Err(DmaAddressFault::AddressOutOfRange),
        };
        if self.domains[d].partition_id != partition_id {
            return Err(DmaAddressFault::CrossPartitionViolation);
        }
        match sl_walk(&self.domains[d].table, self.domains[d].base_iova, buffer_addr, size_bytes, is_write) {
            Ok(phys) => Ok(DmaDeviceAddress(phys)),
            Err(VtdWalkFault::NotPresent) => Err(DmaAddressFault::TranslationNotPresent),
            Err(VtdWalkFault::Permission) => Err(DmaAddressFault::ProtectionFaultReadOnly),
            Err(VtdWalkFault::RangeOutOfBounds) => Err(DmaAddressFault::AddressOutOfRange),
        }
    }

    fn unmap_iova(
        &mut self,
        partition_id: u32,
        iova: u64,
        size_bytes: usize,
    ) -> Result<(), DmaAddressFault> {
        // Model minimum: unmapping individual IOVA tidak didukung (konservatif,
        // fail-closed) — gunakan unpin(&lease) agar generation anti-TOCTOU tegak.
        let _ = (partition_id, iova, size_bytes);
        Err(DmaAddressFault::TranslationNotPresent)
    }

    fn pin(&mut self, partition_id: u32, phys: u64, len: u64) -> Result<IovaLease, DmaError> {
        self.pin_internal(partition_id, phys, len, true)
    }

    fn unpin(&mut self, lease: &IovaLease) -> Result<(), DmaError> {
        self.validate_lease(lease)?;
        let slot = lease.slot();
        let d = self.slots[slot].domain;
        let off = lease.iova() - self.domains[d].base_iova;
        let first = (off >> VTD_PAGE_SHIFT) as usize;
        let pages = ((lease.len() + VTD_PAGE_SIZE - 1) >> VTD_PAGE_SHIFT) as usize;
        let mut i = first;
        while i < first + pages && i < VTD_LEAVES_PER_TABLE {
            self.domains[d].table[i] = 0;
            i += 1;
        }
        self.slots[slot].in_use = false;
        self.slots[slot].generation = self.slots[slot].generation.wrapping_add(1);
        self.push_qi(qi_global_iotlb_invalidate());
        self.push_qi(qi_wait());
        Ok(())
    }

    fn iova_alloc(&mut self, partition_id: u32, len: u64) -> Result<u64, DmaError> {
        if len == 0 || (len & 0x3F) != 0 {
            return Err(DmaError::Misaligned);
        }
        let d = match self.domain_of(partition_id) {
            Some(d) => d,
            None => return Err(DmaError::PermissionDenied),
        };
        let rounded = (len + VTD_PAGE_SIZE - 1) & !(VTD_PAGE_SIZE - 1);
        let off = self.domains[d].next_off;
        if off + rounded > VTD_DOMAIN_WINDOW {
            return Err(DmaError::RangeOutOfBounds);
        }
        self.domains[d].next_off = off + rounded;
        Ok(self.domains[d].base_iova + off)
    }

    /// Konservatif: dealloc menaikkan generation domain => SEMUA lease partisi
    /// ini jadi stale (fail-closed; tidak ada re-use diam-diam).
    fn iova_dealloc(&mut self, partition_id: u32, _iova: u64, _len: u64) -> Result<(), DmaError> {
        let d = match self.domain_of(partition_id) {
            Some(d) => d,
            None => return Err(DmaError::PermissionDenied),
        };
        for s in self.slots.iter_mut() {
            if s.in_use && s.domain == d {
                s.in_use = false;
                s.generation = s.generation.wrapping_add(1);
            }
        }
        self.domains[d].table = [0; VTD_LEAVES_PER_TABLE];
        self.domains[d].next_off = 0;
        self.push_qi(qi_global_iotlb_invalidate());
        self.push_qi(qi_wait());
        Ok(())
    }

    fn validate_pin(&self, lease: &IovaLease) -> Result<(), DmaError> {
        self.validate_lease(lease)
    }
}

// -----------------------------------------------------------------------------
// helpers byte-order
// -----------------------------------------------------------------------------

const fn rd_u32(b: &[u8], o: usize) -> u32 {
    u32::from_le_bytes([b[o], b[o + 1], b[o + 2], b[o + 3]])
}

const fn rd_u16(b: &[u8], o: usize) -> u16 {
    u16::from_le_bytes([b[o], b[o + 1]])
}

const fn rd_u64(b: &[u8], o: usize) -> u64 {
    u64::from_le_bytes([
        b[o], b[o + 1], b[o + 2], b[o + 3], b[o + 4], b[o + 5], b[o + 6], b[o + 7],
    ])
}

// -----------------------------------------------------------------------------
// HOST TESTS (QP-01..) — logika tabel/parsing terverifikasi tanpa hardware
// -----------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;

    /// RSDP rev2 36-byte dengan checksum valid + XSDT pointer.
    fn build_rsdp(xsdt_phys: u64, out: &mut [u8; 36]) {
        out[0..8].copy_from_slice(b"RSD PTR ");
        out[8..14].copy_from_slice(b"ASLQEM"); // oem id 6
        out[14] = 0; // reserved
        out[15] = 2; // revision
        out[16..20].copy_from_slice(&0u32.to_le_bytes()); // rsdt=0
        out[20..24].copy_from_slice(&36u32.to_le_bytes()); // length
        out[24..32].copy_from_slice(&xsdt_phys.to_le_bytes());
        out[32] = 0; // extended checksum placeholder
        out[33..36].copy_from_slice(&[0, 0, 0]);
        // fix checksum utama (20 byte pertama)
        out[9] = 0;
        let mut sum: u8 = 0;
        for i in 0..20 {
            sum = sum.wrapping_add(out[i]);
        }
        out[9] = 0u8.wrapping_sub(sum);
        let mut s2: u8 = 0;
        for i in 0..36 {
            s2 = s2.wrapping_add(out[i]);
        }
        out[32] = 0u8.wrapping_sub(s2);
    }

    #[test]
    fn qp01_rsdp_scan_and_checksum() {
        let mut rsdp = [0u8; 36];
        build_rsdp(0x1f_0000, &mut rsdp);
        let mut window = [0u8; 128 * 1024];
        window[0x2000..0x2000 + 36].copy_from_slice(&rsdp);
        let found = find_rsdp(&window, 0xE_0000).expect("rsdp");
        assert_eq!(found, 0xE_0000 + 0x2000);
        // rusak checksum -> harus ditolak
        window[0x2000 + 9] ^= 0xFF;
        assert!(find_rsdp(&window, 0xE_0000).is_none());
    }

    #[test]
    fn qp02_root_table_walk_finds_dmar() {
        // XSDT berisi 2 entri: APIC lalu DMAR
        let mut xsdt = [0u8; 64];
        xsdt[0..4].copy_from_slice(b"XSDT");
        xsdt[4..8].copy_from_slice(&(36u32 + 16).to_le_bytes());
        xsdt[36..44].copy_from_slice(&0x10u64.to_le_bytes());
        xsdt[44..52].copy_from_slice(&0x20u64.to_le_bytes());
        let sigs: [(u64, [u8; 8]); 2] = [
            (0x10, *b"APIC\0\0\0\0"),
            (0x20, *b"DMAR\0\0\0\0"),
        ];
        let read8 = |phys: u64| -> [u8; 8] {
            for (a, s) in sigs.iter() {
                if *a == phys {
                    return *s;
                }
            }
            [0u8; 8]
        };
        let dmar_phys = find_acpi_table_with(&xsdt, b"DMAR", read8).expect("dmar ptr");
        assert_eq!(dmar_phys, 0x20);
        assert!(find_acpi_table_with(&xsdt, b"IVRS", read8).is_none());
    }

    fn build_dmar(base: u64) -> [u8; 64] {
        let mut t = [0u8; 64];
        t[0..4].copy_from_slice(b"DMAR");
        t[4..8].copy_from_slice(&64u32.to_le_bytes());
        t[8] = 1; // revision
        // header 48 byte -> subtable DRHD di offset 48:
        // type u8@0, length u8@1, spec ver u16@2, flags u8@4, reserved u8@5,
        // segment u16@6, register base u64@8
        t[48] = 0; // type DRHD
        t[49] = 16; // length
        t[52..54].copy_from_slice(&1u16.to_le_bytes()); // spec version
        t[54] = 1; // DRHD flags include remapping supported
        t[54..56].copy_from_slice(&0u16.to_le_bytes()); // segment 0
        t[56..64].copy_from_slice(&base.to_le_bytes());
        t
    }

    /// Layout QEMU build_dmar_q35: Type 2 byte (0), Length 2 byte di +2, flags+
    /// reserved di +4/+5, segment +6, base +8; total table = 48+72.
    fn build_dmar_qemu_style(base: u64) -> [u8; 120] {
        let mut t = [0u8; 120];
        t[0..4].copy_from_slice(b"DMAR");
        t[4..8].copy_from_slice(&120u32.to_le_bytes());
        t[8] = 1; // revision
        t[36] = 0x2F; // host address width
        // subtable DRHD @48
        t[48..50].copy_from_slice(&0u16.to_le_bytes()); // type 2-byte
        t[50..52].copy_from_slice(&72u16.to_le_bytes()); // length 2-byte
        t[54..56].copy_from_slice(&0u16.to_le_bytes()); // segment 0
        t[56..64].copy_from_slice(&base.to_le_bytes());
        // scope entries (diabaikan parser)
        t
    }

    #[test]
    fn qp03_dmar_drhd_parse() {
        let t = build_dmar(0xFEDD_0000);
        let d = parse_dmar(&t).expect("drhd legacy-u8-length");
        assert_eq!(d.register_base, 0xFEDD_0000);
        assert_eq!(d.drhd_count, 1);
        let mut bad = t;
        bad[0] = b'X';
        assert!(parse_dmar(&bad).is_none());
        // varian layout nyata QEMU q35 (bukti byte dump runtime 2026-10-20)
        let q = build_dmar_qemu_style(0xFED9_0000);
        let d2 = parse_dmar(&q).expect("drhd qemu-u16-length");
        assert_eq!(d2.register_base, 0xFED9_0000);
        assert_eq!(d2.drhd_count, 1);
    }

    #[test]
    fn qp04_root_context_encoders() {
        let mut root = [0u8; 4096];
        write_root_entry(&mut root, 0, 0x1234_5000 | 0xABF); // unaligned低位 dibersihkan
        let lo = rd_u64(&root, 0);
        assert_eq!(lo & !0xFFF, 0x1234_5000);
        assert_eq!(lo & 1, 1);
        let mut ctx = [0u8; 4096];
        fill_contexts_passthrough(&mut ctx);
        let lo_pt = rd_u64(&ctx, 31 * 16); // devfn 31
        let (present, tt, _) = context_entry_fields(lo_pt);
        assert!(present && tt == CTX_TT_PASS_THROUGH);
        write_context_entry(&mut ctx, 31, 0x2000_0000, CTX_TT_SECOND_LEVEL);
        let (p2, tt2, sl) = context_entry_fields(rd_u64(&ctx, 31 * 16));
        assert!(p2 && tt2 == CTX_TT_SECOND_LEVEL && sl == 0x2000_0000);
    }

    #[test]
    fn qp05_sl_walk_semantics() {
        let mut table = [0u64; VTD_LEAVES_PER_TABLE];
        table[0] = sl_leaf(0x4000_0000, true);
        table[1] = sl_leaf(0x4000_1000, false);
        let base = VTD_IOVA_BASE;
        assert_eq!(sl_walk(&table, base, base, 4096, true), Ok(0x4000_0000));
        assert_eq!(sl_walk(&table, base, base + 0x1040, 16, false), Ok(0x4000_1040));
        assert_eq!(sl_walk(&table, base, base + 0x1000, 16, true), Err(VtdWalkFault::Permission));
        assert_eq!(sl_walk(&table, base, base + 0x2000, 16, false), Err(VtdWalkFault::NotPresent));
        assert_eq!(sl_walk(&table, base, base - 0x10, 16, false), Err(VtdWalkFault::RangeOutOfBounds));
        assert_eq!(sl_walk(&table, base, base, 0, false), Err(VtdWalkFault::RangeOutOfBounds));
    }

    #[test]
    fn qp06_backend_pin_translate_fault() {
        let mut sp = VtdAddressSpace::new();
        sp.register_domain(7).unwrap();
        // pin belum ada -> permission denied utk partisi tak terdaftar
        assert_eq!(sp.pin(9, 0x4000_0000, 4096), Err(DmaError::PermissionDenied));
        let lease = sp.pin(7, 0x4000_0000, 0x800).expect("pin");
        assert_eq!(lease.iova(), VTD_IOVA_BASE);
        assert_eq!(sp.validate_lease(&lease), Ok(()));
        let qi_after_pin = sp.qi_pushed();
        assert!(qi_after_pin >= 2); // invalidate + wait
        // translate walk SL nyata
        let got = sp
            .translate_and_check(7, lease.iova(), 0x800, true)
            .expect("translate");
        assert_eq!(got.0, 0x4000_0000);
        // partisi asing -> violation TANPA menyentuh fault ring domain (cross claim)
        assert_eq!(
            sp.translate_and_check(5, lease.iova(), 16, false),
            Err(DmaAddressFault::CrossPartitionViolation)
        );
        // IOVA unmapped dalam window milik partisi -> NotPresent + record fault
        sp.record_fault(0x1000_1000, FR_REASON_SL_NOT_PRESENT, false);
        assert_eq!(sp.fault_count(), 1);
        let f = sp.fault_at(0).unwrap();
        assert_eq!(f.reason, FR_REASON_SL_NOT_PRESENT);
        let enc = encode_fault_record(&f);
        let dec = decode_fault_record(&enc);
        assert_eq!(dec, f);
        // unpin -> stale selamanya
        assert_eq!(sp.unpin(&lease), Ok(()));
        assert_eq!(sp.validate_lease(&lease), Err(DmaError::GenerationMismatch));
        assert_eq!(sp.unpin(&lease), Err(DmaError::GenerationMismatch));
    }

    #[test]
    fn qp07_backend_alignment_bounds_slots() {
        let mut sp = VtdAddressSpace::new();
        sp.register_domain(1).unwrap();
        assert_eq!(sp.pin(1, 0x4000_0010, 4096), Err(DmaError::Misaligned));
        // 64B-aligned + 4KiB page-aligned base => diterima, konsumsi 1 leaf
        assert!(sp.pin(1, 0x4000_1000, 64).is_ok());
        // base fisik harus 4KiB-aligned (granularitas leaf SL hardware)
        assert_eq!(sp.pin(1, 0x4000_2040, 4096), Err(DmaError::Misaligned));
        assert_eq!(sp.pin(1, 0x4000_2000, 0), Err(DmaError::Misaligned));
        // window 2MiB: pin melewati batas -> RangeOutOfBounds
        let mut big = VtdAddressSpace::new();
        big.register_domain(2).unwrap();
        assert_eq!(
            big.pin(2, 0x5000_0000, VTD_DOMAIN_WINDOW + 0x1000),
            Err(DmaError::RangeOutOfBounds)
        );
    }

    #[test]
    fn qp08_ro_lease_and_dealloc_fails_closed() {
        let mut sp = VtdAddressSpace::new();
        sp.register_domain(3).unwrap();
        let rx = sp.pin_ro(3, 0x4100_0000, 0x1000).unwrap();
        assert_eq!(
            sp.translate_and_check(3, rx.iova(), 0x1000, true),
            Err(DmaAddressFault::ProtectionFaultReadOnly)
        );
        assert_eq!(sp.translate_and_check(3, rx.iova(), 0x1000, false).is_ok(), true);
        assert_eq!(sp.iova_dealloc(3, rx.iova(), 0x1000), Ok(()));
        assert_eq!(sp.validate_lease(&rx), Err(DmaError::GenerationMismatch));
        // setelah dealloc, translate wilayah tadi = NotPresent
        assert_eq!(
            sp.translate_and_check(3, rx.iova(), 16, false),
            Err(DmaAddressFault::TranslationNotPresent)
        );
        assert_eq!(sp.iova_dealloc(88, 0, 0), Err(DmaError::PermissionDenied));
    }

    #[test]
    fn qp09_iova_alloc_bump_and_repin() {
        let mut sp = VtdAddressSpace::new();
        sp.register_domain(4).unwrap();
        let a = sp.iova_alloc(4, 0x4000).unwrap();
        let b = sp.iova_alloc(4, 0x100).unwrap();
        assert_eq!(a, VTD_IOVA_BASE);
        assert_eq!(b, VTD_IOVA_BASE + 0x4000);
        assert_eq!(sp.iova_alloc(4, 0), Err(DmaError::Misaligned));
        assert_eq!(
            sp.iova_alloc(4, VTD_DOMAIN_WINDOW),
            Err(DmaError::RangeOutOfBounds)
        );
    }

    #[test]
    fn qp10_unmapped_translate_walk_returns_notpresent() {
        // jalur translate publik di domain kosong => NotPresent + tercatat via record_fault
        let mut sp = VtdAddressSpace::new();
        sp.register_domain(6).unwrap();
        sp.set_origin_sid(0x0800);
        assert_eq!(
            sp.translate_and_check(6, VTD_IOVA_BASE + 0x100, 64, false),
            Err(DmaAddressFault::TranslationNotPresent)
        );
        // dan di luar SEMUA window -> AddressOutOfRange
        assert_eq!(
            sp.translate_and_check(6, 0x9000_0000, 64, false),
            Err(DmaAddressFault::AddressOutOfRange)
        );
    }

    #[test]
    fn qp11_sl_walk_hw_three_level_mirror() {
        // rantai identik runtime QM-01: PDP[0]->PD[128]->PT[0]->stash
        let base = VTD_IOVA_BASE; // 0x1000_0000: l3=0, l2=128, l1=0
        let mut pdp = [0u64; VTD_LEAVES_PER_TABLE];
        let mut pd = [0u64; VTD_LEAVES_PER_TABLE];
        let mut pt = [0u64; VTD_LEAVES_PER_TABLE];
        let pd_fake = 0x2000_0000u64;
        let pt_fake = 0x2010_0000u64;
        let stash = 0x4000_0000u64;
        pdp[((base >> 30) & 0x1FF) as usize] = sspt_nonleaf(pd_fake);
        pd[((base >> 21) & 0x1FF) as usize] = sspt_nonleaf(pt_fake);
        pt[0] = sl_leaf(stash, true);
        assert_eq!(sl_walk_hw(&pdp, &pd, &pt, base, 4096, true), Ok(stash));
        assert_eq!(sl_walk_hw(&pdp, &pd, &pt, base + 0x40, 16, true), Ok(stash + 0x40));
        // RO leaf: read lolos, write ditolak Permission
        pt[1] = sl_leaf(stash + 0x1000, false);
        assert_eq!(sl_walk_hw(&pdp, &pd, &pt, base + 0x1000, 16, false), Ok(stash + 0x1000));
        assert_eq!(sl_walk_hw(&pdp, &pd, &pt, base + 0x1000, 16, true), Err(VtdWalkFault::Permission));
        // leaf belum dipetakan (PT offset 128) -> NotPresent
        assert_eq!(
            sl_walk_hw(&pdp, &pd, &pt, base + 0x8_0000, 64, false),
            Err(VtdWalkFault::NotPresent)
        );
        // PD slot kosong (window tetangga) -> NotPresent di level 2
        assert_eq!(
            sl_walk_hw(&pdp, &pd, &pt, base + 0x20_0000, 64, false),
            Err(VtdWalkFault::NotPresent)
        );
        // 2MiB large leaf di L2: PS=1, bits 20:11 nol
        let big = sspt_leaf_2m(0x8000_0000, true);
        assert_eq!(big & 0x1F_F000, 0);
        assert_eq!(big & SSPT_PS, SSPT_PS);
        pd[((base >> 21) & 0x1FF) as usize] = big;
        assert_eq!(sl_walk_hw(&pdp, &pd, &pt, base + 0x1234, 8, true), Ok(0x8000_1234));
        // rentang melewati dua halaman leaf
        pt[2] = sl_leaf(stash + 0x2000, true);
        // dengan large leaf di L2, seluruh offset dalam 2MiB terpetakan
        // meski PT tidak dipakai sama sekali:
        assert_eq!(sl_walk_hw(&pdp, &pd, &pt, base + 0x1FF0, 64, true), Ok(0x8000_1FF0));
        pd[((base >> 21) & 0x1FF) as usize] = sspt_nonleaf(pt_fake);
        pt[0] = sl_leaf(stash, true);
        pt[1] = 0;
        assert_eq!(sl_walk_hw(&pdp, &pd, &pt, base, 0x1000 * 2, true), Err(VtdWalkFault::NotPresent));
        assert_eq!(sl_walk_hw(&pdp, &pd, &pt, base, 0, true), Err(VtdWalkFault::RangeOutOfBounds));
        // tipe deskriptor QI global: opcode 2 + GLOBAL bit 4 (persis yang
        // diterima vtd_process_iotlb_desc QEMU)
        let d = qi_global_iotlb_invalidate();
        assert_eq!(d.lo & 0xF, QI_OP_IOTLB_GLOBAL_INV);
        assert_eq!(d.lo & 0x30, QI_INV_IOTLB_GLOBAL);
        assert_eq!(d.hi & 0xF80, 0); // reserved hi nol -> lolos rsvd check
    }
}
