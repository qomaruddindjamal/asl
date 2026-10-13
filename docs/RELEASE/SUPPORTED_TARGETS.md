# ASL CORE KERNEL: SUPPORTED TARGETS & SILICON TIER MATRIX
===========================================================
**Release Version:** v1.0.0-rc.1 / v1.0.0 Reference Baseline  
**Governing Authority:** `asl_core.md` (150 Unified Architectures)  

## 1. TARGET SUPPORT TIERS CLASSIFICATION

All 150 architectures cataloged in `asl_core.md` are formally organized into distinct engineering support tiers:

| Tier | Status Definition | Target Triple / Architecture Family | Concrete Verification Evidence |
| :--- | :--- | :--- | :--- |
| **Tier 1: Reference Host & CI** | **TEST VERIFIED (Host Simulation)** | `x86_64-pc-windows-msvc` (17 Workspace Crates) | 31 Unit tests passing (`cargo test`) |
| **Tier 2: Bare-Metal Hypervisor** | **RUNTIME VERIFIED (VirtualBox)** | `x86_64-unknown-none` (Limine Base v2, Higher-Half) | Milestones M1..M6 verified in `Analysis/reports/asl_vm_serial.log` |
| **Tier 3: Bare-Metal Staged** | **SPECIFIED / STRUCTURAL CRATES** | AArch64, RISC-V 64, MIPS64, SPARC V8, POWER10, s390x | Architectural crates compile and pass logic unit tests |
| **Tier 4: Staged Hardware BSPs** | **PLANNED / STAGED ROADMAP** | 150 Silicon Platforms (Network, Carrier GPON/OLT, CCTV, TMR) | Defined across 36-Month Flight Plan (`docs/ROADMAP/ROADMAP_MASTER.md`) |

## 2. EXPLICIT LIMITATIONS & OPEN FINDINGS

The following findings documented in `audit/asl-core-kernel/SECURITY_FINDINGS.md` remain **OPEN** and define explicit target-level boundaries:
1. `ATOMIC-SPARC-01`: SPARC V8 32-bit platforms require software CAS traps for 64-bit atomics.
2. `DMA-CACHE-01`: Non-coherent bus DMA masters require explicit cache clean/invalidate barriers.
3. `DMA-ADDR-01`: Physical address clipping occurs on legacy 32-bit PCI bridges without IOMMU bounce buffers.
4. `ALIGN-01`: 64-byte descriptor alignment required for heterogeneous cache line architectures.
