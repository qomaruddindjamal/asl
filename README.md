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
| **G9 visible acceptance run (PHASE-6B-CLOSEOUT T3)** | QEMU q35 dengan **jendela terbuka**, cmdline **tanpa** `-display none`, disaksikan pemilik repo | **VERIFIED-ON-VM (2026-10-20 20:48 +0700) — BUKAN VERIFIED-ON-HARDWARE** | `Analysis/reports/g9_acceptance_serial_2026-10-20.log`: **`M6: Kernel Cold Boot Complete`** di baris 32 dan prompt **`ASL-OS>`** di baris 55; checker harness `scripts/g9_acceptance_run.ps1` melaporkan `M6=True shell=True crash=False`, 0 pola exception/panic/FATAL, stderr QEMU 0 baris saat checker selesai (1 baris assertion backend GTK QEMU menyusul pada 20:51:46, bukan keluaran kernel — berkas stderr ikut ter-commit); metadata run di `g9_acceptance_summary_2026-10-20.log`. ISO yang di-boot = build tip `bb4a9df`, 4.118.528 B, SHA256 `FACA92F4E1E92B2EE5EB8D2E931C0B1A64DF9C2035A308529E69D4E5128C8355` (sah hanya untuk build ini). Catatan jujur: banner console masih `ASL Kernel v1.0.2` (string hardcoded `asl-kernel-core/src/lib.rs:1126`) sedangkan identitas rilis produk `v1.1.0` — lihat RELEASE_NOTES §0.4 |
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
- **Bukti hosted (PHASE-6A/6B/CLOSEOUT + PHASE-7 T0, terukur ulang 2026-10-21):** workflow sudah benar-benar berjalan di runner GitHub **empat belas kali hijau** pada rantai `8b829d8` -> `6623dd8` di `main` (`push` @ `8b829d8`, `5a180f6`, `fff0975`, `cc3fe28`; `pull_request` PR #3 @ `cc3fe28`; `push` @ `c56c69a` = merge commit di `main`, `45a33c7`, `5829e28`, `bb4a9df`, `e78b186` = CLOSEOUT T3/G9, `7104814` = commit bertag `v1.1.0`, `08f706a`, `3c716d4`, `6623dd8`), dengan **tiga tingkat bukti yang dibedakan tegas**:
  1. **8 run** diverifikasi dari **log mentah** (`gh run view --log`) **DAN** ekstrak-nya **ter-commit di repo** — `39399083819`, `39400350619`, `39417746328`, `39434189599`, `39435093922` @ `7104814` bertag, `39437444616`, `39444157361`, `39486017207` @ `7e2ac24` (run atas branch `feature/phase7`; yang terakhir ini bukti pertama yang masuk repo lewat jalur PR) (berkas `Analysis/reports/ci_hosted_<run>_2026-10-20.log`); angka dihitung ulang dari teks: 30 suite **172 passed / 0 failed**, langkah release 13 baris `Compiling` + `Finished` dengan **0 `warning:` / 0 `error:`**, ELF Linux **243.560 B**. Ini satu-satunya tingkat yang bisa diaudit pihak ketiga langsung dari repo.
  2. **5 run** (`39417162899`, `39417504070` = `pull_request` PR #3, `39418646354`, `39422495702`, `39423423307`) log mentahnya **ditarik dan diukur pada sesinya sendiri**, tetapi ekstrak ter-commit tidak pernah dibuat; berkas mentahnya hanya hidup di `logs/` yang di-ignore, jadi tingkat ini **tidak dapat diaudit dari repo**. Pemetaan berkas->run (kolom langkah + stempel waktu UTC vs `createdAt`), SHA256 tiap berkas, dan angka hasil ukur ulang hari ini (identik: 172/0, release 0/0, ELF 243.560 B) tercatat di `Analysis/reports/phase7_t0_protection_2026-10-21.md` §5.1.
  3. **2 run** (`39400212750` @ `5a180f6`, `39436250243` @ `08f706a`) **hanya** dari **metadata API** `conclusion=success` + `headSha`, tanpa penarikan log.
  **Dua koreksi append-only atas pernyataan ini:** (a) teks versi pra-T6 menulis bahwa angka *tiap* run dibaca dari log mentah — terlalu kuat; (b) `6623dd8` lalu menggolongkan **7** run sebagai "hanya metadata API" — terlalu lemah untuk **5** di antaranya, terbukti dari berkas mentah yang masih ada di `logs/` dengan mtime 19 detik dan 60 detik setelah run masing-masing dibuat. Kedua koreksi dilakukan di commit `PHASE-7-0` ini lewat dokumen baru, bukan dengan menulis-ulang riwayat commit (G8), dan diratifikasi sebagai entri fault log #14. Karena tree yang diuji docs-only (nol `.rs` sejak `cc3fe28`), angka test/build/footprint identik di semua run yang teksnya terukur — tetapi ketiga tingkat **tidak boleh dibaca setara**.
  **Total termasuk branch fitur (PHASE-7):** **15** run `asl-ci-minimal` hijau = 14 di `main` + 1 di `feature/phase7` (`39486017207` @ `7e2ac24`; ekstrak ter-commit `Analysis/reports/ci_hosted_39486017207_2026-10-20.log`: 30 suite 172/0, 13 `Compiling`, `Finished` 2.45s, release 0 warning/0 error, ELF Linux 243.560 B). Run branch fitur itu adalah bukti pertama yang masuk repo lewat **PR**, jadi ia sekaligus menguji pipeline hasil Opsi B.
- **Delta platform yang dicatat jujur:** artefak ELF yang sama menghasilkan **243.560 B di `ubuntu-latest`** vs **243.568 B di Windows** (selisih 8 B karena pautan/identitas biner per-platform). Angka yang dikutip §6 tetap **243.568 B** karena itulah biner yang di-boot di VM lokal; riwayat lengkapnya di `Analysis/reports/elf_footprint_history.txt`.
- Catatan kebenaran artefak: build ISO **tidak byte-reproducible** (xorriso menanam volume timestamp), jadi ukuran ISO tetap 4.118.528 B tetapi SHA256 berubah tiap rebuild — SHA256 di log CI hanya sah untuk run tersebut. **[PHASE-7 T1, 2026-10-21 — status kini:]** kalimat di atas dipertahankan sebagai keadaan **pra-T1**; sejak T1 build ISO **reproducible per-platform** di host Windows ini: dua build bersih menghasilkan SHA256 identik `9cd49028d8b1b9b70d9316c9410a2902f20673cbb6c0ca5836dfe5808cc8e8f2` (`cmp` = 0 byte berbeda). Dua akar nondeterminisme yang ditutup: stempel waktu volume/mtime xorriso (`--modification-date` + `--set_all_file_dates`, dari `SOURCE_DATE_EPOCH` atau konstanta baked `1767225600`) dan **Disk GUID acak yang ditulis `limine bios-install`** di struktur GPT (GUID ditentukan ulang dari epoch + `HeaderCRC32`/`PartitionEntryArrayCRC32` dihitung ulang). Delta antar-platform (ELF 8 B: 243.568 Windows vs 243.560 Linux) tetap **didokumentasikan dan bukan target**; pasangan pengukuran Linux→Linux belum ada karena CI hosted tidak menjalankan langkah ISO. Bukti: `Analysis/reports/phase7_t1_iso_reproducible_2026-10-21.md` + `Analysis/reports/iso_reproducible_t1_2026-10-21.log`.

---

## 7. REPOSITORY INTEGRITY & LOCAL-ONLY GOVERNANCE

To protect local research environments and prevent intellectual property or governance leaks:
- The governance scripts (`tools/`), project directives (`AGENT.md`, `GEMINI.md`), and raw serial logs (`*.log`) are excluded from remote tracking via [`.gitignore`](file:///c:/asl/.gitignore).
- **Koreksi nuansa (PHASE-5D):** `.gitignore` mengecualikan `*.log` **tetapi** meng-include-ulang `Analysis/reports/*.log`, karena CM_PLAN §3 mewajibkan artefak bukti RUNTIME VERIFIED ter-track. Jadi pernyataan "raw serial logs strictly excluded" hanya berlaku untuk log di luar `Analysis/reports/` (mis. `logs/`, transcript sementara harness) serta artefak selangkah CI `ci_*.log.step*.log` yang sengaja tidak di-stage. Bukti yang ter-track dipilih eksplisit per-file saat commit, bukan lewat `git add -A`.
- **Status PHASE-7 T1 (2026-10-21, branch `feature/phase7`, satu berkas build script — nol `.rs`):** AUD-03 ditutup untuk jalur kanonik `scripts/build_kernel.ps1 -Action iso` (dipakai `ci_local.ps1` langkah [3/4]). Kriteria exit terukur: **dua build bersih pada host+platform yang sama → SHA256 ISO identik** (`9cd49028…`, `cmp` 0 byte berbeda, ukuran tetap 4.118.528 B). Penemuan di luar teks rencana: pin tanggal xorriso saja **tidak cukup** — `limine 8.7.0 bios-install` menulis Disk GUID acak di header GPT primer+cadangan (terukur: dua build ber-pin-tanggal tetap berbeda **144 byte pada 12 run**, seluruhnya di struktur GPT), sehingga build kini menetapkan GUID dari epoch dan menghitung ulang kedua CRC32 GPT; jika header `EFI PART` tidak ditemukan build STOP (fail-closed). Image ter-pin diverifikasi **boot** di QEMU 10.2.50 (`M6=True`, prompt shell tercapai, tanpa crash, serial 55 baris) — kategorinya **VERIFIED-ON-VM smoke**, bukan run penerimaan G9 (jadwalnya T4) dan bukan VERIFIED-ON-HARDWARE. Gerbang G4 diulang pada tree ini: 30 suite **172 passed / 0 failed**, release `x86_64-unknown-none` **0 warning / 0 error** (13 baris `Compiling` pada build dari-nol). Yang **tidak** diubah: `scripts/test_hv2_stack_overflow_qemu.ps1` (jalur ISO non-kanonik untuk uji negatif HV-2) masih non-reproducible, dan reproduksibilitas belum diukur pada host Linux. Bukti: `Analysis/reports/phase7_t1_iso_reproducible_2026-10-21.md`, `Analysis/reports/iso_reproducible_t1_2026-10-21.log`, ratifikasi `ACCOUNTABILITY_FAULT_LOG.json` entri #16. `CHECKLIST_STATE.json` tidak disentuh.
- **Status PHASE-7 T0 (2026-10-21, branch `feature/phase7`):** pekerjaan fase ini **wajib lewat PR** — **yang terukur adalah konfigurasi proteksi** (required check `test-and-build` strict + `enforce_admins=true` + PR wajib, diverifikasi field-per-field lewat GET balik); kalimat "direct push ke `main` ditolak GitHub" adalah **konsekuensi yang disimpulkan dari konfigurasi itu, bukan hasil push uji** (tidak ada push uji ke `main`, sengaja tidak dilakukan), lihat kualifikasi append-only `Analysis/reports/phase7_t0_protection_2026-10-21.md` §3.1 — dan agen tidak melonggarkan proteksi (yang dilonggarkan hanyalah syarat approval akun kedua, atas keputusan pemilik). Realisasi T0: proteksi Opsi B dipasang + diverifikasi GET (lihat koreksi di entri CLOSEOUT di atas), ekstrak log mentah run `39444157361` masuk repo lewat PR karena jalur direct push sudah tertutup, hitungan run hosted direkonsiliasi ke **14** dengan **tiga** tingkat bukti (butiran di §6), dan 13 berkas `.log` scratch tak-ter-track di `Analysis/reports/` diklasifikasi dengan bukti byte/line: 9 dihapus (dupleks atau murni bungkus PowerShell), 4 disahkan sebagai baseline PHASE-7 — termasuk inventaris nama test pra-T3 (`ci_2026-10-20.log.step1.log`) dan transkrip xorriso pra-T1 (`ci_2026-10-20.log.step3_iso.log`). Bukti: `Analysis/reports/phase7_t0_protection_2026-10-21.md`, `Analysis/reports/phase7_t0_hygiene_2026-10-21.md`. Nol berkas `.rs` berubah di T0; `CHECKLIST_STATE.json` tidak disentuh.
- **Status remote (PHASE-6A, 2026-10-20):** branch `security/asl-v1.0.2-audit` sudah ter-push ke `origin` (rantai fast-forward `95bac29` → `8b829d8` → `5a180f6` → `fff0975` → `cc3fe28`, **tanpa force / tanpa rewrite riwayat** sesuai G8). Judgment "LOCAL-ONLY" masih berlaku untuk `tools/`, direktif agen, dan log harness yang tidak dipilih sebagai bukti — bukan untuk artefak `Analysis/reports/` yang memang di-track sebagai bukti.
- **Status merge (PHASE-6B, 2026-10-20 — koreksi atas kalimat "PR belum dibuat/di-merge" pada entri 6A di atas):** **PR #3 sudah di-merge** ke `main` dengan **merge commit `c56c69a`** (`gh pr merge --merge`; squash/rebase dilarang karena akan menulis-ulang rantai 17 commit misi; branch sumber **tidak** dihapus). CI hosted di tip `main` pasca-merge = **success** dan diverifikasi dari log mentah (run `39417746328`).
- **Status CLOSEOUT (PHASE-6B-CLOSEOUT T0–T6, 2026-10-20 — penutup; menggantikan daftar “Yang masih belum ada” di entri 6B dan T0–T5 di atas):** sudah selesai: merge `main` (`c56c69a`), koreksi append-only `45a33c7`→`5829e28`, sinkron identitas tiga berkas ke `v1.1.0` + penutupan resmi `QODER_EXECUTION_LOG` untuk PHASE-1..6B (`bb4a9df`), build ISO final di tip `bb4a9df`, **run penerimaan berjendela G9 = PASS** dengan bukti ter-commit (`e78b186` + koreksi presisi `7104814`), **annotated tag `v1.1.0`** (`90954ce` → peel `7104814`) dan **rilis GitHub non-pre-release** https://github.com/qomaruddindjamal/asl/releases/tag/v1.1.0 dengan ISO yang persis di-boot G9 + `SHA256SUMS.txt` + log serial/summary G9 + log build ISO, **pembersihan higiene root (T5)** (`--hygiene` = `[PASS (V)] ROOT WORKSPACE BERSIH 100%`; 9 `.log` root → 0, 4 disahkan ke `Analysis/reports/` dengan nama asli, 5 dihapus sebagai dupleks byte-identik terbukti `cmp`; manifest di `hygiene_root_logs_2026-10-20.log`). Langkah **T6 (branch protection `main`)** = pintu terakhir yang **dieksekusi segera setelah commit ini ter-push dan CI hijau**: config atas keputusan pemilik “required check + PR + 1 approval” (check `test-and-build` wajib hijau, merge wajib PR dengan 1 review approval, direct push ditolak termasuk untuk admin, force-push dan penghapusan branch diblokir); payload + keadaan awal (`404 Branch not protected`) sudah tercatat di `Analysis/reports/branch_protection_main_2026-10-20.log`. **KOREKSI APPEND-ONLY (PHASE-7 T0, 2026-10-21):** syarat *1 review approval* di atas sudah **tidak berlaku lagi**. Karena repo hanya punya satu akun GitHub (`qomaruddindjamal` = penulis = owner), aturan itu secara struktural memblokir seluruh perubahan `main` berikutnya (penulis tidak bisa approve PR sendiri, dan `enforce_admins=true` menutup pintas admin — deadlock ini memang dinyatakan sebelum pemasangan, lihat §7 butir 8 `QODER_FINAL_REPORT.md`). Pemilik repo memutuskan **Opsi B**: `required_approving_review_count=0` dan `dismiss_stale_reviews=false`, sedangkan **PR tetap wajib**, status check `test-and-build` **tetap wajib hijau** (strict), anti force-push, anti penghapusan branch dan `enforce_admins=true` **tetap** berlaku — diverifikasi field-per-field lewat GET balik di `Analysis/reports/phase7_t0_protection_2026-10-21.md` §2–§3 dan diratifikasi sebagai entri fault log #13. **Catatan kejujuran waktu:** berkas ini di-commit SEBELUM proteksi dipasang — itu memang satu-satunya urutan yang mungkin, karena setelah proteksi aktif dokumen bukti tidak bisa lagi masuk `main` tanpa PR; **bukti GET pasca-pasang** karena itu disampaikan di laporan chat T7. **Konsekuensi yang dinyatakan:** repo hanya punya satu akun GitHub, sehingga penulis PR tidak dapat menyetujui PR sendiri — commit berikutnya ke `main` menuntut reviewer kedua atau penyesuaian aturan oleh pemilik; agen tidak akan melonggarkan proteksi. **PHASE-7 / T0 `RND_09`: gerbang prasyarat CLOSEOUT kini terpenuhi, tetapi pekerjaannya tetap STOP sampai pemilik repo memberi otorisasi terpisah.**
- Semua perubahan harus lolos `python tools/Sub_Assist.py --hygiene` sebelum commit, **dan** lolos pipeline CI minimal (§6) — pipeline merah = tidak boleh merge.
- The 24 Phase_01..06 gates in [`docs/ROADMAP/CHECKLIST_STATE.json`](file:///c:/asl/docs/ROADMAP/CHECKLIST_STATE.json) are permanently **PASS & LOCKED** and must not be altered.

---

## 8. LICENSE & COPYRIGHT

ASL Core Kernel is free software: you can redistribute it and/or modify it under the terms of the GNU General Public License as published by the Free Software Foundation, either version 3 of the License, or (at your option) any later version.

Copyright © 2026 qomaruddindjamal.
