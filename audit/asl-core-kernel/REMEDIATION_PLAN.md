# REMEDIATION PLAN: ASL CORE KERNEL (REVISED SIKLUS 60-HARI)
**Date:** 2026-10-13  
**Auditor Lead:** ANTI-GRAVITY (Independent Lead Auditor & R&D Systems Engineer)  
**Governing Standard:** DO-178C DAL-A / ARINC 653 Part 1  

---

## 1. REKAPITULASI REGISTER TEMUAN TERKINI

* **Total CLOSED (12):** `SEC-01`, `SEC-02`, `SEC-03`, `SEC-04`, `SEC-05`, `AG-01`, `AG-02`, `AG-03`, `AG-05`, `AG-06`, `ALIGN-01`, `AG-04`.
* **Total OPEN / IN-PROGRESS (4):** `DMA-CACHE-01`, `DMA-ADDR-01`, `ATOMIC-SPARC-01`, `VT-D-DRV-01` (PLANNED).

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
| **DMA-CACHE-01** | TINGGI | OPEN | `asl-hal/src/cache.rs` | Trait `CacheMaintenanceOps` + bounce buffer + buddy pool 64B pass; verifikasi zero-stale read via Gem5 / Zynq-7000 | Zero stale DMA read (QEMU TCG tidak sah; butuh Gem5 / hardware fisik) |
| **DMA-ADDR-01** | TINGGI | OPEN | `asl-hal/src/dma_space.rs` | Trait `DmaAddressSpace`; eliminasi `IdentityDmaSpace` di build DAL-A; 4 host unit tests pass (checksum Partisi B terlindungi) | Host logic = SIMULATION VERIFIED; open sampai fault-path hardware (VT-D-DRV-01) |
| **ATOMIC-SPARC-01** | SEDANG | OPEN | `asl-space-tmr/src/lib.rs` | Tahap 1 & 2 selesai: AtomicU32 downscale + AtomicU64Split + UniversalAtomicU64. Pengujian nightly sparc64 target. | Pengukuran empiris via TSIM3/GRMON |
| **VT-D-DRV-01** | SEDANG | PLANNED | Hardware IOMMU Engine | Pemrograman register root/context table VT-d Intel q35 & evaluasi hardware fault path | Diverifikasi pada QEMU q35 / target hardware fisik |

---

## 3. RENCANA EKSEKUSI SIKLUS 90-HARI
1. Eksekusi pengujian simulasi Gem5 untuk memvalidasi zero-stale read pada `CacheMaintenanceOps` (penutupan penuh `DMA-CACHE-01`).
2. Implementasi pembatasan `IdentityDmaSpace` compile-time error dan penegakan IOMMU translation space (`DMA-ADDR-01`).
3. Pengukuran siklus WCET pada TSIM3 / board hardware GR712RC untuk menutup `ATOMIC-SPARC-01`.
