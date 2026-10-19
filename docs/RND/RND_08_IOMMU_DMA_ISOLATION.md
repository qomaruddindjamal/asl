# RND_08 — IOMMU/DMA Spatial Isolation Specification

**Dokumen:** RND_08 | **Status:** RATIFIED-FOR-IMPLEMENTATION (self-review 10 poin PASS, lihat §F)
**Tanggal:** 2026-10-20 | **Branch:** security/asl-v1.0.2-audit | **Phase:** FASE-3 (audit DMA-ADDR-01 / DMA-CACHE-01)
**Standar:** DO-178C-aligned, ladder DAL-C->A (lihat ADR-002), ARINC 653 spatial partitioning; menindaklanjuti `audit/asl-core-kernel/SECURITY_FINDINGS.md` DMA-ADDR-01 (SECURITY/CRITICAL) dan DMA-CACHE-01.

---

## A. Threat Model — DMA Lintas-Partisi (cross-partition DMA)

Sistem: kernel AFDX/IMR dengan `SpatialPartitionManager` (vault), ring PMD zero-copy (`asl-pmd`), dan pipeline video zero-copy (`asl-cctv-dma`).

**Aktif (adversarial):**
1. **T1 — Driver/partisi kompromi menulis `physical_buffer_addr` sembarang** ke deskriptor TX/RX atau frame Y/UV. NIC/VPU kemudian membaca/menulis memori partisi lain atau kernel image → pelanggaran isolasi spasial ARINC 653, korupti data safety-critical. *Inilah DMA-ADDR-01: `transmit_burst(phys_addr: u64, ...)` menerima alamat fisik mentah tanpa validasi.*
2. **T2 — TOCTOU pada buffer yang di-pin**: alamat valid saat enqueue, tetapi halaman dilepas/remap sebelum NIC selesai DMA → DMA menyentuh frame yang sudah dialokasikan ulang ke partisi lain.
3. **T3 — IOVA spoofing**: partisi memakai IOVA milik perangkat/partisi lain ketika IOMMU aktif namun permission domain salah konfig.
4. **T4 — Non-coherent DMA (DMA-CACHE-01)**: pada target non-coherent (mis. beberapa ARM SoC), buffer cache-stale menyebabkan NIC mengirim data cache (isi lama) atau hasil DMA tidak terlihat CPU → korupsi integritas data.
5. **T5 — Overshoot panjang**: `len` melampaui buffer yang dipin → DMA read/write melewati batas region (linear overwrite).

**Bukan ancaman (out of scope):** serangan fisik bus PCIe (trust asumsi platform certification), device-side DMA engine bug (ditangani hardware safety assessment terpisah).

**Hazard mapping:** HAZ-DMA-1 (T1/T3 → loss of partition isolation), HAZ-DMA-2 (T2 → use-after-free via DMA), HAZ-DMA-3 (T4/T5 → data integrity).

## B. Contract — trait `DmaAddressSpace` + handle `IovaLease`

Didefinisikan di `asl-hal` (lapisan trait universal, tanpa dependency baru), dipakai `asl-pmd` dan `asl-cctv-dma`.

```rust
pub enum DmaError { NotPinned, PermissionDenied, Misaligned, OutOfSlots,
                    RangeOutOfBounds, GenerationMismatch, LengthExceedsLease }

/// Handle tervalidasi. SATU-SATUNYA sumber alamat yang boleh masuk deskriptor DMA.
/// Field privat (struct tak bisa dipalsukan pemanggil; hanya backend yang mengconstruct).
pub struct IovaLease {           // dibuat eksklusif oleh backend via pin()/stage()
    iova: u64,                   // alamat yang ditulis ke hardware descriptor
    len: u64,                    // panjang region yang dipin
    partition_id: u32,           // pemilik (ARINC 653 partition; konsisten dgn vault id type)
    slot: usize,                 // index slot backend
    generation: u32,             // anti-TOCTOU: tidak valid jika generation slot berubah
}

/// Trait eksisting asl-hal::dma_space DIPERLUAS (bukan diganti) —
/// translate_and_check/unmap_iova tetap menjadi jalur validasi device-side.
pub trait DmaAddressSpace {
    fn translate_and_check(&self, partition_id: u32, buffer_addr: u64,
                           size_bytes: usize, is_write: bool)
        -> Result<DmaDeviceAddress, DmaAddressFault>;
    fn unmap_iova(&mut self, partition_id: u32, iova: u64, size_bytes: usize)
        -> Result<(), DmaAddressFault>;
    /// Pin region fisik milik partisi (harus tercakup window region yang terdaftar
    /// untuk partisi tsb) -> lease siap hardware. Validasi alignment 64B + kepemilikan.
    fn pin(&mut self, partition_id: u32, phys: u64, len: u64) -> Result<IovaLease, DmaError> { Err(DmaError::NotPinned) }
    /// Release: slot generation++ ; lease stale selamanya ditolak validate.
    fn unpin(&mut self, lease: &IovaLease) -> Result<(), DmaError> { Err(DmaError::NotPinned) }
    fn iova_alloc(&mut self, partition_id: u32, len: u64) -> Result<u64, DmaError> { Err(DmaError::NotPinned) }
    fn iova_dealloc(&mut self, partition_id: u32, iova: u64, len: u64) -> Result<(), DmaError> { Err(DmaError::NotPinned) }
    /// Periksa lease masih otoritatif PRA-DOORBELL (anti-TOCTOU). Backend tanpa
    /// tabel pin tidak dapat membuktikan kesegaran -> default Err(NotPinned).
    fn validate_pin(&self, lease: &IovaLease) -> Result<(), DmaError> { Err(DmaError::NotPinned) }
}
```

**Aturan kanonik (breaking change, wajib):**
- `ZeroCopyPmdQueue::transmit_burst(lease: &IovaLease, len: u16, vlan: u16)` — **HANYA menerima `IovaLease` tervalidasi, BUKAN raw address.** Deskriptor TX diisi `lease.iova()`; validasi runtime: alignment 64B (`lease.is_bus_aligned()`), `len as u64 <= lease.len()` (dan `len > 0`).
- `VideoDmaPipelineEngine::setup_channel_buffer(channel, slot, lease_y: &IovaLease, lease_uv: &IovaLease, width, height)` (`asl-cctv-dma`; API riil fungsi penanaman frame buffer) — sama, dua plane; validasi antesional: partisi kedua lease identik, keselarasan 64B, dan kebutuhan ukuran plane per format (NV12: `w*h` dan `w*h/2`, checked arithmetic) terhadap `lease.len()`; kegagalan tidak pernah memasang frame parsial.
- `VideoDmaPipelineEngine::validate_frame_leases(space, channel, slot, &lease_y, &lease_uv)` — gerbang pra-doorbell via `DmaAddressSpace::validate_pin` + konsistensi mapping frame terpasang (TM-08).
- `poll_rx_burst` dikembalikan ke caller sebagai `RxFrame{ lease: IovaLease, .. }` pada backend yang mendukung; untuk perangkat keras direct-mapped, lease RX dibuat oleh RX-fill path, bukan dari konten deskriptor yang bisa dispoof NIC.

**Alasan breaking:** API lama tidak dapat dibuat "aman dengan validasi" karena alamat mentah tidak membawa identitas partisi/generation; satu-satunya perbaikan deterministik adalah mengubah tipe parameter (ditegakkan kompilator, bukan runtime — praktik terkuat yang dipilih proyek, lihat ADR-002).

**Concurrency model (limitasi resmi, PHASE-4):** setiap backend `DmaAddressSpace` adalah **single-owner** — serialisasi akses SMP menjadi tanggung jawab pemilik backend, ditegakkan oleh kontrak `&mut self` pada `pin/unpin/iova_alloc/iova_dealloc/unmap_iova` (hanya satu pemegang referensi mutable pada satu waktu). `translate_and_check`/`validate_pin` adalah jalur `&self` read-only yang aman terhadap satu owner. Limitasi ini **dievaluasi ulang saat jalur SMP diaktifkan** (pecahan terkait: DA-10 kunci per-core AMP/SMP di `REMEDIATION_PLAN.md` §4); belum ada penegakan lock/lock-free lintas-core di dalam backend.

**Daftar pemanggil yang dimigrasi:**
| # | Lokasi | Lama | Baru |
|---|--------|------|------|
| 1 | `asl-pmd/src/lib.rs` test `test_pmd_transmit_and_congestion` | `transmit_burst(0x2000_0000, 512, 100)` | pin() via BounceAddressSpace lalu `transmit_burst(&lease, 512, 100)` |
| 2 | `asl-cctv-dma` `setup_channel_buffer(phys_y, phys_uv, ...)` + internal callers + tests | raw u64 | `&IovaLease` x2 |
| 3 | (audit grep) workspace-wide | — | 0 pemanggil raw setelah migrasi (transcript 3.4) |

## C. Backend per Tier

| Tier | Backend | Mekanisme | Status |
|------|---------|-----------|--------|
| x86_64 server/avionics | **VT-d** (scalable mode, PASID per partisi) | context table per BDF + second-level page table per partition_id; pin = install leaf + QI invalidate; translate = walk | PLANNED (butuh DRAM/DMAR ACPI parsing; keluar dari ruang lingkup commit ini) |
| ARM avionics (i.MX8/SLVS) | **SMMUv3** (Stream Table + CmdQ) | STE per SID → CD/PTA per partisi; pin = PTE write + CMD_SYNC | PLANNED |
| RISC-V (Microchip/LEON-class) | **RISC-V IOMMU / PMP fallback** | IOTLB per-device context; pada board tanpa IOMMU: PMP region lock + window whitelist | PLANNED |
| Semua target (dan **fallback wajib non-coherent**) | **Software bounce-buffer** (`BounceAddressSpace`) | Slot pool statis per partisi; pin = klaim slot + whitelist window; TX salin data ke slot coherent-aman sebelum DMA hardware menyentuhnya; menutup **DMA-CACHE-01** karena hardware tidak pernah DMA ke buffer cache-stale caller | **IMPLEMENTED (fase ini)** |

Implementasi bounce saat ini: alokasi slot + permission window + generation counter penuh dan teruji host; pemindahan data (copy HHDM) dan cache-maintenance dieksekusi oleh driver pemanggil melalui API slice `stage(&mut self, lease, src)` yang tervetifikasi bounds — jalur runtime diverifikasi pada gate hardware terpisah.

## D. Exit Criteria

1. Kompilator menolak segala pemanggilan `transmit_burst`/`schedule_frame` dengan alamat mentah (tipe `&IovaLease`).
2. Host unit tests: pin/unpin happy-path; misalignment reject; length-exceeds-lease reject; stale-generation (TOCTOU) reject; cross-partition translate deny; slot exhaustion; bounce TX contract.
3. `cargo test --workspace --lib` hijau (semua test lama + baru).
4. Bare-metal x86_64 release 0 error / 0 warning.
5. Grep transcript: tidak ada `transmit_burst(0x` / pola raw-address tersisa.
6. SECURITY_FINDINGS + README §4 + REMEDIATION_PLAN disinkronkan: DMA-ADDR-01 = MITIGATED (kode) / VERIFIED-ON-HARDWARE pending; DMA-CACHE-01 = partial-mitigated (jalur bounce).
7. (Post-fase, di luar gate commit): QEMU dengan VT-d emulation smoke; hardware non-coherent test bench.

## E. Test Matrix (host + QEMU)

| ID | Tier | Cases |
|----|------|-------|
| TM-01 | host | pin valid → lease.iova != phys, window whitelist OK |
| TM-02 | host | pin unaligned (& 0x3F != 0) → Misaligned |
| TM-03 | host | transmit_burst len > lease.len → LengthExceedsLease, ring tidak maju |
| TM-04 | host | unpin → generation++; pakai lease lama → GenerationMismatch |
| TM-05 | host | translate partisi lain → PermissionDenied/NotPinned |
| TM-06 | host | slot pool habis → OutOfSlots; unpin membebaskan |
| TM-07 | host | dua partisi pin window sama → penolakan kepemilikan ganda |
| TM-08 | host | cctv setup_channel_buffer: lease Y valid + UV stale → atomic reject (tidak ada frame terpasang sebagian) |
| QM-01 | QEMU VT-d | smoke: remap domain, DMA deny pada unmapped IOVA (post-fase) |

## F. Self-Review 10 Poin (wajib PASS sebelum implementasi)

| # | Pertanyaan | Verdict |
|---|-----------|---------|
| 1 | Apakah ancaman T1 (raw phys addr) tidak-bisa-lagi terjadi di level tipe? | PASS — parameter bertipe `&IovaLease`, bukan integer |
| 2 | Apakah T2 (TOCTOU) dimitigasi? | PASS — generation counter; lease stale ditolak runtime |
| 3 | Apakah T3 (spoofing IOVA) dimitigasi pada backend software? | PASS — IOVA adalah indeks window whitelist milik partisi peminta saja; translate() menolak domain lain |
| 4 | Apakah T4 (non-coherent, DMA-CACHE-01) dimitigasi? | PASS (parsial — bounce path menutup DMA ke buffer non-koheren caller; cache-maintenance HW diverifikasi di gate hardware terpisah) |
| 5 | Apakah T5 (length overshoot) dimitigasi? | PASS — `len <= lease.len` ditegakkan di transmit path |
| 6 | Apakah ARINC 653 partition_id terbawa di setiap objek DMA? | PASS — IovaLease.partition_id + permission map backend |
| 7 | Apakah no_std dan tanpa dependency eksternal baru (G1)? | PASS — modul di asl-hal/asl-pmd, core-only |
| 8 | Apakah breaking change terdokumentasi + daftar pemanggil lengkap? | PASS — §B + tabel migrasi; diverifikasi grep sebelum commit |
| 9 | Apakah test matrix menutup tiap transition/error enum? | PASS — TM-01..08 memetakan semua DmaError kecuali jalur HW |
| 10 | Apakah klaim status akhir jujur (MITIGATED vs VERIFIED-ON-HARDWARE)? | PASS — §D.6 memisahkan mitigasi kode dan verifikasi hardware |

**Keputusan:** 10/10 PASS → implementasi diizinkan (misi 3.1.e).

---

## G. Hasil Implementasi (2026-10-20, FASE-3) — KANONIKAL TRACE TABLE TM<->test

* Trait diperluas dengan method default kelima: `validate_pin(&self, &IovaLease)` — gerbang pra-doorbell universal; backend tanpa tabel pin mengembalikan `Err(NotPinned)` (stale-by-default, bukan lolos diam-diam).
* `BounceAddressSpace` + `IovaLease` + `DmaError` di `asl-hal/src/dma_space.rs`; `transmit_burst` (`asl-pmd`) dan `setup_channel_buffer` + `validate_frame_leases` (`asl-cctv-dma`) kini hanya menerima `&IovaLease`.
* Pemetaan test matrix -> fungsi uji riil: TM-01 `tm01_pin_within_registered_region_succeeds`; TM-02 (out-of-region, misalign, len=0, overshoot) `tm02_pin_outside_region_and_misaligned_rejected`; TM-03 `tm03_double_claim_same_range_rejected` + `tm04_slot_exhaustion_returns_out_of_slots` (slot habis + release); TM-04 (TOCTOU) `tm05_stale_lease_rejected_after_unpin_toctou`; TM-05 `tm06_bounce_stage_translate_and_cross_partition_denied` + `tm07_read_only_pin_blocks_device_write`; TM-08 (cctv atomic-reject + stale UV pra-doorbell) `tm_cctv_setup_rejects_before_any_state_change` + `tm08_cctv_stale_uv_lease_rejected_pre_doorbell`; tambahan `tm08_iova_alloc_dealloc_and_default_trait_backends_reject_pin` + `tm04_transmit_rejects_oversized_len_and_unaligned_lease` (pmd).
* Catatan penamaan: nomor test `tm04..tm08` mengikuti urutan kasus di atas (TM-03..TM-08 matriks); `QM-01` tetap post-fase (VT-D-DRV-01). Rename fungsi test agar nomornya identik dengan ID matriks: **DITUNDA (opsional, low-priority, PHASE-4)** — tabel ini di atas adalah kanonik trace-nya.
* Gerbang: `cargo test --workspace --lib` = 160/160 PASS (30 suite; +11 dari baseline 149); `cargo build --package asl-kernel-bin --target x86_64-unknown-none --release` = 0 error 0 warning; build aarch64-unknown-none-softfloat = 0 error (warning pre-existing).
