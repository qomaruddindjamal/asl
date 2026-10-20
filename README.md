# ASL CORE KERNEL: CANONICAL OS DEVELOPMENT ENTRY POINT
================================================================================
**Project:** ASL Core Kernel
**Repository Root:** `C:\asl`
**Workspace:** `C:\asl\asl-kernel`
**Git Remote:** `https://github.com/qomaruddindjamal/asl.git`
**Audit Baseline Commit:** `5efedde`
**Audit Verification Date:** 2026-10-13
**Governing Standard:** DO-178C-aligned, ladder DAL-C->A (lihat ADR-002) / ARINC 653 Integrated Modular Avionics (IMA)

---

## 1. PROJECT IDENTITY & PURPOSE

**ASL Core Kernel** is an ultra-reliable, bare-metal microkernel written in modern, dependency-free Rust (`#![no_std]`). Engineered around the principles of **Integrated Modular Avionics (IMA)**, ASL provides a deterministic, partitioned execution environment designed to orchestrate 150 diverse silicon computing architectures—from mission-critical avionics computers and flight controllers to enterprise fiber switching, optical transceivers (GPON/OLT), video surveillance NPUs, and space-grade processors.

---

## 2. VERIFIED PROJECT STATUS (EVIDENCE-FIRST BASELINE)

All status claims in this repository follow strict evidence-first classification:

| Milestone / Component | Claimed Scope | Verified Status | Concrete Evidence / Artifact |
| :--- | :--- | :---: | :--- |
| **Workspace Unit Tests** | 31 Workspace Crates (30 lib + 1 bin launcher) | **TEST VERIFIED (2026-10-20)** | **172 unit tests passed, 0 failed** on host target `x86_64-pc-windows-msvc` via `cargo test --workspace --lib` (no `--target` flag required; bare-metal builds must pass an explicit `--target`). Source of truth for this run: `Analysis/reports/ci_2026-10-20.log` langkah `[1/4]`. Historical progression measured in the same mission: 128 (pre-misi) → 149 (Fase-2) → 160 (Fase-3) → 171 (PHASE-5B) → 172 (PHASE-5C, test signature latch stack-overflow). |
| **Bare-Metal Compiler Target** | `x86_64-unknown-none` | **BUILD VERIFIED** | ELF64 executable compiled cleanly at `asl-kernel/target/x86_64-unknown-none/release/asl-kernel` |
| **Higher-Half Launcher** | Limine Base Protocol v2 | **IMPLEMENTED** | `asl-kernel-bin/src/main.rs`, dedicated 512 KB static stack, page-aligned PT_LOAD (`p_align=0x1000`), FLAGS(6) |
| **Avionics Bootable ISO** | El Torito BIOS + UEFI Hybrid | **BUILD VERIFIED** | `asl-kernel/target/asl-core-kernel.iso` (**4.118.528 bytes**, terukur 2026-10-20 oleh `scripts/ci_local.ps1`; angka lama "3.917.824 bytes" adalah ukuran pra-fase dan sudah usang) via `xorriso` v1.5.2 + `limine bios-install`. Catatan kebenaran: **ISO tidak byte-reproducible** (xorriso menanam volume timestamp) sehingga ukuran tetap sama tetapi SHA256 berubah tiap rebuild — SHA256 hanya boleh dikutip dari artefak log run bersangkutan |
| **IOMMU / VT-d fault path** | QEMU q35 `-device intel-iommu` cold-boot (QM-01) | **VERIFIED-ON-VM (2026-10-20, PHASE-5B)** — **BUKAN** VERIFIED-ON-HARDWARE | `Analysis/reports/qm01_vtd_serial_2026-10-20.log`: DMAR/ACPI parse nyata, domain second-level 3-level terpasang per partisi, GCMD `TE|SRTP|QIE` diterima (GSSTS TES/RTPS/QIES=1), IOVA unmapped **DITOLAK** dengan fault record terencode, QI invalidate+wait **DRAINED**. Konfirmasi board fisis masih terbuka (`VT-D-DRV-01`, `DMA-ADDR-01`) |
| **Stack-overflow guard page end-to-end** | QEMU q35 cold-boot, injeksi deliberate pada build ber-fitur `test-hv2-stack-overflow` | **VERIFIED-ON-VM (2026-10-20, PHASE-5C)** — **BUKAN** VERIFIED-ON-HARDWARE | `Analysis/reports/hv2_stack_overflow_serial_2026-10-20.log`: bukti paging "guard → NOT MAPPED", `#PF` vektor 14 dengan CR2 di halaman guard (`#PF` dipindah ke IST1 sehingga overflow tidak lagi tersamar jadi #DF), cetak verbatim **`STACK OVERFLOW (GUARD PAGE HIT)`**, DR FSM reason=3 (MemoryIntegrity) Armed→Recording→Recovery, `Controlled Halt Active`. Boot nominal (tanpa fitur) tetap capai M6 + shell. Tidak ada screenshot (VM headless `-display none`) |
| **Minimal CI pipeline (PHASE-5D)** | 4 langkah fail-closed | **RUNTIME VERIFIED (lokal)** + workflow hosted **VERIFIED-ON-CI untuk langkah [1]–[2]** (langkah [3]–[4] TIDAK dicakup hosted) | Lokal: `scripts/ci_local.ps1` → `Analysis/reports/ci_2026-10-20.log`: `[1/4]` 172/172, `[2/4]` 0 error 0 warning ELF 243.568 B (Windows), `[3/4]` M6 + shell, `[4/4]` ringkasan HIJAU. **Hosted (PHASE-6A):** run `asl-ci-minimal` id `39399083819` pada `headSha 8b829d8` = `conclusion success` — 30 suite **172 passed / 0 failed** di `ubuntu-latest`, langkah build release **0 warning / 0 error**, ELF ter-link di Linux **243.560 B** (8 B lebih kecil dari artefak Windows; lihat `Analysis/reports/elf_footprint_history.txt`), log: https://github.com/qomaruddindjamal/asl/actions/runs/39399083819 . Kebijakan: **pipeline MERAH = tidak boleh merge** |
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

In compliance with the project's strict accountability charter, the following technical findings remain formally unresolved in the project risk register (`audit/asl-core-kernel/SECURITY_FINDINGS.md`, 16 entries: 12 CLOSED / 2 VERIFIED-ON-VM / 1 PARTIAL-MITIGATED-CODE / 1 OPEN) until dedicated target hardware testbeds are deployed:

1. **`ATOMIC-SPARC-01` (OPEN):** Generic bare-metal SPARC target (`sparc-unknown-none-elf`) lacks compiler 64-bit atomic support (`target_has_atomic="64"`). Crate dependencies requiring `AtomicU64` require target-specific CAS emulations.
2. **`DMA-CACHE-01` (PARTIAL-MITIGATED, CODE):** Bounce staging via `asl-hal::dma_space::BounceAddressSpace::stage()` removes DMA-on-stale-caller-buffer by construction (hardware only touches the coherent staging pool). Explicit hardware cache invalidation/clean barriers (`dma_cache_clean_inval`) across non-coherent CPU architectures remain pending a real BSP/testbed.
3. **`DMA-ADDR-01` (VERIFIED-ON-VM — VERIFIED-ON-HARDWARE masih PENDING):** API-nya lease-typed (`transmit_burst()` / `setup_channel_buffer()` hanya menerima `&IovaLease` hasil `DmaAddressSpace::pin/stage`; alamat fisik mentah ditolak kompilator), dan sejak **PHASE-5B (2026-10-20)** jalur VT-d nyata dibuktikan **di QEMU q35**: domain second-level 3-level terpasang per partisi, IOVA unmapped **DITOLAK** dengan fault record terencode, QI invalidate+wait **DRAINED**. Bukti: `Analysis/reports/qm01_vtd_serial_2026-10-20.log`. **VERIFIED-ON-VM ≠ VERIFIED-ON-HARDWARE** (ADR-002 §5) — konfirmasi board fisis tetap terbuka.
4. **`VT-D-DRV-01` (VERIFIED-ON-VM, PHASE-5B 2026-10-20):** pemrograman root/context table + second-level + QI driver VT-d di `asl-hal/src/vtd.rs` + `asl-kernel-core/src/vtd_boot.rs` diterima model hardware `intel-iommu`; sisa pekerjaan hanya konfirmasi timing/fault di silicon riil.

> **Correction (2026-10-20):** `ALIGN-01` was previously listed here as OPEN. It is **CLOSED** in the register and in code (`asl-pmd/src/lib.rs:35` `#[repr(C, align(64))]` + static asserts + runtime `init_pool` check + 4 unit tests). This section now matches the register exactly.

> **Correction (2026-10-20, FASE-3):** Items 2–3 above previously read "interface currently takes raw physical memory addresses / requires explicit cache barriers, OPEN". That described the pre-RND_08 state and is superseded: the lease-typed API is implemented and host-verified (+11 new/updated DMA tests in FASE-3 across `asl-hal` TM-01..TM-08, `asl-pmd`, `asl-cctv-dma`; workspace total 160/160); only hardware verification remains. Full design, threat model, and test matrix: `docs/RND/RND_08_IOMMU_DMA_ISOLATION.md`.

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

### Hardware & Runtime Requirements (Verified v1.0.1; footprint updated 2026-10-20, measurement-evidence format per PHASE-5A)
- **Minimum Target RAM:** **4 MB RAM** (bare-metal release kernel ELF currently **243.568 bytes** ≈ 238 KB, measured on 2026-10-20 at the `[PHASE-5C]` stack-overflow-injection tree — see append-only evidence file `Analysis/reports/elf_footprint_history.txt` for the commit hash and delta; do NOT quote a footprint number here without adding a matching line to that file first. The historical "~70 KB" figure measured the v1.0.1 minimal launcher, not the current binary. Verified running stably with zero crash on 64 MB).
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

# 6. PHASE-5D minimal CI — local runner yang SAH (4 langkah, fail-closed)
#    Output: Analysis\reports\ci_<tanggal>.log (ASCII) + transcript per langkah
pwsh -File C:\asl\scripts\ci_local.ps1
```

### Continuous Integration (CI) minimal — PHASE-5D

Dua jalur, keduanya wajib hijau sebelum merge:

| Langkah | `.github/workflows/asl-ci-minimal.yml` (hosted) | `scripts/ci_local.ps1` (lokal) |
|---|---|---|
| [1] `cargo test --workspace --lib` | ya | ya (parse `test result:` — 0 failed) |
| [2] build `x86_64-unknown-none --release` | ya | ya + gerbang **0 error / 0 warning** (parse keluaran) |
| [3] QEMU q35 cold-boot smoke sampai **M6** + prompt shell | **tidak** | ya |
| [4] Ringkasan + bukti footprint + keputusan pipeline | **tidak** | ya |

- **Kebijakan merge: pipeline MERAH = TIDAK BOLEH merge.** Gerbang fase G4 (test semua lulus + build release 0 error 0 warning) adalah gerbang yang sama yang ditegakkan runner ini.
- Workflow hosted **tidak dapat** menjalankan langkah [3]: ia butuh hypervisor QEMU, ISO hasil xorriso (build cygwin), Limine bios-install, dan path lokal. Karena itu workflow hanya menegakkan [1]–[2]; langkah [3]–[4] wajib dijalankan lewat `scripts/ci_local.ps1` sebelum commit fase.
- Workflow sengaja **tidak** menyetel env `RUSTFLAGS`: `asl-kernel/.cargo/config.toml` memuat rustflags wajib per-target (`-T linker.ld`, `max-page-size=0x1000`, `relocation-model=static`, `code-model=kernel`), dan env `RUSTFLAGS` akan **menggantikan** (bukan menambah) nilai itu sehingga linking gagal. Gerbang "0 warning" ditegakkan oleh parsing `ci_local.ps1`, bukan `-D warnings`.
- **Bukti hosted (PHASE-6A, 2026-10-20):** workflow sudah benar-benar berjalan di runner GitHub — run `39399083819`, trigger `push` ke `security/asl-v1.0.2-audit`, `headSha 8b829d8`, `conclusion=success`, durasi job 20 s. Nomor ini **VERIFIED-ON-CI**, bukan CODE-COMPLETE: log mentahnya ditarik dan dibaca (`gh run view 39399083819 --log`), bukan hanya dipercaya dari status hijau.
- **Delta platform yang dicatat jujur:** artefak ELF yang sama menghasilkan **243.560 B di `ubuntu-latest`** vs **243.568 B di Windows** (selisih 8 B karena pautan/identitas biner per-platform). Angka yang dikutip §6 tetap **243.568 B** karena itulah biner yang di-boot di VM lokal; riwayat lengkapnya di `Analysis/reports/elf_footprint_history.txt`.
- Catatan kebenaran artefak: build ISO **tidak byte-reproducible** (xorriso menanam volume timestamp), jadi ukuran ISO tetap 4.118.528 B tetapi SHA256 berubah tiap rebuild — SHA256 di log CI hanya sah untuk run tersebut.

---

## 7. REPOSITORY INTEGRITY & LOCAL-ONLY GOVERNANCE

To protect local research environments and prevent intellectual property or governance leaks:
- The governance scripts (`tools/`), project directives (`AGENT.md`, `GEMINI.md`), and raw serial logs (`*.log`) are excluded from remote tracking via [`.gitignore`](file:///c:/asl/.gitignore).
- **Koreksi nuansa (PHASE-5D):** `.gitignore` mengecualikan `*.log` **tetapi** meng-include-ulang `Analysis/reports/*.log`, karena CM_PLAN §3 mewajibkan artefak bukti RUNTIME VERIFIED ter-track. Jadi pernyataan "raw serial logs strictly excluded" hanya berlaku untuk log di luar `Analysis/reports/` (mis. `logs/`, transcript sementara harness) serta artefak selangkah CI `ci_*.log.step*.log` yang sengaja tidak di-stage. Bukti yang ter-track dipilih eksplisit per-file saat commit, bukan lewat `git add -A`.
- **Status remote (PHASE-6A, 2026-10-20):** branch `security/asl-v1.0.2-audit` sudah ter-push ke `origin` (tip `8b829d8`, fast-forward dari `95bac29`, **tanpa force / tanpa rewrite riwayat** sesuai G8). Judgment "LOCAL-ONLY" masih berlaku untuk `tools/`, direktif agen, dan log harness yang tidak dipilih sebagai bukti — bukan untuk artefak `Analysis/reports/` yang memang di-track sebagai bukti. PR untuk kerja ini **belum dibuat/di-merge** dan **tidak ada rilis v1.1.0** (keputusan pemilik repo).
- Semua perubahan harus lolos `python tools/Sub_Assist.py --hygiene` sebelum commit, **dan** lolos pipeline CI minimal (§6) — pipeline merah = tidak boleh merge.
- The 24 Phase_01..06 gates in [`docs/ROADMAP/CHECKLIST_STATE.json`](file:///c:/asl/docs/ROADMAP/CHECKLIST_STATE.json) are permanently **PASS & LOCKED** and must not be altered.

---

## 8. LICENSE & COPYRIGHT

ASL Core Kernel is free software: you can redistribute it and/or modify it under the terms of the GNU General Public License as published by the Free Software Foundation, either version 3 of the License, or (at your option) any later version.

Copyright © 2026 qomaruddindjamal.
