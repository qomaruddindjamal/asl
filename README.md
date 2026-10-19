# ASL CORE KERNEL: CANONICAL OS DEVELOPMENT ENTRY POINT
================================================================================
**Project:** ASL Core Kernel
**Repository Root:** `C:\asl`
**Workspace:** `C:\asl\asl-kernel`
**Git Remote:** `https://github.com/qomaruddindjamal/asl.git`
**Audit Baseline Commit:** `5efedde`
**Audit Verification Date:** 2026-10-13
**Governing Standard:** DO-178C DAL-A / ARINC 653 Integrated Modular Avionics (IMA)

---

## 1. PROJECT IDENTITY & PURPOSE

**ASL Core Kernel** is an ultra-reliable, bare-metal microkernel written in modern, dependency-free Rust (`#![no_std]`). Engineered around the principles of **Integrated Modular Avionics (IMA)**, ASL provides a deterministic, partitioned execution environment designed to orchestrate 150 diverse silicon computing architectures—from mission-critical avionics computers and flight controllers to enterprise fiber switching, optical transceivers (GPON/OLT), video surveillance NPUs, and space-grade processors.

---

## 2. VERIFIED PROJECT STATUS (EVIDENCE-FIRST BASELINE)

All status claims in this repository follow strict evidence-first classification:

| Milestone / Component | Claimed Scope | Verified Status | Concrete Evidence / Artifact |
| :--- | :--- | :---: | :--- |
| **Workspace Unit Tests** | 31 Workspace Crates (30 lib + 1 bin launcher) | **TEST VERIFIED (2026-10-20)** | 128 unit tests passed, 0 failed on host target `x86_64-pc-windows-msvc` via `cargo test --workspace --lib` (no `--target` flag required; bare-metal builds must pass an explicit `--target`) |
| **Bare-Metal Compiler Target** | `x86_64-unknown-none` | **BUILD VERIFIED** | ELF64 executable compiled cleanly at `asl-kernel/target/x86_64-unknown-none/release/asl-kernel` |
| **Higher-Half Launcher** | Limine Base Protocol v2 | **IMPLEMENTED** | `asl-kernel-bin/src/main.rs`, dedicated 512 KB static stack, page-aligned PT_LOAD (`p_align=0x1000`), FLAGS(6) |
| **Avionics Bootable ISO** | El Torito BIOS + UEFI Hybrid | **BUILD VERIFIED** | `asl-kernel/target/asl-core-kernel.iso` (3.917.824 bytes) via `xorriso` v1.5.2 + `limine bios-install` |
| **Hypervisor Execution** | VirtualBox VM `ASL` | **RUNTIME VERIFIED** | Headless boot verified; VM boots to completion and executes deterministic ARINC 653 idle loop |
| **Linear Framebuffer Display** | VBE / GOP 1024x768x32-bpp | **RUNTIME VERIFIED** | Direct screen render verified in `Analysis/reports/asl_vm_screen.png`: ASL ASCII Art banner & `[ OK ]` status |
| **Serial Telemetry M1..M6** | COM1 UART Port (`0x3F8`) | **RUNTIME VERIFIED** | Captured in `Analysis/reports/asl_vm_serial_release.log`: M1 (Entry), M2 (Handover), M3 (Transfer), M4 (Buddy), M5 (ARINC 653), M6 (Idle) |
| **Governance Gates** | Phase 1–6 Gates (24 total) + Milestone_03 Storage (25 gates) | **LOCKED (Phase 1–6) / PARTIAL (Milestone_03)** | `docs/ROADMAP/CHECKLIST_STATE.json`: 24/24 Phase 1–6 gates `PASS & LOCKED`; `Milestone_03` = **PARTIAL, unlocked** — 21 PASS, Gate 3.6 PARTIAL (AHCI DMA), Gate 3.9 PASS_READONLY (ext4), Gate 3.19 PASS with Boot-Assembly GAP, Gate 3.20 GAP_IDENTIFIED (RAID rebuild), Gate 3.25 NOT TESTED (physical HW). Reconciliation evidence: `Analysis/reports/DEEP_AUDIT_REPORT.md` §1 + deterministic gate parse `Analysis/reports/2026-10-20_gate_parse_evidence.txt` + QODER_EXECUTION_LOG (2026-10-20). |
| **150 Hardware Architecture Deployment** | Physical Board Silicon Deployment | **PLANNED / STAGED** | Formal specification complete in `asl_core.md`; physical board BSPs scheduled across 36-month roadmap |

---

## 3. ARCHITECTURE AUTHORITY & DOCUMENTATION MAP

To eliminate discrepancies between drafts and active implementations, the following documents serve as the authoritative sources of truth:

```
[DOCUMENTATION AUTHORITY HIERARCHY]
  │
  ├── System Architecture & Canonical Invariants ──────► docs/CANONICAL/CANONICAL_MASTER.md
  ├── Complete 150-Architecture Hardware Taxonomy ──────► asl_core.md (Single Source of Truth)
  ├── Developer SOP & Toolchain Setup ─────────────────► docs/CANONICAL/DEVELOPER_ONBOARDING.md
  ├── Full Filesystem Inventory (110 Active Files) ─────► docs/CANONICAL/FILE_INVENTORY_AND_AUDIT_MATRIX.md
  ├── Formal Milestone Checklist & Gate Locks ──────────► docs/ROADMAP/CHECKLIST_STATE.json
  ├── Active Workflow Handoff State ────────────────────► docs/ROADMAP/HANDOFF_STATE.json
  ├── Technical Risk & Security Audit Register ────────► audit/asl-core-kernel/SECURITY_FINDINGS.md
  └── Strategic 36-Month Engineering Flight Plan ──────► docs/ROADMAP/ROADMAP_MASTER.md
```

> **Note on Historical Documents:** Files such as `master_150_architectures.md` and `120_board_architectures.md` are preserved historical drafts superseded by the authoritative encyclopedia in [`asl_core.md`](file:///c:/asl/asl_core.md).

---

## 4. OPEN AUDIT FINDINGS & TECHNICAL LIMITATIONS

In compliance with the project's strict accountability charter, the following technical findings remain formally **OPEN** in the project risk register (`audit/asl-core-kernel/SECURITY_FINDINGS.md`, 16 entries: 12 CLOSED / 3 OPEN / 1 PLANNED) until dedicated target hardware testbeds are deployed:

1. **`ATOMIC-SPARC-01` (OPEN):** Generic bare-metal SPARC target (`sparc-unknown-none-elf`) lacks compiler 64-bit atomic support (`target_has_atomic="64"`). Crate dependencies requiring `AtomicU64` require target-specific CAS emulations.
2. **`DMA-CACHE-01` (OPEN):** Physical DMA driver ring buffers require explicit hardware cache invalidation and clean barriers (`dma_cache_clean_inval`) across non-coherent CPU architectures.
3. **`DMA-ADDR-01` (OPEN):** The `transmit_burst()` interface currently takes raw physical memory addresses; integration with IOMMU translation layers is required for multi-tenant partition safety.
4. **`VT-D-DRV-01` (PLANNED):** Minimal VT-d / AMD-Vi root/context table driver is a prerequisite for hardware fault-path verification (mapped to roadmap Fase 3).

> **Correction (2026-10-20):** `ALIGN-01` was previously listed here as OPEN. It is **CLOSED** in the register and in code (`asl-pmd/src/lib.rs:35` `#[repr(C, align(64))]` + static asserts + runtime `init_pool` check + 4 unit tests). This section now matches the register exactly.

---

## 5. OS DEVELOPMENT WORKFLOW & ENTRY POINT

Developing an Operating System subsystem, server, or driver on top of the ASL Core Kernel proceeds in structured, dependency-ordered stages:

```
[STAGE 1: CANONICAL SPECIFICATION]
Review docs/CANONICAL/CANONICAL_MASTER.md and understand ARINC 653 partitioning rules.
                    │
                    ▼
[STAGE 2: BARE-METAL HOST REPRODUCTION]
Verify build toolchains: `rustc`, `cargo`, `xorriso`, and Limine v8.x binary utilities.
                    │
                    ▼
[STAGE 3: HANDOVER CONTRACT INTEGRATION]
Interact exclusively via `AslBootHandover` (defined in `asl-bootloader`). Never assume BIOS/firmware states.
                    │
                    ▼
[STAGE 4: ARINC 653 VAULT PARTITIONING]
Register spatial memory windows using `SpatialPartitionManager` and assign temporal major/minor slots in `MajorFrameSchedule`.
                    │
                    ▼
[STAGE 5: ZERO-COPY AFDX RING BUFFER IPC]
Inter-process communication between OS daemons must use lock-free SPSC AFDX channels (`asl-afdx-ipc`).
                    │
                    ▼
[STAGE 6: INDEPENDENT AUDIT & GATE LOCKING]
Execute workspace regression tests and run `tools/Gemini_Assist.py --audit` before milestone sign-off.
```

---

## 6. PREREQUISITES & DEVELOPMENT COMMANDS

### Hardware & Runtime Requirements (Verified v1.0.1; footprint updated 2026-10-20)
- **Minimum Target RAM:** **4 MB RAM** (bare-metal release kernel ELF measured **211.672 bytes** ≈ 207 KB on 2026-10-20, `target/x86_64-unknown-none/release/asl-kernel`; the historical "~70 KB" figure measured the v1.0.1 minimal launcher before storage/driver/shell integration. Verified running stably with zero crash on 64 MB).
- **Recommended Target RAM:** **64 MB RAM** or higher.
- **CPU Architecture:** x86_64 (Long Mode + PAE supported).
- **Firmware:** BIOS (El Torito CD boot) or UEFI 64-bit with GOP / VBE framebuffer.

### Host Prerequisites & Build Toolchain
- **Host OS:** Windows 10/11 64-bit (or Linux x86_64).
- **Rust Toolchain:** `rustc 1.80+` with targets:
  - `x86_64-pc-windows-msvc` (Host compiler & regression harness)
  - `x86_64-unknown-none` (Bare-metal ELF kernel target)
- **ISO Tools:** `xorriso` (v1.5.0+) and Limine v8.x binaries.
- **Hypervisor:** Oracle VirtualBox (GUI or headless) with VM named `ASL`, or QEMU (`qemu-system-x86_64`).

### Essential Build & Verification Commands

```powershell
# 1. Run full host workspace regression tests (All 31 crates, official command)
cd C:\asl\asl-kernel
cargo test --workspace --lib

# 2. Build the bare-metal higher-half launcher binary
cd C:\asl\asl-kernel
cargo build --package asl-kernel-bin --target x86_64-unknown-none --release

# 3. Master the bootable hybrid ISO
Copy-Item target\x86_64-unknown-none\release\asl-kernel target\iso_root\boot\asl-kernel.elf -Force
xorriso -as mkisofs -b boot/limine/limine-bios-cd.bin -no-emul-boot -boot-load-size 4 -boot-info-table --efi-boot boot/limine/limine-uefi-cd.bin -efi-boot-part --efi-boot-image --protective-msdos-label /cygdrive/c/asl/asl-kernel/target/iso_root -o /cygdrive/c/asl/asl-kernel/target/asl-core-kernel.iso
C:\asl\asl-kernel\target\limine_v8_binary\limine.exe bios-install C:\asl\asl-kernel\target\asl-core-kernel.iso

# 4. Launch VirtualBox VM headless and observe runtime telemetry
& 'C:\Program Files\Oracle\VirtualBox\VBoxManage.exe' startvm 'ASL' --type headless

# 5. Verify local workspace hygiene (Must return 100% clean)
python C:\asl\tools\Sub_Assist.py
```

---

## 7. REPOSITORY INTEGRITY & LOCAL-ONLY GOVERNANCE

To protect local research environments and prevent intellectual property or governance leaks:
- The governance scripts (`tools/`), project directives (`AGENT.md`, `GEMINI.md`), and raw serial logs (`*.log`) are strictly excluded from remote tracking via [`.gitignore`](file:///c:/asl/.gitignore).
- All changes must pass `python tools/Sub_Assist.py --hygiene` prior to commit.
- The 24 Phase_01..06 gates in [`docs/ROADMAP/CHECKLIST_STATE.json`](file:///c:/asl/docs/ROADMAP/CHECKLIST_STATE.json) are permanently **PASS & LOCKED** and must not be altered.

---

## 8. LICENSE & COPYRIGHT

ASL Core Kernel is free software: you can redistribute it and/or modify it under the terms of the GNU General Public License as published by the Free Software Foundation, either version 3 of the License, or (at your option) any later version.

Copyright © 2026 qomaruddindjamal.
