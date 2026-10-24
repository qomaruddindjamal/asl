# Provisioning perkakas Limine (K-5) - versi dikunci + checksum

Satu-sumber angka = `docs/TOOLING/LIMINE-PIN.json` (`schema_version` 1). Berkas ini
hanya menjelaskan; nilai hash/ukuran/waktu dibaca dari ledger dan ledger dibaca dari
pengukuran alat. Ledger ditulis 2026-10-24T08:40:09Z (UTC, jam mesin). Kelas bukti: VERIFIED-ON-HOST.

## 1. Mengapa berkas ini ada

Blocker #50 dan #54 punya satu akar yang sama: perkakas host `limine.exe` dipasang
di dalam `asl-kernel/target/` dan TIDAK PERNAH dilacak repository, sehingga
`cargo clean` memusnahkannya dua kali dan tidak ada catatan versi, URL, maupun
hash yang memungkinkan pembangunan ulang. Ledger ini menutup akar itu: versi
dikunci, jalur di luar `target/`, hash resmi terukur, dan alat pemeriksaan.

Jalur yang kini dicari build (ubah di sub-komit 2, `76fe9a1`):

```
C:\asl\tools\limine\v8.7.0\limine.exe
```

dan dipanggil oleh `scripts/build_kernel.ps1` (aksi `iso`, langkah 2 dan 5) serta
`scripts/test_hv2_stack_overflow_qemu.ps1` baris 28.

## 2. Status sekarang (terukur, bukan harapan)

- Jalur pin `C:\asl\tools\limine\v8.7.0\limine.exe` = TIDAK ADA. Build ISO tetap
  fail-closed: `scripts\build_kernel.ps1 -Action iso` exit 1 pada baris 104
  (terukur 2026-10-24T08:19:24Z, lihat `Analysis/reports/r1_k5_sub2-jalur-alat_2026-10-24.txt`).
- `target/iso_root/boot/limine` = TIDAK ADA saat ledger ditulis, karena staging ikut
  musnah bersama `target/`; ia harus diisi ulang dari payload (lihat bagian 5).
- K-2 (ISO + bukti VM) karena itu MASIH TERBLOKIR. Tidak ada ISO karangan.

## 3. Fakta yang mengubah rencana: v8.7.0 resmi tidak punya paket binari

Perintah pemilik mengunci "Limine v8.7.0 resmi". Diukur terhadap GitHub API rilis
resmi `Limine-Bootloader/Limine`:

| yang diukur | hasil |
|---|---|
| aset rilis v8.7.0 | ChangeLog, limine-8.7.0.tar.bz2, limine-8.7.0.tar.gz, limine-8.7.0.tar.lz, limine-8.7.0.tar.xz, limine-8.7.0.tar.zst |
| aset ber-kata-kunci `binary` pada v8.7.0 | NOL |
| rilis v8.x terdaftar yang diperiksa | 8, yang punya binari: NOL |
| rilis PALING TUA dengan `limine-binary.zip` | v12.6.1 (diterbitkan 2026-08-24T21:43:46Z) |
| total rilis terunduh untuk pemeriksaan | 408, dengan binari: 8 |
| SHA256 `limine-8.7.0.tar.gz` (sumber resmi, diunduh untuk pemeriksaan) | `a70a11f69b4ca5d85d11efca6f273eb26bfda0d22ab49f4bd0a15d7fafe0d047` (603144 B) |

Artinya: pada versi pin, yang tersedia secara resmi HANYA sumber. `limine.exe` dan
payload harus DIBANGUN, dan mesin ini terukur tidak punya perkakas build-nya
(tidak ada `gcc`/`clang`/`make`/`mingw32-make` di PATH, WSL ada tetapi nol distro,
rustup hanya MSVC). Build system v8.7.0 memakai autotools (`configure`,
`GNUmakefile.in` dengan `CC=@CC@`) dan butuh toolchain freestanding lintas
arsitektur untuk payload.

## 4. Kandidat resmi yang lebih baru: BEDA, dan itu dicatat bukan dipaksakan

`v12.9.4` menyediakan `limine-binary.zip` berisi `limine-tool-windows-x86/limine.exe`
(96256 B, SHA256 `f68d3c2c96aca8883134016567b8ba644dfbada2ec16750d3009df288006e9ec`). Paketnya sendiri SHA256 `5f6916d5bcae8f13e0cabebb4d6f5ea0e82e505165fa899b38195868d7853655` (2110373 B, unduh 2026-10-24T08:17:54Z) dan punya
`.sig` (566 B, SHA256 `fcadc3d33339a7f6d7d51ff3ae05212ad9435bcdb32c2f99d69240ee7f86b4f3`) yang TIDAK dapat diverifikasi di mesin ini karena kunci penanda
tidak ada di ring lokal (`gpg --verify` exit 2 -> "Can't check signature: No public key";
kunci RSA `05D29860D0A0668AAEFB9D691F3C021BECA23821`, tanggal tanda tangan 2026-10-19T23:10:35
menurut jam mesin lokal). Pemeriksaan itu membuat ring `~/.gnupg`; tidak ada kunci diimpor.

Pembanding byte yang diminta pemilik - payload kandidat lawan salinan v8.7.0 yang
selamat di `logs/iso_recovery/boot/limine`:

| berkas | rujukan v8.7.0 | sha256 rujukan | kandidat v12.9.4 | sha256 kandidat | delta ukuran | identik? |
|---|---|---|---|---|---|---|
| `BOOTX64.EFI` | 249856 B | 51ca2b20ed14bbc6... | 385024 B | 8a7214bfd8b9a044... | +135168 B | TIDAK |
| `limine-bios-cd.bin` | 24576 B | a94c1cf610c67f13... | 26624 B | 05efcd37e04e18dc... | +2048 B | TIDAK |
| `limine-bios.sys` | 216644 B | b94bd7ae8a7faf58... | 339788 B | 80468d0c0485447d... | +123144 B | TIDAK |
| `limine-uefi-cd.bin` | 2949120 B | 0ea0aa408c464014... | 3145728 B | 3496a6e858c2ea8f... | +196608 B | TIDAK |

Keempatnya BEDA hash dan BEDA ukuran. Sesuai perintah "bila beda, catat - jangan
dipaksakan", kandidat TIDAK dipasang ke jalur pin dan TIDAK dipakai build. Unduhan
hanya masuk `C:\asl\tools\limine\_inspeksi\` (di-ignored oleh `.gitignore` baris
154-155) untuk pengukuran.

## 5. Checklist provisioning (urutan wajib)

1. Pemilik memilih satu jalan (semuanya sah, konsekuensinya di ledger):
   - **A** pindah pin ke `v12.9.4` (binari resmi; ISO dan semua dokumen versi
     berubah; wajib jendela bukti VM),
   - **B** tetap `v8.7.0` dan bangun dari sumber resmi (pemilik memasang
     perkakas C/autotools atau distro WSL lebih dulu),
   - **C** pemilik memasang sendiri `limine.exe` v8.7.0 seperti sebelumnya dan
     mencatat asal-usulnya.
2. Letakkan hasil di `C:\asl\tools\limine\<versi>\` dengan `limine.exe` di puncak,
   dan payload (`BOOTX64.EFI`, `limine-bios-cd.bin`, `limine-bios.sys`,
   `limine-uefi-cd.bin`, `limine.conf`) dalam direktori yang sama.
3. Salin PAYLOAD (bukan `limine.exe`) ke staging ISO:
   `C:\asl\asl-kernel\target\iso_root\boot\limine\`. Langkah ini WAJIB diulang
   setiap habis `cargo clean`.
4. Perbarui ledger: `pinned_version`, `install_root`, dan isi blok hash sebelum
   artefak dipakai build.
5. Jalankan pemeriksaan:
   ```
   powershell -File C:\asl\scripts\check_limine.ps1
   ```
   Keluaran = daftar per berkas (ADA/TIDAK, ukuran, SHA256, cocok/tidak dengan
   ledger) dan exit code NONZERO bila ada yang tidak cocok. Fail-closed.
6. Baru setelah itu: `scripts\build_kernel.ps1 -Action iso`, lalu K-2 (QEMU,
   bukti VM, screendump bila mampu).

## 6. Aturan yang dijaga

- Binari perkakas TIDAK di-commit (`tools/` ter-ignore). Repo menyimpan VERSI,
  JALUR, dan HASH - bukan artefaknya.
- Jangan menaruh perkakas di `target/`: `cargo clean` menghapus seluruh isinya
  (akar fault log #50 dan #54).
- Ledger dan dokumen ini berubah hanya lewat commit append-only; nilai yang sudah
  ter-commit tidak ditulis ulang.
- Empat rujukan dokumen lama ke `target\limine_v8_binary\limine.exe` masih ada
  (README.md:138, docs/CANONICAL/DEVELOPER_ONBOARDING.md:72,
  docs/PANDUAN/MULAI-CEPAT.md:18, docs/PANDUAN/REFERENSI-PERINTAH.md:495) dan
  sengaja tidak disentuh di unit ini: README adalah wilayah pemilik di `main` dan
  cabang R1 dipertahankan nol konflik.

Rujukan rantai: `4335e41` (pulih + blocker terukur) - `65f6137` (utang G8 ditutup) -
`76fe9a1` (jalur alat pindah) - sub-komit ini (ledger + alat periksa).
