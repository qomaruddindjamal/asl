# MULAI CEPAT ASL CORE KERNEL v1.0.2

## Pasal 1 - Tujuan

Satu halaman untuk sampai pada prompt `ASL-KERNEL />_ ` dari repositori bersih:
uji unit di host, bangun ELF release, bangun ISO, lalu boot di QEMU. Setiap
perintah di bawah adalah perintah yang benar-benar dipakai pipeline repo ini.

## Pasal 2 - Prasyarat

- PowerShell 5.1 (`powershell.exe`) di host build. `pwsh` tidak diasumsikan ada.
- Rust dengan toolchain MSVC untuk host build, dan target bare-metal
  `x86_64-unknown-none`. Versi yang terukur saat panduan ini ditulis:
  `rustc 1.99.0 (b940084d7 2026-09-28)` (`VERIFIED-ON-HOST`).
- QEMU di `C:\Program Files\qemu\qemu-system-x86_64.exe` (lokasi ini yang dibaca
  skrip repo).
- `xorriso` dan `limine` untuk langkah ISO. `limine` dibaca dari
  `asl-kernel\target\limine_v8_binary\limine.exe`; bila berkas itu tidak ada,
  langkah ISO berhenti dengan galat, bukan menghasilkan ISO setengah jadi.
- Ruang kerja kargo: semua perintah `cargo` dijalankan dari `asl-kernel`.

## Pasal 3 - Empat langkah

### [1] Uji unit di host

```
cd asl-kernel
cargo test --workspace --lib
```

Hasil terukur pada tree rilis ini: **196 lulus / 0 gagal** (`VERIFIED-ON-HOST`,
diukur lagi setiap tahap; baseline misi 172 dan tidak boleh turun).

### [2] Bangun ELF bare-metal release

```
cargo build --package asl-kernel-bin --target x86_64-unknown-none --release
```

Harus **0 error dan 0 warning**. Keluaran:
`asl-kernel\target\x86_64-unknown-none\release\asl-kernel`, 325.528 byte
(`VERIFIED-ON-HOST`).

Perintah di atas tidak menyetel `RUSTFLAGS`. Itu disengaja: `rustflags` untuk
target ini hidup di `asl-kernel\.cargo\config.toml` dan membawa skrip tautan
(`-Tlinker.ld`). `RUSTFLAGS` dari lingkungan menimpa konfigurasi target itu,
sehingga link gagal (`undefined symbol: __asl_secondary_start`). Bila shell
develop Anda sudah menyetel `RUSTFLAGS`, kosongkan untuk perintah di atas:
`env -u RUSTFLAGS cargo build ...` di Git Bash.

### [3] Bangun ISO hibrida

```
powershell -ExecutionPolicy Bypass -File scripts\build_kernel.ps1 -Action iso
```

Keluaran: `asl-kernel\target\asl-core-kernel.iso`. Langkah ini build release
lagi, mem-master ISO El Torito BIOS+UEFI dengan `xorriso -as mkisofs`,
memasang boot stage dua lewat `limine bios-install`, lalu menstabilkan tanggal
volume dari `SOURCE_DATE_EPOCH` (bawaan tetap: 1767225600) dan GUID GPT yang
sebelumnya membuat SHA256 berubah tiap build walau isi ELF identik.

### [4] Boot di QEMU

```
powershell -ExecutionPolicy Bypass -File scripts\build_kernel.ps1 -Action run-qemu
```

Perintah di atas memakai `-serial stdio -display none -m 64M`. Untuk console
interaktif dari jendela terminal, jalankan QEMU langsung dengan argumen mesin yang
sama dengan yang dipakai pipeline (`-machine q35 -vga std`, ISO sebagai `-cdrom`);
pola ini yang dipakai sesi shell terukur pada tahap P3 (`VERIFIED-ON-VM`):

```
"C:\Program Files\qemu\qemu-system-x86_64.exe" -machine q35 -vga std -m 256M `
  -cdrom asl-kernel\target\asl-core-kernel.iso -serial mon:stdio
```

Urutan yang harus muncul di serial: `[ASL-LAUNCHER] M1`, `M2`, `M3`, lalu
`[ASL-CORE] M0`, `M0+`, `M4`, `M4+`, `M5`, `M6: Kernel Cold Boot Complete`,
banner, dan prompt `ASL-KERNEL />_ ` (akar RAMDISK). Prompt membawa posisi
direktori kerja, jadi sesudah `cd log` ia berubah menjadi `ASL-KERNEL /log>_ `.
Daftar arti tiap tahap ada di
`docs/ROADMAP/PETA-KANONIK.md`.

## Pasal 4 - Lima menit pertama di shell

Perintah yang tersedia dibaca dari dalam shell sendiri, bukan dari hafalan:

```
ASL-KERNEL />_ help
ASL-KERNEL />_ help write-file
ASL-KERNEL />_ where
ASL-KERNEL />_ create-folder log
ASL-KERNEL />_ cd log
ASL-KERNEL /log>_ create-file note.txt
ASL-KERNEL /log>_ write-file note.txt halo dunia
ASL-KERNEL /log>_ cat note.txt
ASL-KERNEL /log>_ up
ASL-KERNEL />_ status
```

Ruang kerja (`cd`, `ls`, `cat`) hanya melihat RAMDISK volatil; `up` menaiki satu
tingkat dan tetap di akar bila sudah di akar. Isi hilang ketika mesin dimatikan.
Bentuk lengkap dan janji setiap perintah:
`docs/PANDUAN/REFERENSI-PERINTAH.md`.

## Pasal 5 - Pipeline lengkap sekali jalan

```
powershell -ExecutionPolicy Bypass -File scripts\ci_local.ps1
```

Lima langkah fail-closed: `[1/5]` uji host, `[2/5]` build release, `[3/5]` smoke
boot QEMU (harus capai M6 + prompt), `[4/5]` gerbang regresi memori boot, `[5/5]`
ringkasan. Langkah berikutnya tidak dijalankan bila langkah sebelumnya gagal.
Sebelum menjalankan ini, baca Pasal 6.

## Pasal 6 - Yang harus diketahui sebelum menulis artefak bukti

- `scripts/ci_local.ps1` menulis lognya ke `Analysis\reports` dengan nama tetap.
  Berkas dengan nama itu bisa menjadi bukti yang sudah ter-commit, sehingga
  menjalankannya tanpa persiapan menimpa artefak bukti. Untuk pengambilan bukti
  baru, pakai skrip gerbang (`scripts\test_ram_floor_3mb.ps1`) yang menulis ke
  `logs\` dengan nama bercap-waktu, atau salin dulu berkas yang mau dilindungi.
- `logs\` tidak masuk git. Bukti yang harus bertahan hidup commit ditulis ke
  `Analysis\reports`.
- QEMU dijalankan headless, karena itu tidak ada tangkapan layar yang bisa
  dikutip sebagai bukti; bukti boot adalah berkas log serial.
- Gerbang memori `-m 3M` berstatus merah pada tree ini dan pipeline
  menyatakannya sebagai deviasi terbuka, bukan kelulusan. Lihat
  `docs/PANDUAN/KONFIGURASI-MEMORI.md`.
