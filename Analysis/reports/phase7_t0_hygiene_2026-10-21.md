# PHASE-7 T0 - klasifikasi scratch `.log` di `Analysis/reports/` (2026-10-21)

Aturan misi PHASE-7 T0: berkas `.log` scratch tak-ter-track di direktori bukti harus **disahkan** (di-commit)
atau **dihapus**. Kebijakan yang dipakai = preseden CLOSEOUT T5: tidak ada informasi yang hilang tanpa bukti;
manifest SHA256 + ukuran + mtime diambil SEBELUM tindakan; penghapusan hanya untuk berkas yang (a) byte-identik
dengan artefak ter-commit (`cmp` = 0 byte berbeda), (b) seluruh baris isinya sudah tercakup korpus ter-commit
plus berkas yang justro disahkan pada langkah ini, atau (c) yang baris uniknya murni noise wrapper PowerShell
tanpa nilai verdict - dan itu dinyatakan terang, bukan diam-diam.

## 1. Manifest 13 berkas scratch sebelum tindakan

| berkas | ukuran | mtime | sha256 |
|---|---|---|---|
| `asl_vm_serial_d5.log` | 206 B | 2026-10-13 13:03:08 | `255cf291530c4e20a4070860fd177e8767f37826ac5e903aa6c9272add703fb5` |
| `ci_2026-10-20.log.step1.log` | 22910 B | 2026-10-20 03:12:00 | `a8820249f03e0f93cdc0778591d677390ffecb5c6e6310e623763fc2d30a7757` |
| `ci_2026-10-20.log.step2.log` | 428 B | 2026-10-20 03:12:00 | `1dbf61995fddaa40cb133d4f1f5aac1e70d8b1813f86326e397d6fbbbc6e1b29` |
| `ci_2026-10-20.log.step3_iso.log` | 1829 B | 2026-10-20 03:12:00 | `52a04c7a64f7a451e069361586fec18d6910635eb4f73e50162543c50fe0a8ec` |
| `qemu_ci_smoke_qemu_stderr.log` | 0 B | 2026-10-20 03:12:01 | `e3b0c44298fc1c149afbf4c8996fb92427ae41e4649b934ca495991b7852b855` |
| `qemu_hv2_qemu_stderr.log` | 0 B | 2026-10-20 03:01:51 | `e3b0c44298fc1c149afbf4c8996fb92427ae41e4649b934ca495991b7852b855` |
| `qemu_hv2_serial.log` | 2560 B | 2026-10-20 03:01:53 | `eb016321c91ed08bbc28628f4a797082a06613e6d59aa01898f6ed2278a56379` |
| `qemu_nominal_regression_serial.log` | 2511 B | 2026-10-20 03:02:28 | `f47441b1e651b75c94a5fa5c8351eb9b4691c0d30c514ce1d537c86a39addcc2` |
| `qemu_qm01_vtd_qemu_stderr.log` | 144 B | 2026-10-20 02:34:19 | `15d70544c0382cc9653bdb58149f2430648c47a6e1d9befbcf08dd645dad10e1` |
| `qemu_qm01_vtd_serial.log` | 3580 B | 2026-10-20 02:34:19 | `0630c675404faf27a1cf853d2a4f3803df10d7dcbd896cc5d3c735bb72fa967c` |
| `qemu_serial.log` | 936 B | 2026-10-16 21:50:59 | `508f8d17a0f9ac274ee3b0b8b88ae19606af195e8dbc94c189b164decc4b86e2` |
| `qemu_serial_check.log` | 401 B | 2026-10-16 12:28:34 | `572ecd2a3a408b602e8bda2c1f4792d7bf0aff95c743f0a00dff60427e764066` |
| `vbox_check.log` | 759 B | 2026-10-15 22:40:07 | `12eb2f1cc4bb0a3564b988ccc7cbd0496b6642a93b49b2a921290cb796211b28` |

## 2. Kelas A - dupleks byte-identik dengan artefak ter-commit -> DIHAPUS (7 berkas)

- `qemu_ci_smoke_qemu_stderr.log` (0 B) byte-identik dengan `hv2_qemu_stderr_2026-10-20.log` (`cmp` exit=0, 0 byte berbeda). Harness penulisnya memakai path tetap
  yang sama, jadi berkas ini muncul ulang di setiap run - ia tidak menyimpan informasi unik.
- `qemu_hv2_qemu_stderr.log` (0 B) byte-identik dengan `hv2_qemu_stderr_2026-10-20.log` (`cmp` exit=0, 0 byte berbeda). Harness penulisnya memakai path tetap
  yang sama, jadi berkas ini muncul ulang di setiap run - ia tidak menyimpan informasi unik.
- `qemu_hv2_serial.log` (2560 B) byte-identik dengan `hv2_stack_overflow_serial_2026-10-20.log` (`cmp` exit=0, 0 byte berbeda). Harness penulisnya memakai path tetap
  yang sama, jadi berkas ini muncul ulang di setiap run - ia tidak menyimpan informasi unik.
- `qemu_nominal_regression_serial.log` (2511 B) byte-identik dengan `g9_acceptance_serial_2026-10-20.log`, `nominal_regression_serial_2026-10-20.log`, `qemu_ci_smoke_serial.log` (`cmp` exit=0, 0 byte berbeda). Harness penulisnya memakai path tetap
  yang sama, jadi berkas ini muncul ulang di setiap run - ia tidak menyimpan informasi unik.
- `qemu_qm01_vtd_qemu_stderr.log` (144 B) byte-identik dengan `qm01_vtd_qemu_stderr_2026-10-20.log` (`cmp` exit=0, 0 byte berbeda). Harness penulisnya memakai path tetap
  yang sama, jadi berkas ini muncul ulang di setiap run - ia tidak menyimpan informasi unik.
- `qemu_qm01_vtd_serial.log` (3580 B) byte-identik dengan `qm01_vtd_serial_2026-10-20.log` (`cmp` exit=0, 0 byte berbeda). Harness penulisnya memakai path tetap
  yang sama, jadi berkas ini muncul ulang di setiap run - ia tidak menyimpan informasi unik.
- `vbox_check.log` (759 B) byte-identik dengan `asl_vm_serial.log` (`cmp` exit=0, 0 byte berbeda). Harness penulisnya memakai path tetap
  yang sama, jadi berkas ini muncul ulang di setiap run - ia tidak menyimpan informasi unik.

## 3. Kelas B - 0 baris di luar korpus ter-commit/berkas yang disahkan -> DIHAPUS (1 berkas)

- `qemu_serial_check.log` - 8 baris berisi, seluruhnya sudah ada di artefak ter-commit atau di berkas Kelas C.

## 4. Kelas C - punya baris unik -> DISAHKAN / di-commit (4 berkas)

- `asl_vm_serial_d5.log` (206 B) - fragmen 3 baris dengan varian teks M1 tanpa penanda profil (`[ASL-LAUNCHER] M1: Entered Limine Entry Stub via Assembly Trampoline.`); varian string itu tidak terdokumentasi di tempat lain, ukuran 206 B - lebih murah disimpan daripada hilang.
- `ci_2026-10-20.log.step1.log` (22910 B) - transkrip `cargo test` host 363 baris: SATU-SATUNYA inventaris **nama fungsi test** per suite yang tersisa (artefak kanonik `ci_2026-10-20.log` hanya memuat angka per suite). Dibutuhkan sebagai baseline PHASE-7 T3 (rename M-1) supaya perubahan nama bisa diaudit baris-per-baris. Wrapper PowerShell dibiarkan utuh: keluaran mentah, bukan laporan.
- `ci_2026-10-20.log.step3_iso.log` (1829 B) - transkrip langkah ISO pra-T1 (`GNU xorriso 1.5.2`, status drive/media, path `/cygdrive/c/asl/asl-kernel/target/asl-core-kernel.iso`). Jadi baseline pembanding T1 (ISO reproducible): perbedaan keluaran xorriso pra/post-SOURCE_DATE_EPOCH dapat ditunjuk ke berkas ini.
- `qemu_serial.log` (936 B) - cold-boot QEMU penuh sampai `M6` plus blok introspeksi request Limine (`-> Base Revision unsupported or unchanged.`, `-> Memmap response NULL.`, `-> HHDM response NULL.`, `-> Framebuffer response NULL.`) yang tidak ada di artefak ter-commit mana pun; penanda `profile=release` membuktikan biner release riil.

## 5. Kelas D - baris unik murni noise wrapper, hilang dengan alasan tulis (1 berkas)

- `ci_2026-10-20.log.step2.log` (6 baris berisi): 6 baris di luar korpus, seluruhnya blok PowerShell `NativeCommandError`/`CategoryInfo`/`FullyQualifiedErrorId` dan baris echo command-nya di sekitar keluaran `Finished ... profile [optimized] target(s) in 0.04s` (build ter-cache). Tidak ada verdict kernel atau CI di dalamnya. Keputusan tidak men-stage transkrip per-langkah CI sudah dinyatakan di `QODER_EXECUTION_LOG.md` baris 498 dan 544; sha256 berkas tetap tercatat di §1 sehingga kehilangan dapat dideteksi, bukan diam-diam.
  baris unik (diantaranya): `cargo.exe :     Finished `release` profile [optimized] target(s) in 0.04s`

## 6. Eksekusi dan hasil
- Dihapus 9 berkas (A=7, B=1, D=1). Disahkan 4 berkas (Kelas C).
- Sisa `.log` tak-ter-track setelah penghapusan = **4** berkas, yaitu tepat berkas Kelas C yang di-stage
  pada commit T0 ini (``asl_vm_serial_d5.log`, `ci_2026-10-20.log.step1.log`, `ci_2026-10-20.log.step3_iso.log`, `qemu_serial.log``); angka ini terukur SEBELUM `git add` dijalankan.
- Berkas **ter-track** yang dihapus/dipindah/diubah isinya: **0**. Nol bukti historis hilang; artefak GERBANG AKHIR dan CLOSEOUT tetap utuh.
- Nama berkas Kelas C TIDAK diubah (byte dan nama asli dipertahankan) agar bisa dicocokkan dengan rujukan historis di `QODER_EXECUTION_LOG.md`; statusnya berubah dari 'sengaja tidak di-stage' menjadi 'di-stage sebagai baseline PHASE-7' - itu perubahan keputusan, bukan koreksi bukti.
