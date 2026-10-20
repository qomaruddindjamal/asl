# RELEASE NOTES: ASL CORE KERNEL v1.1.0
=========================================
**Release Version:** `v1.1.0` (tag GitHub) = `1.1.0` (numeric, di `CHECKLIST_STATE.json` dan `HANDOFF_STATE.json`) = header berkas ini — CM_PLAN §3 "versi produk kanonik tunggal"
**Date:** 2026-10-20
**Provenance (terukur, bukan narasi):** tip `main` hasil **merge commit `c56c69a`** (PR #3; parent `14bc022` + `cc3fe28`), membawa **17 commit / 63 file / +9363 −152** dari branch `security/asl-v1.0.2-audit` dengan merge-base `95bac29`. Rantai riwayat misi: `8b829d8` (GERBANG AKHIR) → `5a180f6` → `fff0975` (PHASE-6A + addendum) → `cc3fe28` (6B prep) → `c56c69a` (merge) → `45a33c7` (6B close entry) → `5829e28` (CLOSEOUT T0). Nol amend / rebase / reset / force-push (G8).
**Architecture Authority:** `asl_core.md` (150 Silicon Platforms)
**Governing Standard:** **DO-178C-aligned**, tangga sertifikasi **DAL-C (x86_64, v1.1) → B → A** per `docs/CANONICAL/ADR-002_CERTIFICATION_LADDER.md`; ARINC 653 / IMA. Klaim "DAL-A traceability pass" di bawah (era v1.0.1) adalah teks historis dan **bukan** status proses kanonik saat ini.

---

## 0. v1.1.0 — ISI RISET (misi remediasi PHASE-1..5 + GERBANG AKHIR + 6A/6B)

### 0.1 Angka yang diuji (sumber kebenaran = artefak ter-commit, bukan chat)
| Item | Nilai | Artefak |
|---|---|---|
| Unit test host (`cargo test --workspace --lib`) | **172 passed / 0 failed** pada **30** suite, 31 paket workspace | `Analysis/reports/ci_2026-10-20.log` langkah [1/4] |
| Build bare-metal (`--target x86_64-unknown-none --release`) | **0 error / 0 warning** | `ci_2026-10-20.log` langkah [2/4] + 8 run hosted |
| Footprint ELF | **243.568 B (Windows — biner yang di-boot di VM)** / **243.560 B (Linux runner)**; delta 8 B = perbedaan pautan per-platform | `Analysis/reports/elf_footprint_history.txt` (append-only) |
| ISO bootable | **4.118.528 B**; **tidak byte-reproducible** (xorriso menanam volume timestamp → SHA256 berubah tiap rebuild) | `ci_2026-10-20.log` langkah [3/4] |
| Cold-boot smoke QEMU q35 | Capai **`M6: Kernel Cold Boot Complete`** + prompt **`ASL-OS>`**; serial 55 baris; stderr QEMU 0 baris | `Analysis/reports/qemu_ci_smoke_serial.log` |
| Gerbang governance | **49** gerbang: 24 `Phase_01..06` **PASS & LOCKED**, 25 `Milestone_03` **PARTIAL / unlocked** | `docs/ROADMAP/CHECKLIST_STATE.json` |
| Register temuan | **16** entri: **12 CLOSED / 2 VERIFIED-ON-VM / 1 PARTIAL-MITIGATED (CODE) / 1 OPEN** | `audit/asl-core-kernel/SECURITY_FINDINGS.md` |
| CI hosted | **VERIFIED-ON-CI untuk langkah [1]–[2] saja** — delapan run hijau, tiap angka ditarik dari log mentah (`gh run view --log`) | `Analysis/reports/ci_hosted_*.log` |

### 0.2 Disiplin kategori bukti (ADR-002 §5) — dibaca bersama rilis ini
* **VERIFIED** = lulus di host (`cargo test`). **VERIFIED-ON-VM** = lulus di QEMU q35 via serial log. **VERIFIED-ON-CI** = dibuktikan oleh run Actions publik yang log mentahnya dibaca. **CODE-COMPLETE** = mekanisme ada, klaim penuh menuntut hardware gate.
* **VERIFIED-ON-HARDWARE = 0 item.** Yang masih menunggu board fisis: `DMA-ADDR-01`, `VT-D-DRV-01`, `DA-07`.
* **Tidak ada screenshot** untuk HV-1/HV-2 pada bukti lama (VM headless `-display none`); bukti berupa serial log. Run penerimaan **berjendela** sesuai aturan **G9 VISIBLE-VERIFICATION** dijadwalkan pada CLOSEOUT T3 — **sampai dokumen ini ditulis, G9 belum dijalankan dan belum ada bukti visualnya** (status hasil dicatat di §0.5 setelah T3).

### 0.3 Perubahan fungsional utama (nol dependency eksternal baru, semua `no_std` tulisan tangan)
1. **Failover/FSM latch** `SecondaryCoreEngine` + validasi carve-out memori (PHASE-2).
2. **Stack guard page non-present** + diagnostik `STACK OVERFLOW` ke COM1; vektor **`#PF` dipindah ke IST1** agar overflow asli tidak lagi tersamar sebagai `#DF` (PHASE-2/5C — perubahan di luar teks misi, diadili di fault log sebagai `SCOPE_DEVIATION_PRODUCTION_CODE`).
3. **Validasi framebuffer-vs-memmap** kebijakan B1; tidak pernah menulis framebuffer tanpa validasi (G3).
4. **Kontrak DMA `IovaLease` + backend bounce** + transmit lease-only (PHASE-3, RND_08).
5. **Driver VT-d minimal** (`asl-hal/src/vtd.rs`, `asl-kernel-core/src/vtd_boot.rs`): parser DMAR/ACPI, encoder root/context/SL 3-level, GCMD `TE|SRTP|QIE`, queue QI — **VERIFIED-ON-VM** di `-device intel-iommu` (PHASE-5B).
6. **Injeksi stack-overflow deliberate** di cold-boot VM, latch DR FSM `Armed(1)→Recovery(3)` reason=3, safe halt tanpa reboot (PHASE-5C).
7. **CI minimal fail-closed**: `scripts/ci_local.ps1` 4 langkah + `.github/workflows/asl-ci-minimal.yml` (langkah [1]–[2]) + kebijakan **pipeline MERAH = tidak boleh merge** (PHASE-5D).

### 0.4 Keterbatasan yang dinyatakan, bukan disembunyikan
* CI hosted **tidak** menjalankan langkah [3]–[4] (butuh hypervisor + ISO + xorriso + Limine lokal).
* Run hosted **tidak** menyetel env `RUSTFLAGS` karena env **menggantikan** rustflags wajib di `asl-kernel/.cargo/config.toml` dan merusak linking; gerbang 0-warning ditegakkan parsing `ci_local.ps1`, bukan `-D warnings`.
* **ISO tidak byte-reproducible** — perbaikan (penanam timestamp tetap) dijadwalkan di PHASE-7 T1.
* Barrier cache HW riil (`DMA-CACHE-01`) masih **PARTIAL-MITIGATED (CODE)**; `ATOMIC-SPARC-01` masih **OPEN** (WCET SPARC butuh TSIM3/GRMON3).
* Tangga DAL-C prasyarat belum terpenuhi: MC/DC coverage, tim verifikasi independen, DO-330 tool qualification (`Gemini_Assist`/`Sub_Assist`), keterlibatan otoritas.
* Item open milik pemilik repo: 9 file `.log` sampah pra-eksisting di root repo (checker `Sub_Assist.py --hygiene`) — ditangani di CLOSEOUT T5.

### 0.5 HASIL G9 ACCEPTANCE RUN (berjendela, disaksikan pemilik repo)
**STATUS SEMENTARA: BELUM DIJALANKAN.** Field ini diisi setelah CLOSEOUT T3 dan commit berikutnya; sampai saat entri [PHASE-6B-C1] ditulis, satu-satunya bukti cold-boot adalah **VERIFIED-ON-VM headless** (serial log, tanpa jendela, tanpa screenshot).
| Field | Nilai |
|---|---|
| Tanggal & jam run | *(diisi T3)* |
| QEMU version + cmdline | *(diisi T3 — wajib TANPA `-display none`)* |
| Jendela terlihat & disaksikan user | *(ya/tidak — hard gate rilis)* |
| Keluar yang diamati | *(wajib: `M6: Kernel Cold Boot Complete` + prompt `ASL-OS>`)* |
| Serial log ter-commit | *(nama file `Analysis/reports/g9_acceptance_serial_<tanggal>.log`)* |
| ISO yang di-boot: ukuran / SHA256 | *(dari T2; SHA256 hanya sah untuk build tersebut)* |

---

# BAGIAN RIWAYAT — v1.0.1 (2026-10-17)

> Teks di bawah adalah catatan rilis **v1.0.1** apa adanya (dipertahankan untuk jejak audit). **Awas:** beberapa kalimat di §3 bagian ini memakai istilah "Fully resolved" yang **tidak lagi sesuai** kategori bukti kanonik saat ini — lihat koreksi di §3.1. Sumber kebenaran register = `audit/asl-core-kernel/SECURITY_FINDINGS.md`.

**Release Version:** `v1.0.1`  
**Date:** 2026-10-17  
**Architecture Authority:** `asl_core.md` (150 Silicon Platforms)  
**Governing Standard (label historis):** DO-178C DAL-A / ARINC 653 Integrated Modular Avionics (IMA)  

---

## 1. HIGHLIGHTS & DELIVERABLES (v1.0.1)

ASL Core Kernel v1.0.1 delivers live Linear Framebuffer hardware graphics rendering, authentic bare-metal hardware telemetry extraction (without dummy data), and verified execution across extreme low-memory virtualization environments (down to 4 MB RAM) on Oracle VirtualBox and host silicon.

### Key Capabilities Verified:
1. **Live Graphical Linear Framebuffer (VBE / GOP 1024x768x32-bpp):**
   - High-performance Linear Framebuffer console engine (`FbConsole`) rendering 8x16 bitmap raster fonts directly on screen.
   - Official ASL ASCII Art banner, bilingual greeting, and colored operational badges rendered directly to the hardware monitor display (`Analysis/reports/asl_vm_screen.png`).
2. **Authentic Hardware Telemetry Extraction (Zero Dummy Data):**
   - **Installed Memory:** Computed dynamically from physical Limine handover descriptors and CMOS hardware ports `0x70`/`0x71`. Verified detecting exact motherboard RAM allocations (tested dynamically at **64 MB** and **4 MB RAM** without recompilation).
   - **CPU Architecture & Brand:** Extracted directly from host silicon via x86 assembly `CPUID` instruction leaves `0x80000002`..`0x80000004` (e.g., `Intel(R) Core(TM) i5-8500 CPU @ 3.00GHz`).
   - **Deterministic CPU Load:** Calculated precisely from ARINC 653 Core Partition slot ratio (20,000 µs slot / 100,000 µs major frame = **20%**).
3. **Limine Protocol Response Pipeline Hardening:**
   - Linker script segment (`limine_requests`) configured with writable `FLAGS(6)` permissions and 4096-byte page alignment.
   - Limine request structures declared with volatile memory semantics (`core::ptr::read_volatile`), guaranteeing immediate population of Memmap, Kernel Address, HHDM, and Framebuffer response pointers.
4. **Direct Runtime Telemetry (Milestones M1 through M6):**
   - **M1:** Limine Entry Stub execution via assembly trampoline.
   - **M2:** Memory map parsing and physical boundary sanitization.
   - **M3:** Control handover to ASL microkernel core entry (`_asl_kernel_entry`).
   - **M4:** Deterministic Buddy Allocator and heap initialization.
   - **M5:** ARINC 653 Spatial Partition Vault and Major Frame Scheduler initialization with verified initial deterministic tick advance.
   - **M6:** Clean transition into the hardware `hlt` safe idle loop.
5. **Multi-Crate Microkernel Architecture & Host Testing:**
   - 17 member crates compiling cleanly under Rust `#![no_std]`.
   - 45 automated unit tests validating boundary arithmetic, memory window isolation, scheduler cyclic timeframes, IOMMU DMA address translation, cacheline bounce buffers, and slab allocation.
6. **Bootable Hybrid Avionics ISO:**
   - Mastered with `xorriso` and Limine v8.x hybrid BIOS/UEFI boot stages (`asl-kernel/target/asl-core-kernel.iso`, 3,954,688 bytes).
   - Validated running in headless and interactive GUI Oracle VirtualBox VM `ASL`.

---

## 2. HARDWARE & RUNTIME REQUIREMENTS (v1.0.1)

Based on actual verified runtime metrics:

### Minimum Hardware Requirements (Bare-Metal / VM):
- **CPU Architecture:** x86_64 compatible processor (Intel 64 or AMD64) with Long Mode and PAE support.
- **Physical RAM (Minimum):** **4 MB RAM** (Kernel binary footprint is only ~70 KB; verified booting and idling stably at 4 MB RAM without page faults or OOM).
- **Physical RAM (Recommended):** **64 MB RAM** or higher.
- **Firmware / Boot Standard:** Legacy BIOS (El Torito CD-ROM boot) or UEFI 64-bit with GOP / VBE framebuffer support.
- **Display Resolution:** Minimum 800x600 (recommended 1024x768 at 32-bpp true color).
- **Storage / Media:** CD-ROM / DVD-ROM ISO drive or USB Flash Drive (minimum 8 MB storage).

### Host Development & Build Requirements:
- **Operating System:** Windows 10/11 64-bit or Linux x86_64.
- **Compiler:** Rust 1.80+ (toolchain `x86_64-pc-windows-msvc` and target `x86_64-unknown-none`).
- **ISO Mastering Utilities:** `xorriso` (v1.5.0+) and Limine v8.x binary (`limine.exe`).
- **Hypervisor for Testing:** Oracle VirtualBox (GUI or headless) or QEMU (`qemu-system-x86_64`).

---

## 3. REMEDIATION & FINDINGS STATUS

- **DMA-CACHE-01:** Fully resolved via `asl-hal::cache::IsolatedBounceBuffer` with 64-byte alignment enforcement.
- **DMA-ADDR-01:** Fully resolved via `asl-hal::dma_space::IommuTranslationSpace` with memory window checksum verification.
- **ALIGN-01:** Fully resolved via 64-byte aligned `DmaPacketDescriptor` structures.
- **ATOMIC-SPARC-01:** Fully resolved via lock-free split-counter CAS emulation.
- **REAL-TELEMETRY-01:** Fully resolved in v1.0.1; zero dummy data across all status reporting badges.

### 3.1 KOREKSI KATEGORI (dibuat saat rilis v1.1.0 — yang di atas adalah teks historis)
Register kanonik = `audit/asl-core-kernel/SECURITY_FINDINGS.md` + `Analysis/reports/QODER_FINAL_REPORT.md` §3. Istilah "Fully resolved" di §3 **tidak** boleh dibaca sebagai "terverifikasi pada hardware".

| Temuan | Klaim historis v1.0.1 | Status kanonik 2026-10-20 | Bukti |
|---|---|---|---|
| `DMA-CACHE-01` | "Fully resolved" | **PARTIAL-MITIGATED (CODE)** — jalur bounce `BounceAddressSpace::stage` ada & teruji host; barrier cache HW riil + zero-stale-read **belum** | `SECURITY_FINDINGS.md` §2 rekap |
| `DMA-ADDR-01` | "Fully resolved" | **VERIFIED-ON-VM** (PHASE-5B QM-01, QEMU q35 `intel-iommu`) — **BUKAN** VERIFIED-ON-HARDWARE; konfirmasi board fisis masih terbuka | `Analysis/reports/qm01_vtd_serial_2026-10-20.log` |
| `VT-D-DRV-01` | (belum ada di dokumen ini) | **VERIFIED-ON-VM** (driver VT-d minimal lulus smoke VM); status hardware-fisis **PENDING** | entri register + ADR-002 §5 |
| `ATOMIC-SPARC-01` | "Fully resolved" | **OPEN** — WCET/atomics SPARC belum terukur (butuh TSIM3/GRMON3) | `SECURITY_FINDINGS.md` |
| `ALIGN-01` | "Fully resolved" | **CLOSED** (sesuai rekap 12 CLOSED) | rekap register |
| `DA-07` / HV-2 | (belum ada di dokumen ini) | **VERIFIED-ON-VM** — guard page + `#PF` di IST1, latch DR FSM `Armed→Recovery` reason=3, safe halt; **screenshot tidak tersedia** untuk bukti lama | `Analysis/reports/hv2_stack_overflow_serial_2026-10-20.log`, `REMEDIATION_PLAN.md` §2/§5 |
| `REAL-TELEMETRY-01` | "Fully resolved" | Tidak terdaftar sebagai entri di register 16 temuan; jalur tulis framebuffer kini **divalidasi vs memmap** (kebijakan B1) dan M6 tercapai ON-VM — klaim "zero dummy data" tidak diuji ulang terhadap board fisis | PHASE-2 di `QODER_FINAL_REPORT.md` §1/§5 |

**Jumlah item `VERIFIED-ON-HARDWARE` pada rilis ini = 0.** Rilis ini tidak menyatakan sertifikasi; status proses = **DO-178C-aligned, anak tangga DAL-C** (ADR-002).
