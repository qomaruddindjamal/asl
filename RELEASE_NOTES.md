# RELEASE NOTES: ASL CORE KERNEL v1.1.0
=========================================
**Release Version:** `v1.1.0` (tag GitHub) = `1.1.0` (numeric, di `CHECKLIST_STATE.json` dan `HANDOFF_STATE.json`) = header berkas ini — CM_PLAN §3 "versi produk kanonik tunggal"
**Date:** 2026-10-20
**Rilis GitHub (CLOSEOUT T4, 2026-10-20 14:02:13Z):** https://github.com/qomaruddindjamal/asl/releases/tag/v1.1.0 — annotated tag `v1.1.0` = tag object `90954ce008eb1e4774e792357ddcf528bc7f41a3` → peel commit `7104814e2ea1b3c45ff4b69c0fe70ed6d8c0423e`; `draft=false`, `prerelease=false`, `target_commitish=main`; 5 aset termasuk `asl-core-kernel-v1.1.0.iso` 4.118.528 B ber-digest `sha256:faca92f4e1e92b2ee5eb8d2e931c0b1a64df9c2035a308529e69d4e5128c8355` = **biner persis yang di-boot pada run G9** (bukan rebuild). Bukti: `Analysis/reports/release_v1.1.0_evidence_2026-10-20.log` + `Analysis/reports/release_v1.1.0_SHA256SUMS_2026-10-20.txt`.
**Provenance (terukur, bukan narasi):** tip `main` hasil **merge commit `c56c69a`** (PR #3; parent `14bc022` + `cc3fe28`), membawa **17 commit / 63 file / +9363 −152** dari branch `security/asl-v1.0.2-audit` dengan merge-base `95bac29`. Rantai riwayat misi: `8b829d8` (GERBANG AKHIR) → `5a180f6` → `fff0975` (PHASE-6A + addendum) → `cc3fe28` (6B prep) → `c56c69a` (merge) → `45a33c7` (6B close entry) → `5829e28` (CLOSEOUT T0) → `bb4a9df` (CLOSEOUT T1, basis build ISO T2) → `e78b186` (CLOSEOUT T3 = bukti G9 berjendela) → `7104814` (koreksi presisi waktu bukti T3 + ekstrak CI `39434189599`; **tip yang di-tag `v1.1.0`**). Nol amend / rebase / reset / force-push (G8).
**Architecture Authority:** `asl_core.md` (150 Silicon Platforms)
**Governing Standard:** **DO-178C-aligned**, tangga sertifikasi **DAL-C (x86_64, v1.1) → B → A** per `docs/CANONICAL/ADR-002_CERTIFICATION_LADDER.md`; ARINC 653 / IMA. Klaim "DAL-A traceability pass" di bawah (era v1.0.1) adalah teks historis dan **bukan** status proses kanonik saat ini.

---

## 0. v1.1.0 — ISI RISET (misi remediasi PHASE-1..5 + GERBANG AKHIR + 6A/6B)

### 0.1 Angka yang diuji (sumber kebenaran = artefak ter-commit, bukan chat)
| Item | Nilai | Artefak |
|---|---|---|
| Unit test host (`cargo test --workspace --lib`) | **172 passed / 0 failed** pada **30** suite, 31 paket workspace | `Analysis/reports/ci_2026-10-20.log` langkah [1/4] |
| Build bare-metal (`--target x86_64-unknown-none --release`) | **0 error / 0 warning** | `ci_2026-10-20.log` langkah [2/4] + 15 run hosted (8 dengan ekstrak log mentah ter-commit; rinci di baris CI hosted di bawah) **[PHASE-7-2, 2026-10-21: angka baris ini keadaan pra-T1b; terukur kini 25 run hosted hijau dengan 18 ekstrak ter-commit]** |
| Footprint ELF | **243.568 B (Windows — biner yang di-boot di VM)** / **243.560 B (Linux runner)**; delta 8 B = perbedaan pautan per-platform | `Analysis/reports/elf_footprint_history.txt` (append-only) |
| ISO bootable | **4.118.528 B**; **tidak byte-reproducible** (xorriso menanam volume timestamp → SHA256 berubah tiap rebuild). Build final yang di-boot G9 di tip `bb4a9df`: SHA256 `FACA92F4E1E92B2EE5EB8D2E931C0B1A64DF9C2035A308529E69D4E5128C8355` **[KOREKSI APPEND-ONLY PHASE-7 T1, 2026-10-21: sejak T1 build ISO kanonik reproducible per-platform — dua build bersih di host Windows menghasilkan SHA256 identik `9CD49028D8B1B9B70D9316C9410A2902F20673CBB6C0CA5836DFE5808CC8E8F2`, `cmp` 0 byte berbeda; kalimat "tidak byte-reproducible" di atas tetap berlaku untuk build pra-T1 termasuk ISO rilis ini]** | `Analysis/reports/iso_build_v1.1.0_2026-10-20.log` + `ci_2026-10-20.log` langkah [3/4] + `Analysis/reports/iso_reproducible_t1_2026-10-21.log` |
| Cold-boot smoke QEMU q35 | Capai **`M6: Kernel Cold Boot Complete`** + prompt **`ASL-OS>`**; serial 55 baris; stderr QEMU 0 baris | `Analysis/reports/qemu_ci_smoke_serial.log` |
| **G9 acceptance run (berjendela)** | **PASS** — 2026-10-20 20:48:23 (+0700), QEMU 10.2.50, cmdline **tanpa** `-display none`, jendela disaksikan; `M6=True shell=True crash=False`; serial 55 baris; stderr 0 baris saat checker selesai, +1 baris assertion GTK QEMU pada 20:51:46 (bukan keluaran kernel) | `Analysis/reports/g9_acceptance_serial_2026-10-20.log`, `g9_acceptance_summary_2026-10-20.log`, `g9_acceptance_qemu_stderr_2026-10-20.log` |
| Gerbang governance | **49** gerbang: 24 `Phase_01..06` **PASS & LOCKED**, 25 `Milestone_03` **PARTIAL / unlocked** | `docs/ROADMAP/CHECKLIST_STATE.json` |
| Register temuan | **16** entri: **12 CLOSED / 2 VERIFIED-ON-VM / 1 PARTIAL-MITIGATED (CODE) / 1 OPEN** | `audit/asl-core-kernel/SECURITY_FINDINGS.md` |
| CI hosted | **VERIFIED-ON-CI untuk langkah [1]–[2] saja** — **14** run `asl-ci-minimal` hijau di rantai `8b829d8` -> `6623dd8` di `main` + **1** run branch `feature/phase7` (`39486017207` @ `7e2ac24`) = **15 total** (terukur ulang 2026-10-21 dengan `gh run list --workflow asl-ci-minimal --json databaseId,headSha,conclusion,event`), dengan **tiga tingkat bukti yang dibedakan tegas**: **(1) 8 run** diverifikasi dari **log mentah** (`gh run view --log`) **dan** diekstrak-ter-commit ke repo (`39399083819` @ `8b829d8`, `39400350619` @ `fff0975`, `39417746328` @ `c56c69a`, `39434189599` @ `e78b186`, `39435093922` @ `7104814` bertag, `39437444616` @ `3c716d4`, `39444157361` @ `6623dd8`, `39486017207` @ `7e2ac24` (run branch fitur, pertama lewat jalur PR)) — angka dihitung ulang dari teks: 30 suite 172/0, release 0 warning/0 error, ELF 243.560 B; **(2) 5 run** (`39417162899`, `39417504070` pull_request PR #3, `39418646354`, `39422495702`, `39423423307`) log mentahnya **ditarik dan diukur pada sesinya**, tetapi tidak ada ekstrak ter-commit dan berkas mentahnya hanya ada di `logs/` (di-ignore) → **tidak dapat diaudit dari repo**; **(3) 2 run** (`39400212750`, `39436250243`) **hanya** metadata API `conclusion=success` + `headSha`. Karena tree docs-only (nol `.rs` sejak `cc3fe28`) angka test/build/footprint identik di semua run yang teksnya terukur, tetapi ketiga tingkat **tidak boleh dibaca setara**. **Koreksi append-only (dua arah):** baris versi pra-T6 menulis “tiap angka ditarik dari log mentah” untuk 11 run — terlalu kuat; `6623dd8` lalu menggolongkan 7 run sebagai “hanya metadata API” — terlalu lemah untuk 5 di antaranya, terbukti dari berkas mentah `logs/` yang masih ada dan diukur ulang hari ini (SHA256 + pemetaan di `Analysis/reports/phase7_t0_protection_2026-10-21.md` §5.1). Tidak ada riwayat yang ditulis-ulang (G8); diratifikasi sebagai fault log #14. **[PHASE-7-1b, 2026-10-21 — status kini:]** terukur ulang lewat `gh run list --workflow asl-ci-minimal --limit 60`: **21** run hijau = 14 pada rantai `main` + 7 pada `feature/phase7`, tingkat bukti **14 / 5 / 2** karena enam ekstrak ter-commit ditambahkan pada commit docs-only ini (`ci_hosted_39486751618`, `39487122265`, `39488022468`, `39488028743`, `39491506434`, `39491517474`; masing-masing 30 suite 172/0, 13 `Compiling`, release 0 warning/0 error, ELF Linux 243.560 B). Termasuk di dalamnya dua run atas commit T1 `eaea57c` yang mengubah jalur build ISO — keduanya hijau, jadi gerbang G5 tidak terpicu. Angka pra-1b di atas tetap terbaca sebagai keadaan pada commit T0. **[PHASE-7-2, 2026-10-21 — status kini:]** terukur ulang `gh run list --workflow asl-ci-minimal --limit 60`: **25** run hijau = 14 rantai `main` + **11** `feature/phase7`; jumlah berkas ekstrak ter-commit **diukur** = **18** (empat sisanya masuk pada commit T2: `39492616260`, `39492625070` @ `7876749` dan `39493034754`, `39493041973` @ `489d4ee`, keempatnya tetap 30 suite 172/0, 13 `Compiling`, release 0 warning/0 error, ELF Linux 243.560 B), sehingga tingkat bukti menjadi **18 / 5 / 2** dan 18+5+2 = 25. Mulai commit T2 workflow berjalan dengan **dua** job — `test-and-build` (tetap satu-satunya required check) dan `coverage` **non-blocking** (`continue-on-error: true`, tidak didaftarkan di `required_status_checks`, ambang advisory `--fail-under-lines 80` yang baru ditetapkan setelah baseline host 86,82% terukur) — jadi "VERIFIED-ON-CI untuk langkah [1]–[2] saja" tetap berlaku untuk gerbang, dan keluaran job coverage di CI adalah **laporan**, bukan gerbang. | `Analysis/reports/ci_hosted_*.log` (7 ekstrak ter-commit pada keadaan row ini; kini 14 — lihat anotasi [PHASE-7-1b]) + `release_v1.1.0_evidence_2026-10-20.log` + `phase7_t0_protection_2026-10-21.md` |

### 0.2 Disiplin kategori bukti (ADR-002 §5) — dibaca bersama rilis ini
* **VERIFIED** = lulus di host (`cargo test`). **VERIFIED-ON-VM** = lulus di QEMU q35 via serial log. **VERIFIED-ON-CI** = dibuktikan oleh run Actions publik yang log mentahnya dibaca. **CODE-COMPLETE** = mekanisme ada, klaim penuh menuntut hardware gate.
* **VERIFIED-ON-HARDWARE = 0 item.** Yang masih menunggu board fisis: `DMA-ADDR-01`, `VT-D-DRV-01`, `DA-07`.
* **Bukti visual G9: SUDAH DIJALANKAN** pada CLOSEOUT T3 (2026-10-20 20:48:23, offset mesin +0700, hasil **PASS**, lihat §0.5) dengan jendela QEMU **terbuka** dan cmdline **tanpa** `-display none`. Catatan kategori: HV-1/HV-2 era v1.0.1 tetap **VERIFIED-ON-VM headless** (serial log saja, tanpa screenshot) dan tidak di-upgrade oleh run ini; run G9 adalah cold-boot penuh yang disaksikan, bukan pengukuran HV-1/HV-2. Hasil ini **tidak** mengubah **VERIFIED-ON-HARDWARE = 0**.

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
* **ISO tidak byte-reproducible** — perbaikan (penanam timestamp tetap) dijadwalkan di PHASE-7 T1. **[STATUS PHASE-7 T1, 2026-10-21: TERTUTUP untuk jalur build kanonik per-platform.** Dua build bersih di host Windows yang sama menghasilkan SHA256 ISO identik (`9cd49028…`, `cmp` 0 byte berbeda). Akar nondeterminisme ada **dua**, bukan satu: (a) stempel waktu xorriso di PVD + mtime berkas sumber — di-pin lewat `--modification-date`/`--set_all_file_dates` dari `SOURCE_DATE_EPOCH` (baked default 1767225600), dan (b) **Disk GUID acak yang ditulis `limine bios-install`** ke header GPT primer+cadangan beserta Unique GUID entri dan dua CRC32 turunannya — Limine 8.7.0 tidak punya opsi pin, sehingga GUID diturunkan deterministik dari epoch dan CRC dihitung ulang di build script (fail-closed bila header GPT tidak ditemukan). Delta ukuran biner **antar-platform** (ELF 8 B) tetap **didokumentasikan, bukan target**. Bukti: `Analysis/reports/iso_reproducible_t1_2026-10-21.log` + `Analysis/reports/phase7_t1_iso_reproducible_2026-10-21.md`]
* **Banner kernel di boot console masih menulis `ASL Kernel v1.0.2`** (`asl-kernel/asl-kernel-core/src/lib.rs:1126`, string hardcoded) sementara identitas rilis produk = **v1.1.0**. Yang disinkronkan oleh CM_PLAN §3 hanyalah tiga berkas konfigurasi/dokumen (`RELEASE_NOTES.md`, `CHECKLIST_STATE.json.version`, `HANDOFF_STATE.json`), bukan string di dalam source. Menyamakan banner menuntut perubahan `.rs` sehingga **tidak** dilakukan di CLOSEOUT (aturan "NOL perubahan .rs") — dijadwalkan di PHASE-7 dengan re-gate G4. Konsekuensi jujur: serial log G9 §0.5 mengutip `v1.0.2`, dan itu bukan salah ketik dokumen ini.
* Barrier cache HW riil (`DMA-CACHE-01`) masih **PARTIAL-MITIGATED (CODE)**; `ATOMIC-SPARC-01` masih **OPEN** (WCET SPARC butuh TSIM3/GRMON3).
* Tangga DAL-C prasyarat belum terpenuhi: MC/DC coverage, tim verifikasi independen, DO-330 tool qualification (`Gemini_Assist`/`Sub_Assist`), keterlibatan otoritas.
* Item higiene root (dulunya terbuka): 9 berkas `.log` pra-eksisting di root repo — **ditutup di CLOSEOUT T5** (2026-10-20): 4 disahkan ke `Analysis/reports/` dengan nama asli, 5 dihapus sebagai dupleks byte-identik (`cmp` = 0 byte berbeda), checker `Sub_Assist.py --hygiene` = `[PASS (V)] ROOT WORKSPACE BERSIH 100%`; manifest + SHA256 + pertimbangan di `Analysis/reports/hygiene_root_logs_2026-10-20.log`. Nol byte isi hilang dari repo.
* **CLOSEOUT T0–T6: SELESAI.** T6 branch protection `main` **dipasang sebagai langkah paling akhir, tepat setelah baris ini ter-commit dan ter-push** (urutan ini paksa: bukti administrasi harus masuk sebelum proteksi, karena sesudahnya direct push ditolak). Config atas keputusan pemilik "PR + 1 approval": required check `test-and-build`, merge wajib PR + 1 review approval, direct push ditolak (termasuk admin), force-push + penghapusan branch diblokir, `enforce_admins=true`. Payload + keadaan awal (`404 Branch not protected`) + asal nama check terdokumentasi di `Analysis/reports/branch_protection_main_2026-10-20.log`; **bukti GET pasca-pasang disampaikan di laporan chat** karena setelah proteksi aktif commit tidak bisa masuk `main` tanpa PR. Konsekuensi yang dinyatakan: dengan **satu akun** GitHub di repo, penulis PR tidak bisa approve PR sendiri — commit berikutnya (PHASE-7 / RND_09) butuh reviewer kedua atau penyesuaian aturan oleh pemilik. **[KOREKSI APPEND-ONLY PHASE-7 T0, 2026-10-21]** pemilik memilih jalur kedua: aturan diubah ke **Opsi B** (`required_approving_review_count=0`, `dismiss_stale_reviews=false`) sedangkan PR wajib, check `test-and-build` wajib hijau (strict), anti force-push, anti penghapusan branch dan `enforce_admins=true` **tetap** berlaku — payload PUT + verifikasi GET field-per-field ada di `Analysis/reports/phase7_t0_protection_2026-10-21.md` §2–§3, ratifikasinya fault log entri #13 dengan result CLOSED. Konsekuensi urutan langkah di bawah tidak berubah. Urutan langkah membuat commit bukti T4, T5 dan T6 masuk **di atas** tip bertag, jadi tip `main` tidak lagi identik dengan objek yang di-tag — tercatat di `Analysis/reports/release_v1.1.0_evidence_2026-10-20.log` bagian [6].

### 0.5 HASIL G9 ACCEPTANCE RUN (berjendela, disaksikan pemilik repo)
**STATUS: DIJALANKAN — PASS** (CLOSEOUT T3). Ini bukti visual cold-boot pertama yang memenuhi aturan G9; sebelumnya hanya ada bukti **VERIFIED-ON-VM headless** (serial log, tanpa jendela).

| Field | Nilai (terukur) |
|---|---|
| Tanggal & jam run | 2026-10-20 **20:48:23** (offset mesin +0700) — capai `ASL-OS>` pada 20:48:29 (mtime serial log) |
| QEMU version + cmdline | `QEMU emulator version 10.2.50 (v10.2.0-14086-g840320663e)`; `qemu-system-x86_64.exe -machine q35 -vga std -m 256M -cdrom C:\asl\asl-kernel\target\asl-core-kernel.iso -serial file:C:\asl\Analysis\reports\g9_acceptance_serial_2026-10-20.log -no-reboot -no-shutdown` — **TANPA `-display none` / `-WindowStyle Hidden`** |
| Jendela terlihat & disaksikan user | **YA** — proses PID 18760 punya `MainWindowTitle = "QEMU"`, jendela **dibiarkan TERBUKA** setelah PASS agar pemilik repo menyaksikan; ditutup manual via `scripts\g9_acceptance_run.ps1 -StopQemu` |
| Keluar yang diamati | **`M6: Kernel Cold Boot Complete`** (baris 32 serial) **+ prompt `ASL-OS>`** (baris 55, baris terakhir); banner `ASL Kernel v1.0.2`, 256 MB RAM terdeteksi, 6 perangkat PCI bus 0, DMAR **ABSEN** (VM tanpa IOMMU → probe VT-d tidak aktif di run ini) |
| Hasil checker | `M6=True shell=True crash=False`; **0** pola `ASL EXCEPTION` / `ASL-CORE-PANIC` / `FATAL`; serial **55 baris** (`wc -l` = 54 karena baris terakhir tidak diakhiri newline); stderr QEMU **0 baris pada saat checker selesai (20:48:29)**. Koreksi presisi: berkas stderr kemudian memuat **1 baris** pada **20:51:46** (±3 menit setelah prompt), yaitu `qemu: Gdk: gdk_seat_default_remove_tool: assertion 'tool != NULL' failed` — assertion **backend tampilan GTK QEMU** saat jendela berinteraksi dengan pointer user, **bukan** keluaran kernel dan tidak memuat exception/panic/FATAL. Berkas stderr ter-commit berisi baris tersebut agar pembaca tidak menemukan selisih dengan ringkasan |
| Serial log ter-commit | `Analysis/reports/g9_acceptance_serial_2026-10-20.log` (55 baris) + `Analysis/reports/g9_acceptance_summary_2026-10-20.log` (metadata run) + `Analysis/reports/g9_acceptance_qemu_stderr_2026-10-20.log` (1 baris assertion GTK QEMU pasca-boot, lihat baris "Hasil checker") |
| ISO yang di-boot: ukuran / SHA256 | **4.118.528 B** / `FACA92F4E1E92B2EE5EB8D2E931C0B1A64DF9C2035A308529E69D4E5128C8355` (build T2 di tip main `bb4a9df`; SHA256 **hanya sah untuk build ini** — ISO tidak byte-reproducible) **[KOREKSI APPEND-ONLY PHASE-7 T1, 2026-10-21: kalimat terakhir itu kini hanya berlaku untuk build pra-T1. Sejak T1, build ISO kanonik reproducible per-platform, jadi SHA256 build ulang di host yang sama tidak lagi berubah-ubah; biner yang di-boot pada baris di atas tetaplah yang ter-hash `faca92f4…` dan tidak di-rebuild]** |
| Kategori bukti yang bertambah | **VERIFIED-ON-VM** dengan observasi visual. **VERIFIED-ON-CI tetap langkah [1]–[2]. VERIFIED-ON-HARDWARE tetap 0 item.** |
| Harness | `scripts/g9_acceptance_run.ps1` — fail-closed: exit 1 pada timeout 180 s, crash, atau proses QEMU berhenti; tidak ada jalur "PASS" tanpa kedua string di serial log |

---

# BAGIAN RIWAYAT — v1.0.1 (2026-10-17)

> Teks di bawah adalah catatan rilis **v1.0.1** apa adanya (dipertahankan untuk jejak audit). **Awas:** beberapa kalimat di §3 bagian ini memakai istilah "Fully resolved" yang **tidak lagi sesuai** kategori bukti kanonik saat ini — lihat koreksi di §3.1. Sumber kebenaran register = `audit/asl-core-kernel/SECURITY_FINDINGS.md`.

**Release Version:** `v1.0.1`  
**Date:** 2026-10-17  
**Architecture Authority:** `asl_core.md` (150 Silicon Platforms)  
**Governing Standard (label historis):** DO-178C DAL-A / ARINC 653 Integrated Modular Avionics (IMA)  

---

### 0.6 PHASE-7 (mulai 2026-10-21) — status di atas rilis v1.1.0
* Rilis produk tetap **v1.1.0** (tag annotated `90954ce` -> peel `7104814`). PHASE-7 berjalan di branch `feature/phase7` (base `6623dd8`) dan **satu PR (#4)**; `main` tidak boleh menerima direct push lagi, jadi semua bukti fase ini masuk lewat PR.
* **T0 selesai (commit `PHASE-7-0`, docs/evidence-only, nol `.rs`):** proteksi Opsi B dipasang + diverifikasi GET; ekstrak log mentah run `39444157361` masuk repo lewat PR (pertama kalinya bukti hosted masuk lewat pipeline baru); hitungan run direkonsiliasi ke **14** dengan **tiga** tingkat bukti; 13 berkas `.log` scratch tak-ter-track di `Analysis/reports/` diklasifikasi dengan bukti byte/line — 9 dihapus (7 dupleks byte-identik terbukti `cmp`, 1 tersimpan penuh di korpus ter-commit, 1 murni bungkus PowerShell), 4 disahkan sebagai baseline fase (inventaris nama test pra-T3 + transkrip xorriso pra-T1 + serial QEMU unik). Nol byte isi hilang tanpa dicatat.
* Keterbatasan yang **masih terbuka** dan jadi sasaran fase ini: ISO tidak byte-reproducible (T1), coverage `asl-storage`/`asl-bootloader`/`asl-kernel-core-secondary` belum terukur (T2), penamaan fungsi test M-1 (T3), banner kernel `v1.0.2` di `lib.rs:1126` (T4, butuh otorisasi eksplisit owner + run berjendela ulang). **[STATUS PHASE-7 T1, 2026-10-21: butir pertama (ISO reproducible per-platform) sudah ditutup di T1 — lihat bullet T1 di bawah; tiga butir lainnya masih terbuka sesuai jadwal.]** **[STATUS PHASE-7 T2, 2026-10-21: butir kedua (coverage tiga crate terukur) ditutup di T2 dengan baseline terukur + job CI non-blocking; dua butir tersisa (T3 rename test M-1, T4 banner + run berjendela) masih terbuka.]**
* **T1 selesai (commit `[PHASE-7-1]`, perubahan jalur build + docs/evidence, nol `.rs` dan nol `Cargo.*`):** kriteria exit tercapai dan **terukur** — dua build bersih pada host+platform sama menghasilkan ISO SHA256 identik `9cd49028d8b1b9b70d9316c9410a2902f20673cbb6c0ca5836dfe5808cc8e8f2` (4.118.528 B, `cmp` 0 byte berbeda); jalur `SOURCE_DATE_EPOCH=1700000000` juga berpasangan identik (`6be74c35…`), membuktikan pin bekerja lewat env bukan kebetulan. Satu berkas berubah: `scripts/build_kernel.ps1` (aksi `iso`). Penemuan pengukuran: sumber nondeterminisme ada **dua** — stempel waktu xorriso **dan** GUID GPT acak dari `limine bios-install` (144 byte berbeda pada 12 run, seluruhnya di struktur GPT). Image ter-pin tetap boot di QEMU 10.2.50 (`M6=True shell=True crash=False`, serial 55 baris, stderr 0 baris) — dikategorikan **VERIFIED-ON-VM smoke setara langkah [3/4], BUKAN run penerimaan G9**; G9 PHASE-7 dijadwalkan di T4. Gerbang G4 diulang pada tree ini: 30 suite **172/0**, release **0 warning / 0 error**, ELF 243.568 B. Yang **tidak** disentuh: `scripts/test_hv2_stack_overflow_qemu.ps1` (jalur ISO tersuntik non-kanonik, masih non-reproducible), `CHECKLIST_STATE.json`, artefak bukti pra-T1. Ratifikasi keputusan + batas dinyatakan di `Analysis/reports/phase7_t1_iso_reproducible_2026-10-21.md` §5, fault log entri #16.
* **T1 susulan `[PHASE-7-1b]` (docs-only, 2026-10-21):** dua run hosted atas commit T1 `eaea57c` (`39491506434` push + `39491517474` pull_request) **hijau** — syarat "CI merah = STOP" (G5) tidak terpicu; log mentahnya diekstrak dan di-commit bersama empat ekstrak branch fitur yang tertunda, sehingga tingkat bukti tertinggi menjadi 14 dari 21 run. Satu kekeliruan angka dikoreksi append-only: transkrip bukti T1 adalah 122 baris (terukur `wc -l`), bukan 123 seperti yang dikutip commit `eaea57c`; ratifikasi fault log entri #17.
* **T2 (coverage terukur + job CI non-blocking, 2026-10-21, satu berkas workflow + docs/evidence — nol `.rs`, nol `Cargo.*`):** celah "coverage belum terukur" ditutup dengan angka, bukan klaim. `cargo llvm-cov 0.9.1` (tooling host, di-install agen **atas izin eksplisit pemilik**; bukan dependency crate sehingga G1 utuh) pada `asl-storage` + `asl-bootloader` + `asl-kernel-core-secondary`, `--lib`, target host, rev `489d4ee`: **regions 87,72%** (6.027/740 missed), **functions 80,89%** (225/43), **lines 86,82%** (3.718/490); **branch TIDAK DIUKUR** (toolchain stable). Per crate regions/lines: bootloader 91,75/91,37%, kernel-core-secondary 91,99/92,71%, storage 87,08/85,83%. Run kedua pada rev yang sama → **13 baris tabel byte-identik**. Job `coverage` **non-blocking dua lapis** (`continue-on-error: true` + tidak ada di `required_status_checks.contexts = ["test-and-build"]`) dengan ambang advisory `--fail-under-lines 80` yang **ditetapkan setelah** baseline dan yang mekanisme diuji (exit 0 pada 80, exit 1 pada 90; versi ini tidak mencetak pesan saat ambang gagal, jadi penanda pelanggaran adalah exit code). Batas yang dinyatakan terang: angka ini "baris berkas yang pernah dijalankan" dan **bukan** "logika produksi yang teruji" (modul uji berada di dalam `lib.rs` yang sama), hanya tiga crate (tidak ada angka workspace-wide), **59 test pada 3 suite** bukan pengganti gerbang G4 **172 test pada 30 suite**, dan tidak ada dasar klaim MC/DC untuk tangga DAL-C. Gerbang G4 **diulang lokal pada tree commit ini sebelum push** dan tetap hijau: `cargo test --workspace --lib` = 30 suite **172 passed / 0 failed** (exit 0), dan release `x86_64-unknown-none` **setelah `target/x86_64-unknown-none/release` dihapus** = 13 baris `Compiling`, **0 warning / 0 error**, ELF 243.568 B dengan SHA256 identik dengan biner sebelum dibersihkan. Bukti: `Analysis/reports/coverage_baseline_t2_2026-10-21.log` (233 baris ASCII, memuat keluaran mentah 99 baris) + `Analysis/reports/phase7_t2_coverage_2026-10-21.md` + fault log entri #18. Empat ekstrak CI yang tertunda ikut ditutup (rantai hijau terukur **25** run; ekstrak ter-commit **18**). `CHECKLIST_STATE.json` tidak disentuh.
* Bukti: `Analysis/reports/phase7_t0_protection_2026-10-21.md`, `Analysis/reports/phase7_t0_hygiene_2026-10-21.md`, `Analysis/reports/ci_hosted_39444157361_2026-10-20.log`, `Analysis/reports/ci_hosted_39486017207_2026-10-20.log` (run branch fitur pertama; ekstrak-nya masuk repo lewat PR, bukti fungsional bahwa pipeline baru bekerja).

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
