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
