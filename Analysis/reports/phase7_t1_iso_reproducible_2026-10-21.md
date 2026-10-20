# PHASE-7 T1 - ISO reproducible per-platform (2026-10-21)

Aturan yang mengikat langkah ini: kontrak PHASE-7 memperbolehkan perubahan di luar berkas `.rs`
untuk jalur build; kontrak `.rs` tetap tertutup rapat (hanya rename test di T3 dan satu string banner di T4).
G1 (nol dependensi baru), G4 (gerbang test+build diulang), G5 (CI merah = STOP), G7 (angka harus terukur),
G8 (nol amend/rebase/reset; koreksi selalu commit baru yang menyebut hash yang dikoreksi) dan R1/R2
(bukti append-only + laporan per langkah) dipatuhi. Nol berkas `.rs` dan nol `Cargo.*` berubah di T1;
`CHECKLIST_STATE.json` tidak disentuh (24 gerbang terkunci tetap).

## 1. Yang ditargetkan

Temuan **AUD-03** (`docs/ROADMAP/AUDIT_ASL_AUD_03_ADVERSARIAL_PR2.md`): dua build independen menghasilkan
SHA256 ISO yang berbeda walaupun biner ELF di dalamnya identik, karena `xorriso 1.5.2` menanam stempel waktu
generasi pada volume descriptor. Exit criterion PHASE-7 T1: **dua build bersih pada host dan platform yang sama
harus menghasilkan SHA256 ISO yang identik**; delta antar-platform (8 B) didokumentasikan dan **bukan** target.
Baseline pra-T1 yang dipakai: `Analysis/reports/ci_2026-10-20.log.step3_iso.log` (transkrip xorriso yang disahkan
di T0) + `Analysis/reports/iso_build_v1.1.0_2026-10-20.log` (ISO rilis 4.118.528 B, SHA256 `faca92f4…`).

## 2. Diagnosis: ada DUA sumber nondeterminisme, bukan satu

| sumber | bukti terukur | perlakuan |
|---|---|---|
| (a) stempel waktu xorriso di PVD + mtime berkas sumber di Rock Ridge | ISO pra-T1: PVD `created/modified = 2026102012363400` (= waktu generasi) dan berubah tiap rebuild | `--modification-date=<YYYYMMDDhhmmsscc>` dan `--set_all_file_dates=<sama>`, nilainya dari `SOURCE_DATE_EPOCH` |
| (b) **Disk GUID acak yang ditulis `limine.exe bios-install`** di header GPT primer+cadangan, plus Unique GUID tiap entri partisi yang diturunkan darinya dan dua CRC32 yang mengikuti | Dua build bersih dengan pin tanggal saja: `iso1` `230bc0aa…` vs `iso2` `84ac73f9…`; ELF identik `ee1274be…`; selisih **144 byte pada 12 run** dan **seluruhnya** berada di struktur GPT (`0x210`, `0x238`, `0x258`, `0x410/0x490/0x510`, `0x3e5a10/0x3e5a90/0x3e5b10`, `0x3ed610/0x3ed638/0x3ed658`) | GUID ditentukan ulang dari epoch (`SHA256("ASL-ISO-GPT:<epoch>")`), lalu `PartitionEntryArrayCRC32` dan `HeaderCRC32` dihitung ulang sesuai tata letak GPT UEFI |

Tanpa butir (b), T1 **tidak** akan lolos meski tanggal sudah di-pin - ini penemuan pengukuran, bukan asumsi.
`limine 8.7.0` tidak menyediakan opsi untuk menetapkan GUID (diperiksa dari `limine.exe bios-install --help`:
hanya `--force-mbr`, `--uninstall`, `--uninstall-data-file`, `--quiet`), jadi pin dilakukan pada image hasil,
dengan dasar bahwa GUID itu tidak dipakai untuk mencari stage 2 (lokasi stage 2 dinyatakan sebagai LBA, dan
lintasan boot BIOS tetap terbukti jalan - butir 4.3).

## 3. Perubahan

Satu berkas: `scripts/build_kernel.ps1`, aksi `iso`.

- Langkah 4: `SOURCE_DATE_EPOCH` dibaca bila berisi digit (`^\d{1,10}$`); bila tidak diset, dipakai konstanta
  baked `1767225600` = 2026-01-01T00:00:00Z. Nilai epoch dikonversi ke `yyyyMMddHHmmss` + `00` dan diberikan ke
  xorriso lewat `--modification-date=` dan `--set_all_file_dates`.
- Langkah 6 (baru): setelah `bios-install`, semua header `EFI PART` (512-byte aligned, rev 0x10000) dicari;
  Disk GUID diganti dengan turunan epoch, Unique GUID tiap entri partisi = GUID basis dengan byte ke-9 = indeks+1;
  `PartitionEntryArrayCRC32` (offset 88) dan `HeaderCRC32` (offset 16, dihitung dengan field itu dinolkan)
  dihitung ulang untuk header primer dan cadangan, lalu image ditulis balik. Bila header tidak ditemukan,
  build **STOP** (fail-closed) - tidak ada jalur yang diam-diam merilis image tanpa pin.
- Helper `Get-Crc32`/`Get-Slice`/`Set-UInt32Le` memakai aritmetika Int64 dan konstanta desimal. Alasan teknis
  (terukur, bukan dugaan): PowerShell 5.1 mem-parse literal heksa di atas `0x7FFFFFFF` sebagai Int32 negatif,
  sehingga `0xFFFFFFFF` dalam `-bxor`/`-band` melakukan sign-extension. Versi pertama menghasilkan
  `crc32("123456789") = 0xFFFFFFFFF273791B` (salah); setelah diperbaiki `0xCBF43926` (vektor uji baku).
- Nol dependensi baru: hanya .NET BCL (`SHA256`, `BitConverter`, `IO.File`).

## 4. Pengukuran

### 4.1 Kriteria exit - dua build bersih, host dan platform sama

"bersih" = `target/x86_64-unknown-none/release` dihapus + ISO lama dihapus + salinan `iso_root/boot/asl-kernel.elf`
dihapus (direkompilasi dari nol: Build A mencatat 13 baris `Compiling` dan `Finished … in 6.78s`); staging Limine
dipertahankan karena ia input mastering, bukan keluaran cargo.

| build | ukuran ISO | SHA256 |
|---|---|---|
| A (`logs/t1_cleanA.log`) | 4.118.528 B | `9cd49028d8b1b9b70d9316c9410a2902f20673cbb6c0ca5836dfe5808cc8e8f2` |
| B (`logs/t1_cleanB.log`) | 4.118.528 B | `9cd49028d8b1b9b70d9316c9410a2902f20673cbb6c0ca5836dfe5808cc8e8f2` |

`cmp` atas kedua salinan = keluar 0, **0 byte berbeda**. **KRITERIA EXIT T1 TERPENUHI.**

### 4.2 Jalur `SOURCE_DATE_EPOCH` dan validasi struktur

Dua build dengan `SOURCE_DATE_EPOCH=1700000000` → PVD `2023111422132000`, GUID `d86dec7b-2df4-4b49-b100-3b4e4941a550`,
ISO `epochD`/`epochE` sama-sama `6be74c3537de225720b3e9e57fc9b4c0b559b7caa603d026125cfbdc2b52a3ef` (`cmp` 0 byte).
Membuktikan pin bekerja lewat env dan bahwa identitas hash bukan kebetulan.

Struktur GPT image ter-pin divalidasi dengan parser **independen** (Python `struct` + `binascii.crc32`, bukan kode build):
header primer `0x200` (myLBA 1, backupLBA 8043, FirstUsable 64, LastUsable 7980, 44 entri × 128 B, array di LBA 2) dan
header cadangan `0x3ed600` (array di LBA 7981). Disk GUID kedua header = nilai pin; CRC32 array dan CRC32 header
**cocok** dengan hitung ulang; entri `Gap0` (64..127), `EFI boot partition` (128..5887), `Gap1` (5888..7979) membawa
Unique GUID turunan (byte ke-9 = 01/02/03).

### 4.3 Image ter-pin tetap boot - dengan kategori bukti yang benar

QEMU 10.2.50, `-machine q35 -vga std -m 256M -cdrom logs/t1/cleanA.iso`, serial ke berkas:
`reached=True m6=True crash=False`, serial **55 baris**, stderr QEMU 0 baris → **BOOT CHECK PASS**.
Kategori: **VERIFIED-ON-VM** setara smoke rutin langkah [3/4] gerbang lokal (headless), **bukan** run penerimaan G9
dan **bukan** VERIFIED-ON-HARDWARE. Jalur UEFI/ESP tidak dilatih lokal - sama seperti fase sebelumnya; untuk jalur itu
yang validated hanyalah konsistensi struktur GPT di 4.2. Run G9 PHASE-7 yang disaksikan pemilik dijadwalkan di T4.

### 4.4 Gerbang G4 pada tree ini

- `cargo test --workspace --target x86_64-pc-windows-msvc --lib` → **30 suite, 172 passed / 0 failed**, exit 0
  (21 baris warning milik profil test, non-blocking).
- `cargo build --package asl-kernel-bin --target x86_64-unknown-none --release` → exit 0, **0 warning / 0 error**;
  recompilasi dari-nol yang sesungguhnya terjadi pada Build A.
- ELF **243.568 B** (Windows) - rujukan silang hosted CI Linux: 243.560 B.

## 5. Batas yang dinyatakan terang

1. Yang diukur adalah **reproducible PER-PLATFORM** pada satu host (Windows + xorriso cygwin). Delta antar-platform
   (ELF 8 B) didokumentasikan dan bukan target; pasangan Linux→Linux belum diukur karena CI hosted tidak dapat
   menjalankan langkah ISO (butuh xorriso cygwin, path `/cygdrive`, Limine lokal, dan QEMU).
2. Konstanta epoch baked membuat build default di host yang sama identik; bila operator ingin image mencerminkan
   waktu rilis, set `SOURCE_DATE_EPOCH` secara eksplisit - itu jalur pertama yang diuji.
3. `scripts/test_hv2_stack_overflow_qemu.ps1` punya panggilan xorriso sendiri (ISO tersuntik untuk uji negatif HV-2)
   dan **tidak** diubah di T1; jalur non-kanonik itu masih non-reproducible. Jalur kanonik = `build_kernel.ps1`
   yang dipakai `ci_local.ps1` langkah [3/4].
4. Salinan biner ISO yang diukur hidup di `logs/t1/` (ter-ignore `**/logs/`); bukti ter-commitnya adalah transkrip
   `Analysis/reports/iso_reproducible_t1_2026-10-21.log` yang memuat SHA256/ukuran/offset. Ini sama dengan tingkat
   bukti yang dinyatakan di T0 §5.1 (log mentah tanpa ekstrak ter-commit = tier 2), dan karenanya **tidak**
   diklaim sebagai tier 1.
5. Ditemukan dua drift alat yang hanya dicatat, tidak "direkonsiliasi": `limine.exe --version` melaporkan **8.7.0**
   (dokumen AUD-02 menyebut 8.4.1) dan toolchain lokal **rustc/cargo 1.99.0** (dokumen lama 1.86.0-nightly).
   Karena nol `.rs` berubah, angka gerbang tetap sah; klaim versi di dokumen lama tidak ditulis-ulang.
6. ISO pra-T1 yang identik dengan artefak rilis disalin ke `logs/pre_t1/` sebelum rebuild; SHA256 `faca92f4…`
   tetap sah sebagai hash rilis v1.1.0 di `Analysis/reports/release_v1.1.0_SHA256SUMS_2026-10-20.txt`. Nol artefak
   bukti lama dihapus atau diubah isinya.

## 6. Sinkron dokumen yang dilakukan commit T1 ini

| berkas | butir | dari | menjadi |
|---|---|---|---|
| `README.md` §6 catatan kebenaran artefak | status reproducible ISO | "build ISO **tidak byte-reproducible** … SHA256 berubah tiap rebuild" | annotasi `[PHASE-7 T1, 2026-10-21]`: per-platform kini reproducible (`9cd49028…` untuk dua build bersih); teks lama dipertahankan sebagai keadaan pra-T1 |
| `README.md` §7 | daftar realisasi | sampai T0 | bullet **Status PHASE-7 T1** (perubahan build script + dua akar nondeterminisme + hasil ukur + boot check) |
| `RELEASE_NOTES.md` §0.1 baris ISO, §0.2 butir keterbatasan, §0.5 baris ISO, §0.6 | "tidak byte-reproducible" dan daftar sasaran terbuka | seperti di atas | annotasi append-only + status T1 selesai; delta antar-platform 8 B dinyatakan bukan target |
| `docs/ROADMAP/AUDIT_ASL_AUD_03_ADVERSARIAL_PR2.md` | kesimpulan reproducible | temuan terbuka | catatan status append-only: akar (a) dan (b) tertutup untuk host Windows, bukti di laporan T1 |
| `Analysis/reports/QODER_FINAL_REPORT.md` | §2/§7/§8/§9 + verdict + keterbatasan | ISO tidak reproducible | baris **T1** di §9 + annotasi butir keterbatasan (teks lama tetap terbaca) |
| `docs/ROADMAP/HANDOFF_STATE.json` | `active_phase`, `next_immediate_step`, kunci `phase7_t1_state` baru, `timestamp` | T1 belum dimulai | T1 selesai, langkah berikutnya T2 (coverage) |
| `docs/ROADMAP/ACCOUNTABILITY_FAULT_LOG.json` | `fault_history` | 15 entri | **16** entri: ratifikasi keputusan pin GPT GUID + kategorisasi boot check (`ISO_REPRODUCIBILITY_GPT_GUID_PINNING_DECISION`, CLOSED); `violations_recorded` tetap **0** |

Bukti: `Analysis/reports/iso_reproducible_t1_2026-10-21.log` (123 baris ASCII) dan laporan ini.
Nol berkas `.rs`, nol `Cargo.toml`/`Cargo.lock`, nol `CHECKLIST_STATE.json`.

## 7. Kepatuhan G8

Commit T1 ini dibuat di atas tip `feature/phase7` `b9a4057` sebagai commit baru (nol amend/rebase/reset/force-push);
hash yang dikutip di dalam dokumen (`b9a4057`, `6623dd8`, `0a49ba7`) semuanya commit yang sudah ada. Run CI atas
commit T1 ini dilaporkan lewat check PR dan ekstrak log mentahnya masuk repo pada commit docs-only berikutnya
(pola anti-regres yang sama seperti T0 §6).
