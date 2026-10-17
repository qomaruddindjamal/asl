# CANONICAL KNOWLEDGE SYSTEM: ASL CORE KERNEL ECOSYSTEM
=======================================================
**Status:** CANONICAL & AUTHORITATIVE  
**Version:** 1.0.0-rev1  
**Project:** ASL Core Kernel  
**Repository Root:** `C:\asl`  
**Git Remote:** `https://github.com/qomaruddindjamal/asl.git`  
**Governing Standard:** DO-178C DAL-A / ARINC 653 IMA  

---

## 1. PURPOSE & ARCHITECTURAL FOUNDATION

The **ASL Core Kernel** is an Integrated Modular Avionics (IMA) microkernel architecture written in safe, bare-metal Rust (`#![no_std]`). The design unifies 150 heterogeneous silicon computing, switching, and signal architectures into a deterministic, partitioned execution environment.

### Inverted Avionics Pyramid (Principle of Architectural Separation)
1. **Core Microkernel (`asl-kernel-core`):** Pure deterministic state coordinator. Zero heap fragmentation, O(1) buddy-slab allocation, ARINC 653 space-and-time partitioning, zero-copy AFDX SPSC ring buffers.
2. **Platform Launcher (`asl-kernel-bin`):** Higher-half Limine base protocol revision 2 bootloader adapter. Converts arbitrary firmware boot info into `AslBootHandover` contracts without polluting kernel core.
3. **Hardware Abstraction Layer (`asl-hal`):** Universal traits (`ArchitectureRuntime`, `InterruptController`, `DeterministicTimer`, `MemoryManagementUnit`, `EarlyDiagnosticConsole`) across all silicon tiers.
4. **Specialized Subsystem Crates:** Modular RTOS engine (`asl-nano`), network switching (`asl-pmd`, `asl-tcam`), optical GPON/EPON framing (`asl-ont-gpon`, `asl-olt-carrier`), DMA engines (`asl-cctv-dma`), radiation-hardened space voting (`asl-space-tmr`), and mission-critical mainframes (`asl-s390x-mainframe`).

---

## 2. DOCUMENTATION AUTHORITY MATRIX

To prevent ambiguity, each domain of knowledge has exactly one authoritative master:

| Domain | Authoritative Primary Source | Status | Secondary / Historical References |
| :--- | :--- | :---: | :--- |
| **System Architecture & Design** | `asl_avionics_kernel_blueprint.md` | **AUTHORITATIVE** | `docs/CANONICAL/CANONICAL_MASTER.md`, `asl_core.md` |
| **Complete 150 Taxonomy** | `asl_core.md` | **AUTHORITATIVE** | `master_150_architectures.md`, `120_board_architectures.md` |
| **Governance & Locking Gates** | `docs/ROADMAP/CHECKLIST_STATE.json` | **AUTHORITATIVE** | `tools/Gemini_Assist.py`, `PHASE_01..06.md` |
| **Active Handoff State** | `docs/ROADMAP/HANDOFF_STATE.json` | **AUTHORITATIVE** | `docs/ROADMAP/README.md` |
| **Accountability & Integrity** | `docs/ROADMAP/ACCOUNTABILITY_FAULT_LOG.json` | **AUTHORITATIVE** | `tools/Accountability_Watchdog.py` |
| **Technical Findings & Audits** | `audit/asl-core-kernel/SECURITY_FINDINGS.md` | **AUTHORITATIVE** | `audit/asl-core-kernel/*.md` |
| **Bare-Metal Boot & ISO Pipeline**| `docs/CANONICAL/DEVELOPER_ONBOARDING.md` | **AUTHORITATIVE** | `asl-kernel-bin/src/main.rs`, `linker.ld` |
| **Developer Onboarding & SOP** | `docs/CANONICAL/DEVELOPER_ONBOARDING.md` | **AUTHORITATIVE** | `README_ASL_CORE.md`, `Analysis/README.md` |

---

## 3. ARCHITECTURE & ROADMAP CONFLICT REGISTER

### Conflict 1: Macro Roadmap vs Phase Checklists
* **Description:** `docs/ROADMAP/ROADMAP_MASTER.md` projects a multi-year roadmap across 6 phases. Meanwhile, `docs/ROADMAP/CHECKLIST_STATE.json` records all 25 checklist gates of Phase 1 to Phase 6 as `PASS & LOCKED`.
* **Resolution & Clarification:**
  - `CHECKLIST_STATE.json` gates represent **Architectural Specification & Proof-of-Concept Implementation Gates** (all 26 unit tests passed in host simulation and crate level).
  - Physical target deployment for all 150 hardware chips remains in progress along the staged engineering timeline.
  - No locked gate in `CHECKLIST_STATE.json` shall be modified or reopened.

### Conflict 2: Board Taxonomy Redundancy
* **Description:** Three documents catalog silicon boards: `asl_core.md` (150 architectures, authoritative), `master_150_architectures.md` (older draft), and `120_board_architectures.md` (early 120-chip draft, local-only).
* **Resolution:**
  - `asl_core.md` is declared the single canonical encyclopedia for the 150-architecture taxonomy.
  - `master_150_architectures.md` and `120_board_architectures.md` are marked as superseded historical drafts.

### Conflict 3: Open Findings Tracking
* **Description:** Four technical audit findings were identified during cross-target review:
  - `ATOMIC-SPARC-01` (lack of 64-bit atomic target support on generic SPARC bare-metal)
  - `DMA-CACHE-01` (explicit cache invalidation barrier requirement for DMA descriptors)
  - `DMA-ADDR-01` (IOMMU translation vs raw physical addresses in DMA burst)
  - `ALIGN-01` (descriptor alignment enforcement)
* **Resolution:**
  - All four findings remain formally **OPEN** in the project risk register until dedicated BSP drivers with hardware testbeds are deployed.

---

## 4. CANONICAL INVARIANTS & SAFETY POLICIES

1. **Zero Monolithic Kernel Panics (DO-178C DAL-A):** Subsystem faults must terminate into partition-isolated safe fallback loops; the microkernel coordinator must never execute unrecoverable panics.
2. **Strict Stack Sizing:** All higher-half x86_64 stubs must allocate a minimum of **512 KB** stack aligned to page boundaries (4096 bytes) to accommodate the complete avionics core runtime state.
3. **PT_LOAD Page Alignment:** Linker scripts and target flags must guarantee `p_align = 0x1000` (4096 bytes) on all loadable segments (`-z max-page-size=0x1000`) to guarantee compatibility with Limine and modern UEFI page tables.
4. **Kernel Self-Overwrite Prevention:** The bootloader memory map adapter must unconditionally extract the physical kernel memory range (`kphys..kphys+ksize`) out of the usable RAM descriptor list before handover.

---

## 5. INTERRUPT SERVICE ROUTINE (ISR) LOCKING POLICIES & AUDIT INVENTORY (AG-04)
**Status:** RATIFIED v1.0 (Final - Amended W-10 & W-11)  
**Governing Safety Standard:** DO-178C DAL-A / ARINC 653 Part 1

### 5.1 Formal Safety Policies (P-ISR-1 to P-ISR-4)
* **P-ISR-1 (No Dynamic Allocation & Zero Panic):** Kode yang dieksekusi di dalam konteks Interrupt Service Routine (ISR) **DILARANG KERAS** melakukan alokasi dinamis pada heap, memanggil panic liar, atau operasi tanpa batas waktu. Seluruh operasi wajib bounded Worst-Case Execution Time (WCET).
* **P-ISR-2 (Lock-Free SPSC Unidirectional Streaming):** Pertukaran data antara konteks ISR dan thread kernel/partisi normal **WAJIB** menggunakan struktur lock-free Single-Producer Single-Consumer (SPSC) searah (seperti AFDX ring buffer atau zero-copy PMD queue) bila memungkinkan, untuk meniadakan ketergantungan mutual exclusion.
* **P-ISR-3 (Uniprocessor Mutual Exclusion & Interrupt Flag Preservation - Amended W-4 & W-10):** Pada arsitektur single-core / uniprocessor, sinkronisasi mutual exclusion antara ISR dan thread normal **WAJIB** dilakukan melalui penonaktifan interupsi lokal sementara dengan penyimpanan status flag sebelumnya (*save-flags pattern*):
  - **x86_64:** Wajib menggunakan pola save-flags (`pushfq; cli` dan `popfq`), **DILARANG BLIND `sti`** yang dapat mengaktifkan interupsi prematur saat berada di dalam critical section bersarang.
  - **AArch64:** Wajib memanipulasi register status DAIF (`msr daifset, #2` untuk mask IRQ, dan `msr daifclr, #2` untuk unmask IRQ) dengan menyimpan nilai DAIF sebelumnya (`mrs %0, daif`).
  - **SPARC V8 (Amended W-10):** Wajib mengeksekusi urutan lengkap read-modify-write Processor Status Register (PSR) dengan 3 instruksi delay slot NOP untuk mengantisipasi pipeline hazard / register write-back delay:
    ```assembly
    rd %psr, %l0
    or %l0, 0x0F00, %l1      ! Mask PIL = 15 (disable interrupts)
    wr %l1, %g0, %psr
    nop                      ! SPARC V8 WRPSR delay slot 1
    nop                      ! SPARC V8 WRPSR delay slot 2
    nop                      ! SPARC V8 WRPSR delay slot 3
    /* Critical Section */
    wr %l0, %g0, %psr        ! Restore original PIL
    nop
    nop
    nop
    ```
  - **Spinlock reguler di dalam ISR DILARANG KERAS** karena memicu deadlock instan jika thread yang diinterupsi sedang memegang lock.
* **P-ISR-4 (SMP Multi-Core Lock Protocol & WCET Budgets - Amended W-11):** Pada arsitektur multi-core (SMP), critical section yang diakses bersama antara thread dan ISR wajib menggabungkan masking interupsi lokal CPU *dan* spinlock hardware (`spin_lock_irqsave`). Batas waktu WCET tidak menggunakan angka global arbitrary melainkan dialokasikan berdasarkan sistem avionik DO-178C per-ISR budget table:

#### Tabel Alokasi Budget WCET per-ISR (DO-178C DAL-A Allocation)
| Domain ISR / Hardware IRQ | Sumber Requirement Sistem | WCET Budget | Karakteristik Critical Section |
| :--- | :--- | :---: | :--- |
| **PIT / APIC Timer Tick** | ARINC 653 Part 1 §3.2 (Major Frame Dispatch) | **≤ 1.5 µs** | Pembacaan tick counter, update minor frame index, slot dispatch |
| **NIC RX Descriptor IRQ** | ARINC 664 Part 7 (AFDX Deterministic Ingestion) | **≤ 3.0 µs** | Handoff ownership pointer descriptor ring, enqueue SPSC frame |
| **VPU/ISP Frame Done** | RTCA DO-254 / CCTV Sensor Streaming | **≤ 2.5 µs** | Volatile MMIO flag acknowledgment, channel buffer swap |
| **UART Serial Telemetry** | ASL Diagnostics & Flight Telemetry Subsystem | **≤ 4.0 µs** | Non-blocking ring buffer write, COM port TX FIFO poll |

### 5.2 ISR Lock Sites Inventory & Compliance Matrix (7 Baris Lengkap - AG-04 Closure)

| File & Baris Kode | Data yang Dilindungi | Mekanisme Pelindung | Konteks Eksekusi | Referensi Policy | Status Kepatuhan |
| :--- | :--- | :--- | :--- | :---: | :---: |
| `asl-pmd/src/lib.rs:71` | Deskriptor ring RX (`rx_ring`, `flags`) | Atomic `Acquire`/`Release` (Lock-Free) | Polled-Mode / IRQ Driver Task | P-ISR-2 | **COMPLIANT** |
| `asl-pmd/src/lib.rs:96` | Deskriptor ring TX (`tx_ring`, `flags`) | Atomic `Acquire`/`Release` (Lock-Free) | Driver Thread / Transmit Task | P-ISR-2 | **COMPLIANT** |
| `asl-afdx-ipc/src/lib.rs:88`| Antrean frame pesan AFDX (`buffer`, `tail`) | Atomic `Acquire`/`Release` SPSC | Inter-Partition IPC Task | P-ISR-2 | **COMPLIANT** |
| `asl-dsp-rpc/src/lib.rs:86` | Antrean Mailbox CPU-DSP | Atomic `Acquire`/`Release` SPSC | Host Task & DSP Consumer | P-ISR-2 | **COMPLIANT** |
| `asl-hal/src/cache.rs:43` | Barrier serialisasi hardware | Compiler Fence / CPU Instruction | Driver Flush / HAL Invalidation | P-ISR-1 | **COMPLIANT** |
| `asl-cctv-dma/src/lib.rs:55`| Status `vpu_lock_flags` & counter | Volatile `MmioRegister<u32>` + Thread Context | Sensor IRQ & Frame Capture Task | P-ISR-2 / P-ISR-3 | **COMPLIANT** |
| `asl-space-tmr/src/lib.rs:38`| Majority voting memory variables | Direct Structure Copy / Local Stack | Background Scrubber Task | P-ISR-1 | **COMPLIANT** |


