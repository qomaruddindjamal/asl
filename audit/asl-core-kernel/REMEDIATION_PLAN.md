# REMEDIATION PLAN: ASL CORE KERNEL (REVISED SIKLUS 60-HARI)
**Date:** 2026-10-13  
**Auditor Lead:** ANTI-GRAVITY (Independent Lead Auditor & R&D Systems Engineer)  
**Governing Standard:** DO-178C-aligned, ladder DAL-C->A (lihat ADR-002) / ARINC 653 Part 1  

---

## 1. REKAPITULASI REGISTER TEMUAN TERKINI

* **Total CLOSED (12):** `SEC-01`, `SEC-02`, `SEC-03`, `SEC-04`, `SEC-05`, `AG-01`, `AG-02`, `AG-03`, `AG-05`, `AG-06`, `ALIGN-01`, `AG-04`.
* **MITIGATED (CODE) — hardware verification pending (1):** `DMA-ADDR-01` (RND_08 FASE-3, 2026-10-20).
* **PARTIAL-MITIGATED (CODE) (1):** `DMA-CACHE-01` (jalur bounce RND_08; barrier HW riil pending).
* **Total OPEN / IN-PROGRESS (2):** `ATOMIC-SPARC-01`, `VT-D-DRV-01` (PLANNED).

---

## 2. MATRIKS PRIORITAS DAN TINDAKAN PER-TEMUAN

| ID | Severity | Status | Lokasi / Bukti | Remediasi | Acceptance Criteria |
| :--- | :---: | :---: | :--- | :--- | :--- |
| **SEC-01** | KRITIS/TINGGI | CLOSED | `asl-security-vault/src/lib.rs:40` | `checked_add` boundary | 5 unit tests pass |
| **SEC-02** | SEDANG | CLOSED | `asl-bootloader/src/lib.rs:141` | 52-bit max limit + `checked_add` | 8 unit tests pass |
| **SEC-03** | SEDANG | CLOSED | `asl-afdx-ipc`, `asl-dsp-rpc`, `asl-pmd` | SPSC atomic Acquire/Release | 10 unit tests pass (DILARANG REGRESI) |
| **SEC-04** | RENDAH | CLOSED | `asl-ont-gpon/src/lib.rs:66` | Mock documented + explicit status | 2 unit tests pass |
| **SEC-05** | RENDAH | CLOSED | `asl-mips-fastpath/src/lib.rs:74` | `u32::from_be` network order | 2 unit tests pass |
| **AG-02** | RENDAH | CLOSED | `README.md §2` | Sinkronisasi prefix `asl-kernel/target` | `Test-Path` return True pada semua artefak |
| **AG-03** | SEDANG | CLOSED | VM Telemetry / Build Profil | Re-capture telemetri M1..M6 dengan build `--release` ke `asl_vm_serial_release.log` | M1..M6 lengkap di log release + idle ≥ 50s |
| **AG-05** | RENDAH | CLOSED | Governance & Traceability | Pemetaan eksplisit nama fungsi unit test & SHA256 di TEST_RESULTS.md | 100% test function names documented |
| **ALIGN-01** | SEDANG | CLOSED | `asl-pmd/src/lib.rs` | `#[repr(C, align(64))]` + `init_pool` check + static assert | 4 unit test ALIGN-01 lulus 100% |
| **AG-06** | RENDAH | CLOSED | `asl-kernel-bin/build.rs` | Banner M1 memuat penanda `profile=release` via option_env! | Serial output memuat penanda + 1 unit test pass |
| **AG-01** | SEDANG | CLOSED | `asl-cctv-dma/src/lib.rs` & `asl-hal/src/mmio.rs` | Refactor `is_locked_by_vpu` menggunakan `MmioRegister<u32>` volatile read | 100% compliant, 0 refactor needed, 2 unit tests pass |
| **AG-04** | SEDANG | CLOSED | `CANONICAL_MASTER.md` & `RND_05` | Kebijakan P-ISR-1..4 diratifikasi RATIFIED v1.0 final (W-10 SPARC 3 NOP delay, W-11 per-ISR WCET budgets) + 7-row inventory | Seluruh 7 situs lock 100% COMPLIANT |
| **DMA-CACHE-01** | TINGGI | PARTIAL-MITIGATED (CODE, 2026-10-20 FASE-3) | `asl-hal/src/cache.rs`, `asl-hal/src/dma_space.rs` (`BounceAddressSpace::stage`) | Trait `CacheMaintenanceOps` + buddy pool 64B; RND_08: staging bounce koheren — hardware tidak pernah DMA ke buffer cache-stale caller; verifikasi zero-stale read via Gem5 / Zynq-7000 masih terbuka | Zero stale DMA read HW (QEMU TCG tidak sah; butuh Gem5 / hardware fisik) |
| **DMA-ADDR-01** | TINGGI | MITIGATED (CODE) / VERIFIED-ON-HARDWARE PENDING (2026-10-20 FASE-3) | `asl-hal/src/dma_space.rs`, `asl-pmd/src/lib.rs`, `asl-cctv-dma/src/lib.rs` | RND_08: kontrak `IovaLease` tak-termalsukan; `transmit_burst`/`setup_channel_buffer` hanya terima lease (breaking, ditegakkan kompilator); backend bounce whitelist + generation anti-TOCTOU; +11 host test FASE-3 (8 hal TM-01..08, 1 pmd, 2 cctv) lulus | Host logic = VERIFIED (160/160 workspace); open sampai fault-path hardware (VT-D-DRV-01) |
| **ATOMIC-SPARC-01** | SEDANG | OPEN | `asl-space-tmr/src/lib.rs` | Tahap 1 & 2 selesai: AtomicU32 downscale + AtomicU64Split + UniversalAtomicU64. Pengujian nightly sparc64 target. | Pengukuran empiris via TSIM3/GRMON |
| **VT-D-DRV-01** | SEDANG | PLANNED | Hardware IOMMU Engine | Pemrograman register root/context table VT-d Intel q35 & evaluasi hardware fault path | Diverifikasi pada QEMU q35 / target hardware fisik |

---

## 3. RENCANA EKSEKUSI SIKLUS 90-HARI
1. Eksekusi pengujian simulasi Gem5 untuk memvalidasi zero-stale read pada `CacheMaintenanceOps` (penutupan penuh `DMA-CACHE-01`).
2. Implementasi pembatasan `IdentityDmaSpace` compile-time error dan penegakan IOMMU translation space (`DMA-ADDR-01`).
3. Pengukuran siklus WCET pada TSIM3 / board hardware GR712RC untuk menutup `ATOMIC-SPARC-01`.

---

## 4. PEMETAAN 10 RISIKO DEEP AUDIT (DA-01..DA-10) — ditambah 2026-10-20
Sumber: `Analysis/reports/DEEP_AUDIT_REPORT.md` §6. Owner agen eksekusi: QODER (fase remediasi 1–3); jalur hardware tetap owner lead auditor.

| ID | Severity | Lokasi / File Terdampak | Mitigasi | Status | Owner |
| :--- | :---: | :--- | :--- | :---: | :--- |
| **DA-01** | RENDAH | `asl-kernel/.cargo/config.toml` | Hapus default `[build] target`; host test jalan tanpa flag; konfigurasi terdokumentasi di `DEVELOPER_ONBOARDING.md` (Workspace Cargo Configuration Map) | **CLOSED (FIX-1, 2026-10-20; 128/128 PASS)** | QODER |
| **DA-02** | KRITIS | `asl-pmd/src/lib.rs` (`transmit_burst`), `asl-hal/src/dma_space.rs` | Integrasi `DmaAddressSpace` + kontrak `IovaLease` (RND_08), backend bounce + VT-d | **CODE-COMPLETE (FASE-3, 2026-10-20; 160/160 host test; commit [PHASE-3]) — hardware VT-d pending (DA-05/VT-D-DRV-01)** | QODER / lead auditor (hardware) |
| **DA-03** | TINGGI | `asl-kernel-core/src/lib.rs:610` (`GLOBAL_SECONDARY_ENGINE`) | State machine atomik AtomicU8 + carve-out region engine di launcher; aman dari konteks fault/panic | **DONE (FASE-2, commit 4f55c87; runtime-armed dibuktikan via QEMU serial)** | QODER |
| **DA-04** | SEDANG | `asl-kernel-core/src/lib.rs` (panic_handler) | Halt aman + jalur reset/watchdog terdokumentasi; output diagnostik COM1 sebelum berhenti | **IN-PROGRESS (FASE-2 scope selesai: halt aman + diagnostik + DR FSM; hardware reset line menunggu BSP)** | QODER |
| **DA-05** | SEDANG | seluruh situs `read_volatile` MMIO (`asl-hal/mmio.rs`, driver) | Audit sequencing + `core::sync::atomic::fence` / serialization di jalur kritis | **PLANNED (FASE-3 follow-up)** | Lead auditor |
| **DA-06** | TINGGI | `asl-kernel-core/src/lib.rs:731` (mapping HHDM FB) | Validasi range `base_physical_addr + size` terhadap memmap sebelum mapping; gagal → error serial + halt, tidak menulis FB | **DONE (FASE-2, commit 4f55c87; `validate_framebuffer_memmap` + 4 host tests, kebijakan B1)** | QODER |
| **DA-07** | TINGGI | `asl-kernel-bin/src/main.rs` (stack 512 KB), scheduler task stack | Guard page non-present di bawah stack; double-fault IST1 mencetak `STACK OVERFLOW` via COM1 lalu halt aman | **DONE (FASE-2, commit 4f55c87; guard page ditegakkan via `enforce_stack_guard_page`, bukti serial QEMU; injeksi overflow runtime end-to-end RUNTIME-PENDING)** | QODER |
| **DA-08** | RENDAH | `asl_avionics_kernel_blueprint.md` §3, `asl_kernel_audit_roadmap.md` §5 | ADR-001 Rust-primary + rebase kedua dokumen | **CLOSED (FASE-1, 2026-10-20)** | QODER |
| **DA-09** | SEDANG | `asl-driver-pci/src/lib.rs` | Whitelist vendor/device DAL-A + penolakan device tak dikenal sebelum attach | **PLANNED (FASE-3 follow-up; butuh keputusan policy DAL-A)** | Lead auditor |
| **DA-10** | TINGGI | `asl-kernel-core-secondary`, panic/fault notifier | Selain atomisasi DA-03: kunci per-core / single-writer protocol untuk skenario AMP/SMP saat AP core aktif | **PLANNED (FASE-3 follow-up; butuh BSP multi-core)** | Lead auditor |

---

## 5. HARDWARE VERIFICATION GATE (register pekerjaan tertunda, PHASE-4 2026-10-20 — docs-only)
Pecahan terbuka yang memblokir kenaikan status dari CODE-COMPLETE/MITIGATED-CODE ke VERIFIED-ON-HARDWARE. Prioritas P1 = prasyarat anak tangga DAL-C (ADR-002 §2a-d).

| ID | Prioritas | Exit criteria (harus terbukti) | Bukti yang dibutuhkan | Prasyarat hardware/VM | Owner |
| :--- | :---: | :--- | :--- | :--- | :--- |
| **HV-1 / QM-01** (VT-D-DRV-01, DMA-ADDR-01) | **P1** | Domain VT-d terprogram; DMA pada IOVA tak terpetakan menghasilkan fault tercatat + transfer DITOLAK; smoke lulus berulang deterministik | Log serial QEMU q35 + DMAR/IRTL snapshot + test host pemrograman root/context table | **QEMU q35 with `-device intel-iommu` cukup untuk smoke awal** (tanpa hardware fisik); hardware riil hanya untuk timing | QODER (kode) / lead auditor (sign-off) |
| **HV-2 / STACK-OVERFLOW runtime** (DA-07, fase-2 RUNTIME-PENDING) | **P1** | Injeksi overflow end-to-end di VM: guard page non-present terdeteksi, `STACK OVERFLOW` tercetak di COM1, DR FSM mencatat reason, halt aman TERAMATI (bukan hanya kode) | Serial log ISO + screenshot/rekaman + entry fault; test latch sudah host-verified | **QEMU/VirtualBox VM cold-boot — bisa tanpa hardware fisik** | QODER |
| **HV-3 / Cache-barrier HW riil** (DMA-CACHE-01 penuh) | **P2** | Zero stale DMA read pada target non-koheren riil; `dma_cache_clean_inval` dibuktikan on-silicon | Gem5 (model non-koheren) atau Gem5/Zynq-7000; log before/after checksum | Gem5 boleh tanpa fisik; Zynq/ARM SoC untuk konfirmasi akhir; QEMU TCG TIDAK SAH untuk kriteria ini | Lead auditor |
| **HV-4 / ATOMIC-SPARC-01 WCET** | **P3** | Waktu siklus CAS emulation terukur di bawah anggaran TSIM3/GRMON3 | Output TSIM3/GRMON3 pada sparckit/GR712RC | TSIM3 simulator (tanpa fisik) + board GR712RC (opsional) | Lead auditor |
| **HV-5 / DA-04 jalur reset HW** (SEDANG) | P2* | Panic handler menggerakkan garis reset/watchdog fisis dan sistem pulih teramati | Log + osiloskop/telemetri board | Board avionik riil (tidak dapat direplikasi QEMU) | Lead auditor |
| **HV-6 / DA-05 MMIO fencing** (SEDANG) | P2* | Audit sequencing + fence di semua situs `read_volatile` lolos review + race test | Review record + host stress test | Sebagian **dapat di-host-kan**; konfirmasi ordering di hardware riil | Lead auditor |
| **HV-7 / DA-09 PCI device whitelist** (SEDANG) | P3* | Daftar vendor/device yang diizinkan + penolakan device tak dikenal teruji | Host unit test + log QEMU dengan device asing | **QEMU dapat dipakai untuk uji penolakan** | QODER (usulan) / lead auditor (keputusan policy) |
| **HV-8 / DA-10 kunci per-core AMP/SMP** (TINGGI) | P2* | Protokol single-writer per-core terbukti di 2+ core aktif | BSP multi-core + stress harness; sebagian **bisa di-host** dengan model | Hardware/AP boot riil untuk klaim penuh | Lead auditor |

Catatan prioritas: `Pn*` = prioritas mengikuti severity temuan asalnya di register §2 (DA-04 SEDANG→P2, DA-05 SEDANG→P2, DA-09 SEDANG→P3 (under-prioritized, keputusan lead auditor), DA-10 TINGGI→P2); naikkan hanya bila memblokir exit criteria DAL-C di ADR-002. Aturan PHASE-4 4D.2 terpenuhi: HV-1, HV-2, HV-7 (dan sebagian HV-6/HV-8) punya jalur verifikasi QEMU/VirtualBox tanpa hardware fisik; HV-3/HV-4 punya jalur simulator (Gem5/TSIM3); hanya HV-5 dan konfirmasi akhir HV-3/HV-4/HV-8 mutlak butuh board fisis.
