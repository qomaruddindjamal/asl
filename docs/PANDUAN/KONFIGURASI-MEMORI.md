# KONFIGURASI MEMORI ASL CORE KERNEL v1.0.2

## Pasal 1 - Tujuan

Menjawab dua pertanyaan praktis: berapa RAM yang benar-benar dibutuhkan supaya
kernel sampai ke prompt, dan bagaimana mengubah konfigurasi tanpa merusak janji
memori. Semua angka di dokumen ini adalah hasil ukur, bukan tafsir.

## Pasal 2 - Yang diminta gambar kernel

Diukur dari header program ELF release pada tree rilis (`VERIFIED-ON-HOST`):

| Segmen | Alamat virtual | Byte dari berkas | Byte di RAM |
| --- | --- | --- | --- |
| PT_LOAD 0 | `0xFFFFFFFF80100000` | 448 | 448 |
| PT_LOAD 1 (kode) | `0xFFFFFFFF80101000` | 205.103 | 205.103 |
| PT_LOAD 2 (rodata) | `0xFFFFFFFF80134000` | 28.619 | 28.619 |
| PT_LOAD 3 (data + `.bss`) | `0xFFFFFFFF8013B000` | 49.152 | 893.056 |
| Jumlah | | 283.322 | 1.127.226 |

Selisih 843.904 byte pada segmen terakhir adalah `.bss` bertipe `SHT_NOBITS`:
tidak ada isinya di dalam berkas, bootloader Limine yang menolkan. Karena itu
menambah atau mengurangi slot RAMDISK hampir tidak mengubah ukuran berkas ELF,
tetapi mengubah RAM yang dipakai saat boot.

RAMDISK menempati 262.144 byte dari `.bss` itu. Satu-sumbernya di
`asl-kernel/asl-storage/src/ramdisk.rs`:

- `RAMDISK_DEFAULT_NODES = 32` slot,
- `RAMDISK_FILE_SLOT_BYTES = 8192` byte per slot,
- `RAMDISK_CAPACITY_DEFAULT_BYTES = 262144` (256 KiB),
- `RAMDISK_CAPACITY_MIN_BYTES = 256 KiB`, `RAMDISK_CAPACITY_MAX_BYTES = 512 KiB`.

Batas heap 16 MB adalah aturan misi v1.0.2 dan tidak diubah pada rilis ini.

## Pasal 3 - Lantai boot terukur

Metode: `scripts/test_ram_floor_3mb.ps1`. QEMU `q35`, `-cdrom` ISO rilis, serial
ditulis ke berkas, dan sebuah ambang dinyatakan LULUS bila berkas serial memuat
`M6: Kernel Cold Boot Complete` dan prompt shell yang dibaca dari konstanta
`SHELL_PROMPT` (bukan teks yang disalin ke skrip).

Hasil pada ISO P3/P4, SHA256 `756b87b057ec2f7be0617da5b1ca463eb774e895b8ad44c8f45bf7dea367f1e8`
(`VERIFIED-ON-VM`, 2026-10-21):

| `-m` | Hasil |
| --- | --- |
| 3072K (tepat 3 MB) | GAGAL capai prompt |
| 3328K | GAGAL |
| 3360K | LULUS |
| 3392K | LULUS |
| 3456K | LULUS |

Lantai boot karena itu duduk di rentang `(3328K, 3360K]`.

Pembanding historis, ISO tahap P2 pada branch yang sama sebelum RAMDISK ada:
lulus di 3072K dan gagal di 2944K. Artinya sisa terhadap janji 3 MB sudah kurang
dari 128 KiB bahkan sebelum fitur shell berkas dipasang. Atribusi penuh
dicatat di `docs/ROADMAP/ACCOUNTABILITY_FAULT_LOG.json` entri ke-23: P3
menambah sekitar 251 KiB `.bss` (RAMDISK 256 KiB) dan sekitar 74 KiB gambar
(kode shell, registry, jembatan VFS).

Angka-angka ini terikat pada sidik jari build-nya. Suntingan sekecil apa pun pada
sumber mengubah SHA256 berkas ELF dan ISO, jadi tahap rilis (P8/P9) mengukur
ulang lantai boot pada build yang akan diterbitkan, bukan memakai ulang tabel di
atas.

## Pasal 4 - Status janji `-m 3M`

Janji Bagian 5 misi v1.0.2 adalah boot pada `qemu -m 3M`. Janji Bagian 1.7 adalah
RAMDISK default 512 KiB dengan minimum 256 KiB bila bertabrakan. Putusan pra-jadi
Bagian 6 untuk tabrakan ini (turunkan default ke minimum) sudah diterapkan persis,
dan setelah itu `-m 3M` tetap gagal. Untuk memenuhi 3 MB pada tree ini perlu
memotong footprint statis yang sudah ada sebelum misi ini, dan itu di luar cakupan
suntingan yang diizinkan.

Konsekuensi yang diterima, bukan yang disembunyikan:

- Gerbang memori berstatus **MERAH** dan masuk `ACCOUNTABILITY_FAULT_LOG` dengan
  `result: "OPEN"`.
- Pipeline lokal tidak dibungkam: `ci_local.ps1` langkah `[4/5]` meneruskan
  pipeline dengan banner deviasi, dan ringkasan menyebut deviasi terbuka.
- Kode keluar `scripts/test_ram_floor_3mb.ps1`:
  - `0` - janji misi dan ratchet dua-duanya boot;
  - `1` - ratchet gagal, berarti ada pertumbuhan footprint baru = REGRESI, MERAH;
  - `2` - ratchet lulus tetapi `-m 3M` gagal = DEVIASI TERCATAT. Status merah ini
    tidak pernah ditulis sebagai kelulusan.

## Pasal 5 - Aturan bila akan mengubah konfigurasi memori

1. Naikkan `RAMDISK_DEFAULT_NODES` atau `RAMDISK_FILE_SLOT_BYTES` hanya bersama
   pengukuran ulang lantai boot pada ISO hasil build tree itu. Ratchet 3392K di
   dalam skrip ikut dinaikkan hanya bila lantai baru memang sudah terukur.
2. Jangan memotong footprint statis warisan (IDT/TSS, tabel VTD, stack 512 KB,
   penyangga shell) untuk mengejar angka. Itu objek misi lain dan harus lewat
   keputusan pemilik, bukan lewat gerbang yang dilemahkan.
3. Jangan memakai `scripts/g9_acceptance_run.ps1` lama untuk bukti baru (namanya
   keluaran berbasis tanggal) dan jangan menjalankan `scripts/ci_local.ps1` di
   atas artefak bukti yang sudah ter-commit.
4. Setiap angka yang dikutip di dokumen ini wajib menyebut sidik jari build atau
   berkas log tempat angka itu diukur.

## Pasal 6 - Cara mengukur ulang

```
powershell -NoProfile -ExecutionPolicy Bypass -File scripts\test_ram_floor_3mb.ps1
```

Argumen yang tersedia: `-MissionRam` (bawaan `3M`), `-RatchetRam` (bawaan
`3392K`), `-IsoPath`, `-QemuExe`, `-OutDir`, `-TimeoutSec` (bawaan 120). Keluaran
serial dan stderr bercap-waktu ditulis ke `logs\` (scratch, tidak masuk git).
Untuk memetakan lantai, jalankan skrip beberapa kali dengan `-RatchetRam` berbeda
misalnya `3328K` lalu `3360K`; setiap titik ukur dicatat beserta SHA256 ISO-nya.
