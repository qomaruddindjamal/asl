# SECURITY FINDINGS: ASL CORE KERNEL AUDIT
**Lead Security Auditor:** ANTI-GRAVITY (Independent Lead Auditor & R&D Systems Engineer)  
**Date:** 2026-10-13  
**Review Scope:** Full Workspace (17 Workspace Crates + Bare-metal Entry + Documentation Map)  
**Governing Standard:** DO-178C DAL-A / ARINC 653 Part 1  

---

## 1. COMPREHENSIVE FINDINGS MATRIX

| Finding ID | Title | Severity | Confidence | Status | Acceptance Criteria / Disposition |
| :--- | :--- | :---: | :---: | :---: | :--- |
| **SEC-01** | Integer Overflow in `MemoryWindow::contains` Boundary Check | **HIGH** | CONFIRMED | **CLOSED** | 5 unit tests pass; `checked_add` boundary verified. |
| **SEC-02** | Unbounded Address Space Dereference in `AslBootHandover::validate` | **MEDIUM** | CONFIRMED | **CLOSED** | 7 unit tests pass; 52-bit max physical limit enforced. |
| **SEC-03** | Lack of Concurrency Protection in Multi-Core Shared Rings | **MEDIUM** | CONFIRMED | **CLOSED** | 10 unit tests pass; SPSC lock-free atomic acquire/release (Dilarang regresi). |
| **SEC-04** | Incomplete Bit-Inversion in `GponOntDriver::accelerate_hnat` | **LOW** | CONFIRMED | **CLOSED** | Explicit mock architecture documented; 2 unit tests pass. |
| **SEC-05** | Misleading Byte Swap in `MipsTlbFastpathEngine::normalize_packet_header_u32`| **LOW** | CONFIRMED | **CLOSED** | Explicit network byte order (`u32::from_be`); 2 tests pass. |
| **AG-02** | Path Inconsistency in README.md §2 vs Actual Cargo Workspace Output | **RENDAH** | CONFIRMED | **CLOSED** | README.md disinkronisasi ke `asl-kernel/target/`; `Test-Path` mengonfirmasi `True`. |
| **AG-03** | Runtime M1..M6 Hanya Terverifikasi pada Profil Build Debug | **SEDANG** | CONFIRMED | **CLOSED** | Re-capture dengan `--release` sukses; log M1..M6 release + idle 76s stabil. |
| **AG-05** | Traceability Governance: Bukti Uji Historis Tanpa Nama Fungsi Spesifik | **RENDAH** | CONFIRMED | **CLOSED** | 43 nama fungsi test dan SHA256 hashes terinventarisasi di `TEST_RESULTS.md`. |
| **ALIGN-01** | Strict 64-byte Hardware Cache Line Descriptor Alignment | **SEDANG** | CONFIRMED | **CLOSED** | `#[repr(C, align(64))]` + static asserts + `init_pool` check + 4 unit tests pass. |
| **AG-06** | Penanda Profil Build Release/Debug pada Serial Banner M1 | **RENDAH** | CONFIRMED | **CLOSED** | `build.rs` + `option_env!("ASL_BUILD_PROFILE")` + 1 unit test fallback pass. |
| **DMA-CACHE-01** | Explicit Cache Invalidation Barriers for Non-Coherent DMA | **TINGGI** | CONFIRMED | **OPEN** | Trait `CacheMaintenanceOps` di `asl-hal` + bounce buffer + buddy pool 64B pass. Verifikasi zero-stale read butuh Gem5/Zynq-7000 (QEMU TCG tidak sah). |
| **DMA-ADDR-01** | Physical Address Clipping & Cross-Partition DMA Exposure | **TINGGI** | CONFIRMED | **OPEN** | Trait `DmaAddressSpace`; eliminasi compile-time `IdentityDmaSpace` di build DAL-A; static IOVA windows. |
| **ATOMIC-SPARC-01** | SPARC V8 64-bit Atomic CAS Emulation Requirement | **SEDANG** | CONFIRMED | **OPEN** | Tahap 1 selesai: AtomicU32 downscale (wraparound pass) + AtomicU64Split (pass). Bounded WCET via TSIM3/GRMON3 = REQUIRES EXECUTION. |
| **AG-01** | Missing Volatile Semantics on Memory-Mapped I/O and Ring Polling | **SEDANG** | CONFIRMED | **CLOSED** | Wrapper `MmioRegister<u32>` di `asl-hal` & refactor `asl-cctv-dma` selesai (2 unit tests pass). Volatile hanya MMIO; inter-core atomic aman. |
| **AG-04** | Critical Section Berpotensi Reachable dari Interrupt Service Routine (ISR) | **SEDANG** | CONFIRMED | **CLOSED** | Draf kebijakan P-ISR-1..4 diamendemen (W-10 SPARC WRPSR 3 NOP delay, W-11 WCET budget table) & diratifikasi RATIFIED v1.0 final di `CANONICAL_MASTER.md`. Matriks 7/0 compliant. |
| **VT-D-DRV-01** | VT-d / AMD-Vi Hardware Driver & Fault-Path Programming | **SEDANG** | CONFIRMED | **PLANNED** | Driver minimal VT-d (root/context table programming) sebagai prasyarat fault-path QEMU q35 / target hardware, dipetakan ke milestone Fase 3. |

---

## 2. REKAPITULASI REGISTER TEMUAN TERKINI
* **Total Temuan Terdaftar:** 16 temuan (15 temuan awal + 1 finding terencana VT-D-DRV-01)
* **CLOSED / RESOLVED (12):** `SEC-01`, `SEC-02`, `SEC-03`, `SEC-04`, `SEC-05`, `AG-01`, `AG-02`, `AG-03`, `AG-05`, `AG-06`, `ALIGN-01`, `AG-04`.
* **OPEN / IN-PROGRESS (4):** `DMA-CACHE-01`, `DMA-ADDR-01`, `ATOMIC-SPARC-01`, `VT-D-DRV-01` (PLANNED).
