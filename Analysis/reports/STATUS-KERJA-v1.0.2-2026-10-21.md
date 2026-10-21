# STATUS KERJA MISI v1.0.2 - SHELL INTERAKTIF + IDENTITAS v1.0.2 + DOKUMENTASI PANDUAN

**Tanggal:** 2026-10-21 | **Branch:** `feature/v1.0.2-shell` | **Tip saat dokumen ini ditulis:** `36cf856`
**Otorisasi dokumen:** instruksi pemilik repo pada 2026-10-21 ("buatkan laporan hasil pekerjaan yang
sekarang dalam bentuk .md saja").

## 0. Status dokumen ini

Dokumen ini **BUKAN** `Analysis/reports/LAPORAN-FINAL-v1.0.2.md`. Laporan final versi 1.0.2 adalah
langkah **P10** dan masih kosong: Bagian 7 misi melarang berkas laporan apa pun dibuat sebelum
pengujian akhir selesai, dan pengujian akhir (jendela P8 yang disaksikan pemilik, lalu rilis P9) belum
berlangsung. Yang ada di sini adalah **potret pekerjaan yang sudah selesai** per 2026-10-21, dengan
setiap angka disertai sumber pengukurannya dan kelas buktinya.

Deviasi "dokumen status dibuat sebelum pengujian akhir" dicatat apa adanya di
`docs/ROADMAP/ACCOUNTABILITY_FAULT_LOG.json` entri ke-26, dengan sumber otorisasi di atas.

Kelas bukti yang dipakai di dokumen ini:

| Label | Arti |
|---|---|
| VERIFIED-ON-HOST | Terukur di mesin ini (host build, toolchain MSVC) |
| VERIFIED-ON-CI | Terukur dari log runner hosted (GitHub Actions) |
| VERIFIED-ON-CI-API | Terukur dari API GitHub (metadata run, state PR, proteksi branch) |
| VERIFIED-ON-VM | Terukur di QEMU headless di mesin ini |
| RANCANGAN | Disusun, belum dieksekusi |

Tidak ada satu pun klaim "VERIFIED-ON-HARDWARE" di dokumen ini: tidak ada perangkat keras nyata yang
dilibatkan, dan tidak ada klaim tangkapan layar.

## 1. Peta tahap dan statusnya

| Tahap | Isi | Commit | Status |
|---|---|---|---|
| P0-sisa | Tiga utang fase PHASE-7 ditutup dengan pengukuran | `5ead484` (lewat PR #5) | Selesai |
| P1 | Branch `feature/v1.0.2-shell` dibuat dari `main` | (tanpa commit sendiri) | Selesai |
| P2 | Konstanta satu-sumber identitas rilis (versi + prompt) | `f34c197` (lewat PR #5) | Selesai |
| P3 | Shell kanonik: 11 perintah + 9 alias, jembatan VFS ke RAMDISK volatil, penyunting baris ANSI | `112fc38` | Selesai |
| P4 | Gerbang regresi memori boot 3 MB + perbaikan dobel `ci_local.ps1` | `eeef264` | Selesai |
| P5 | Uji sinkron dokumen perintah (dokumen dibaca `include_str!`, dibandingkan field-per-field) | `206cb76` | Selesai |
| P6 | Dokumentasi PANDUAN berbahasa Indonesia + peta kanonik | `e20c424` | Selesai |
| P7 | Gerbang penuh diukur ulang + lima ekstrak hosted CI | `02faa26` | Selesai |
| P7B | Dua ekstrak hosted CI atas tip P7 + sinkron HANDOFF | `63f0e3e` | Selesai |
| P8-PERSIAPAN | Alat jendela penerimaan fail-closed dibuat dan diukur (bukan jendela P8 itu sendiri) | `36cf856` | Selesai |
| **P8** | **Jendela G9 akhir, disaksikan pemilik** | - | **Menunggu pemilik** |
| **P9** | Rilis v1.0.2 + penghapusan tag/rilis v1.1.0 | - | **Belum** |
| **P10** | `LAPORAN-FINAL-v1.0.2.md` | - | **Belum (harus terakhir)** |

PR #5 sudah MERGED (merge commit `ef8c1b4`, 2026-10-20T23:54:19Z, VERIFIED-ON-CI-API). PR #6 masih
OPEN dengan 7 commit (`112fc38` .. `36cf856`), `mergeable=MERGEABLE`, `mergeStateStatus=CLEAN`
(VERIFIED-ON-CI-API).

Total perubahan misi v1.0.2 pada branch (merge-base `ef8c1b4` .. `36cf856`, 7 commit): 31 berkas,
+5.900/-86 baris, seluruhnya di dalam cakupan P2-P3 dan dokumen/bukti (VERIFIED-ON-HOST lewat
`git diff --shortstat`). Dokumen ini masuk pada commit berikutnya, jadi semua tree yang disebut
adalah `36cf856`.

## 2. Hasil kerja yang sudah jadi

### 2.1 Shell kanonik (P3)

- **11 perintah kanonik** dengan registry self-describing: `help`, `clear`, `where`, `list`,
  `open`, `create-folder`, `create-file`, `write-file`, `delete-file`, `delete-folder`, `status`.
- **9 alias**: `ls`, `dir` (list); `cd`, `cat`, `type`, `read-file`, `open-file` (open); `mkdir`
  (create-folder); `rm` (delete-file).
- Perintah lama (`meminfo`, `cpuinfo`, `uname`, `lspci`, `lsblk`, `fdisk`, `mount`, `failover`,
  `reboot`, `halt`, `wfi`, `hlt`, `test_*`, `trap_*`) tetap dikenali lewat jalur pengalihan lama;
  keluarannya tidak berubah.
- Penyunting baris dengan sekuens escape, prompt polos tanpa posisi folder, dan baris `Sintaks:`
  yang diambil dari registry (bukan teks manual).
- Sumber: `asl-kernel/asl-cockpit-shell/src/lib.rs` dan `asl-kernel/asl-kernel-core/src/lib.rs`.

### 2.2 Penyimpanan (P3)

- RAMDISK volatil sebagai penyimpanan shell: kapasitas **262.144 byte (256 KiB)**, satu slot berkas
  **8.192 byte** (`RAMDISK_FILE_SLOT_BYTES = 8 * 1024`), 32 node default
  (`RAMDISK_CAPACITY_DEFAULT_BYTES = RAMDISK_DEFAULT_NODES * RAMDISK_FILE_SLOT_BYTES`), maksimum 64 node
  = 524.288 byte. Isi hilang ketika mesin dimatikan.
- Penurunan default 512 KiB ke 256 KiB diambil karena tabrakan dengan lantai memori boot 3 MB
  (keputusan pemilik, tercatat di fault log entri ke-23).

### 2.3 Identitas rilis satu-sumber (P2)

- `KERNEL_VERSION: &str = "v1.0.2"` dan `SHELL_PROMPT: &str = "ASL-KERNEL> "` di
  `asl-kernel/asl-cockpit-shell/src/lib.rs` (baris 22 dan 25). Semua situs tampilan membaca konstanta
  ini; tidak ada penulisan versi kedua di tempat lain.

### 2.4 Dokumentasi (P6) dan penjaganya (P5)

- Empat berkas baru di `docs/PANDUAN/`: `README.md`, `MULAI-CEPAT.md`, `KONFIGURASI-MEMORI.md`,
  `REFERENSI-PERINTAH.md`. Semuanya berbahasa Indonesia, identifier kode tetap bahasa Inggris.
- Dua penjaga test menjaga sinkronnya: `test_docs_command_reference_matches_registry` (isi registry,
  prompt, identitas rilis) dan `test_docs_capacity_figures_match_constants` (angka kapasitas pada
  Pasal 2 dan Pasal 6 dibaca dari konstanta RAMDISK). Kalau registry atau konstanta berubah dan dokumen
  tidak, test menjadi merah.

### 2.5 Alat penerimaan jendela (P8-PERSIAPAN)

`scripts/g9_final_p8_run.ps1` - 363 baris, 17.958 byte, 0 karakter non-ASCII, SHA256
`2a866129fa313ef62a4a3992a84beb63be02f54be63d2570dd95885c79265571` (VERIFIED-ON-HOST). Aturan
fail-closed: jendela QEMU **tanpa** `-display none`, prompt dibaca dari konstanta satu-sumber, 13
penanda sesi yang asal tiap penandanya ditelusuri ke baris sumber, dan kode keluar 0/1/2/3 dengan
makna berbeda: lulus, boot-gagal, sesi belum lengkap, atau 3M merah tanpa penerimaan pemilik.

## 3. Angka hasil pengukuran

| Yang diukur | Angka | Kelas bukti | Cara diukur |
|---|---|---|---|
| Test host (30 suite) | 196 lulus, 0 gagal, 0 diabaikan | VERIFIED-ON-HOST | `cargo test --workspace --lib` |
| Baris `warning` pada profil test host | 21 (non-blocking) | VERIFIED-ON-HOST | hitung baris `^warning` pada log host |
| Baseline misi (172 test) | terpenuhi (196 >= 172) | VERIFIED-ON-HOST | angka dari lock verifikasi misi |
| Build release di host, direktori build baru | 0 error, 0 warning, 13 `Compiling` | VERIFIED-ON-HOST | `cargo build --package asl-kernel-bin --target x86_64-unknown-none --release` |
| Build release di runner hosted | 0 error, 0 warning, 13 `Compiling` | VERIFIED-ON-CI | langkah [2/2] run 39534573189 |
| Hosted CI atas tip `36cf856` | 2 peristiwa, dua-duanya `success` | VERIFIED-ON-CI-API | run 39534573189 (push) + 39534577564 (pull_request) |
| ELF runner CI | 325.584 byte | VERIFIED-ON-CI | `ls -l` pada langkah bukti footprint |
| ELF build host | 325.528 byte, SHA256 `9203864e...a27b216` | VERIFIED-ON-HOST | `sha256sum` + `wc -c` |
| ELF dari direktori build keempat | 325.528 byte, hash identik `9203864e...` | VERIFIED-ON-HOST | build dari nol di `logs/target_p8check` |
| Blok tabel coverage 13 baris | SHA256 `94ee1504...da493` (gabungan LF) | VERIFIED-ON-CI | region llvm-cov pada job coverage |
| ISO rilis | 4.200.448 byte, SHA256 `aa778060...c74de9` | VERIFIED-ON-HOST | `sha256sum` + `wc -c` |
| ISO dua build, direktori sama | identik byte per byte (`cmp` keluar 0) | VERIFIED-ON-HOST | dua kali `scripts/build_kernel.ps1 -Action iso` |
| ELF utuh di dalam ISO | 1 kejadian | VERIFIED-ON-HOST | pencarian byte utuh |
| Cold boot 256M | M6 + prompt tercapai, crash 0, stderr 0 byte, serial 2.516 byte | VERIFIED-ON-VM | `scripts/test_ram_floor_3mb.ps1` dan skrip jendela |
| Reproduksi dingin | serial P7 dan serial kendali-negatif identik (`8c800666...3ca`) | VERIFIED-ON-VM | SHA256 dua berkas |
| Lantai boot 3328K | **gagal** (serial 0 byte) | VERIFIED-ON-VM | probe pada ISO rilis tip `36cf856` |
| Lantai boot 3360K | **lulus** (serial 2.496 byte, stderr 0) | VERIFIED-ON-VM | probe pada ISO rilis tip `36cf856` |
| Janji `-m 3M` | **gagal** (serial 0 byte) | VERIFIED-ON-VM | probe pada ISO rilis tip `36cf856` |
| Alat jendela, `-SelfTest` | keluar 0 | VERIFIED-ON-HOST | `scripts/g9_final_p8_run.ps1 -SelfTest` |
| Alat jendela, kendali-negatif | keluar 2, 13 dari 13 penanda terukur 0 | VERIFIED-ON-HOST | `-SessionTimeoutSeconds 5` |
| G10 (CJK) pada 31 berkas berubah | 0 | VERIFIED-ON-HOST | hitung `ord >= 0x2E80` per berkas |
| Bukti ter-track di `Analysis/reports` | 100 berkas, 42 di antaranya `ci_hosted_*` (setelah commit dokumen ini) | VERIFIED-ON-HOST | `git ls-files` |
| Fault log | 26 entri (termasuk entri ke-26 pada commit ini), `violations_recorded` = 0 | VERIFIED-ON-HOST | `docs/ROADMAP/ACCOUNTABILITY_FAULT_LOG.json` |
| HANDOFF state | 27 kunci | VERIFIED-ON-HOST | `docs/ROADMAP/HANDOFF_STATE.json` |

Konvensi angka byte: **panjang disk** (`Get-Item` / `wc -c`). Jumlah baris dan pasangan CRLF
dilaporkan sebagai field terpisah. Aturan ini lahir dari pembatalan satu angka sendiri (entri ke-25
di bawah).

## 4. Jejak bukti

- `Analysis/reports/` - 100 berkas ter-track; 42 di antaranya ekstrak hosted CI
  `ci_hosted_<run_id>_2026-10-21.log`. Ekstrak baru atas tip `36cf856` (39534573189 push,
  39534577564 pull_request) ikut dalam commit dokumen ini.
- `Analysis/reports/p8_prep_selftest_2026-10-21.log` - 117 baris: statistik skrip, tabel asal 13
  penanda, keluaran SelfTest, kendali-negatif, pernyataan bahwa ini bukan jendela P8, reproduksi
  dingin, koreksi konvensi, dan sidik jari artefak.
- `Analysis/reports/p7_gate_measures_2026-10-21.log` - 124 baris; 10 baris koreksi disisipkan
  (difflib: hanya `equal`/`insert`, nol `delete`/`replace`) tanpa menghapus baris lama.
- `docs/ROADMAP/ACCOUNTABILITY_FAULT_LOG.json` - kronologi kegagalan dan pembatalan klaim.
- `docs/ROADMAP/HANDOFF_STATE.json` - keadaan terkini dan langkah berikutnya, satu-sumber.

## 5. Yang BELUM selesai (terbuka, apa adanya)

1. **P8 - jendela G9 akhir belum dijalankan.** Yang sudah diukur adalah alatnya, bukan jendelanya.
   Jendelanya harus disaksikan pemilik:
   `powershell -ExecutionPolicy Bypass -File scripts\g9_final_p8_run.ps1`.
2. **Gerbang Bagian 5 `-m 3M` masih MERAH.** Lantai terukur: 3.360 KB, bukan 3 MB. Tercatat sebagai
   `ACCOUNTABILITY_FAULT_LOG` entri ke-23 dengan result **OPEN**. Status merah itu tidak diubah
   menjadi hijau; penerimaan deviasi adalah keputusan pemilik (tanda `-AcceptDeviation3M`).
3. **Merge PR #6 ditahan.** Kebijakan 0.2b mengizinkan merge otomatis hanya bila semua gerbang
   Bagian 5 hijau. Merge terjadi di jendela P8/P9 bersama keputusan pemilik.
4. **P9 belum jalan.** Tag dan rilis `v1.1.0` masih ada di GitHub; `v1.0.2` belum ada. Operasi tag
   wajib dicatat sebagai entri fault log, dan akan menjadi ke-27 setelah entri ke-26 dokumen ini.
5. **P10 belum jalan.** `LAPORAN-FINAL-v1.0.2.md` belum ada dan memang tidak boleh ada sebelum P8/P9.

## 6. Keputusan otonom yang diambil (dan alasannya)

1. **Deviasi 3 MB** dibiarkan terbuka dan tercatat, bukan dipoles (keputusan pemilik; entri ke-23).
2. **FAT32 create/delete** sebagai STRETCH ditinggalkan karena berisiko; dicatat di laporan akhir.
3. **RAMDISK default turun 512 KiB ke 256 KiB** karena tabrakan dengan lantai memori boot 3 MB.
4. **Urutan P5 sebelum P6** dipertahankan: penjaga sinkron dokumen harus ada sebelum dokumen ditulis.
5. **Dobel `ci_local.ps1` diperbaiki**; skrip lama tetap ada tetapi tidak boleh dijalankan apa adanya
   karena menimpa artefak bukti yang ter-commit.
6. **Akar masalah insiden RUSTFLAGS** ditemukan: variabel lingkungan `RUSTFLAGS` **menggantikan**
   (bukan menambah) rustflags di `.cargo/config.toml`, sehingga linking tanpa `-Tlinker.ld` rusak.
   Runner hosted sengaja tidak memasang env itu.
7. **Klaim sidik jari ELF per-direktori-build dibatalkan lewat pengukuran** (entri ke-24): empat
   direktori build pada tree yang sama (`asl-kernel/target`, `logs/target_rustflags_trap`,
   `logs/target_p7check`, `logs/target_p8check`) menghasilkan ELF 325.528 byte dengan hash identik
   `9203864e...`. Direktori `logs/target_fromzero` memang berbeda (`765f4e7b...`) karena isinya build
   dari tree SEBELUM tahap P5; faktor penentunya isi tree, bukan lokasi direktori build.
8. **Penahan merge menunggu P8** meski CI hijau, karena gerbang `-m 3M` merah.
9. **Cacat teks G10** pada commit `eeef264`, `206cb76`, `e20c424` dicatat dan dikoreksi di commit
   berikutnya tanpa menulis ulang riwayat.
10. **Penanda `list` diperketat menjadi `Daftar isi /`** karena ringkasan registry pada
    `asl-kernel/asl-cockpit-shell/src/lib.rs:378` juga tercetak lewat `help`; penanda tanpa `/` bisa
    terpenuhi tanpa menjalankan `list` sama sekali. Alasannya ditulis sebagai komentar di dalam skrip.
11. **Konvensi angka byte ditegaskan**: byte = panjang disk; baris dan CRLF field terpisah (entri ke-25).
12. **Dokumen status ini dibuat lebih awal atas instruksi pemilik** (entri ke-26).

## 7. Checklist uji nyata (status per 2026-10-21)

| Uji | Gerbang | Hasil | Kelas |
|---|---|---|---|
| Test host | minimal 172, tanpa penurunan | 196 lulus, 0 gagal | VERIFIED-ON-HOST |
| Build release | 0 error, 0 warning | 0/0 (host dan CI) | VERIFIED-ON-HOST, VERIFIED-ON-CI |
| QEMU smoke 256M | M6 + prompt, crash 0, stderr 0 | terpenuhi (headless) | VERIFIED-ON-VM |
| Sesi Bagian 3 | 11 perintah + alias + editor + RAMDISK, disaksikan pemilik | **belum** (menunggu P8) | - |
| Gerbang `-m 3M` | lulus | **MERAH** (lantai terukur 3.360 KB) | VERIFIED-ON-VM |
| ISO dua build | byte-identik | terpenuhi | VERIFIED-ON-HOST |
| Hosted CI hijau | sebelum merge | hijau atas `36cf856` | VERIFIED-ON-CI-API |
| G8 | nol pelanggaran | 0 | VERIFIED-ON-HOST |
| G10 | CJK = 0 | 0 | VERIFIED-ON-HOST |
| Rilis v1.0.2 | tag + rilis GitHub | **belum** | - |

## 8. Cara melanjutkan

```
# 1) jendela P8 (dijalankan pemilik, jendela QEMU terbuka)
powershell -ExecutionPolicy Bypass -File scripts\g9_final_p8_run.ps1
#    jika deviasi 3 MB diterima secara sadar:
powershell -ExecutionPolicy Bypass -File scripts\g9_final_p8_run.ps1 -AcceptDeviation3M
# 2) tutup jendela bila perlu
powershell -ExecutionPolicy Bypass -File scripts\g9_final_p8_run.ps1 -StopQemu
# 3) P9: merge PR #6, hapus tag dan rilis v1.1.0, buat tag annotated + rilis v1.0.2
# 4) P10: tulis Analysis/reports/LAPORAN-FINAL-v1.0.2.md (PALING AKHIR)
```
