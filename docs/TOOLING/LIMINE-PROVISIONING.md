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

> STATUS SEKARANG = BAGIAN 7. Isi bagian 2..5 di bawah adalah pengukuran jam
> 08:19-08:40Z yang disimpan apa adanya (append-only); jalur pin v8.7.0 sudah
> digantikan v12.9.4 dan blocker #50 tertutup. Jangan membaca bagian ini sebagai
> keadaan hari ini.

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

## 7. VARIAN A DIIIMPLEMENTASIKAN - pin pindah ke v12.9.4 (R3 BAGIAN 3, terukur 2026-10-24T12:24:20Z)

Bagian 1..6 di atas TIDAK ditulis ulang: nilainya dibiarkan apa adanya sebagai riwayat (append-only).
Yang berubah adalah STATUS, karena pemilik mencabut tahan dan memerintahkan varian A pada MASTER PROMPT R3
BAGIAN 3 ( otorisasi tertulis 2026-10-24; P2 butir 7 STATUS-BAGIAN-2 mengikat blocker #50 didahulukan).

Angka di bawah diukur oleh alat pada 2026-10-24T12:24:20Z; berkas bukti: `Analysis/reports/r3_bagian3-pin-limine_2026-10-24.txt`.

| butir | nilai terukur |
|---|---|
| `pinned_version` ledger | `12.9.4` (ditulis 2026-10-24T11:39:45Z, 19493 B, 395 baris LF, nonASCII 0) |
| jalur pin | `C:\asl\tools\limine\v12.9.4` |
| `limine.exe` | 96256 B, SHA256 `f68d3c2c96aca8883134016567b8ba644dfbada2ec16750d3009df288006e9ec` - cocok ledger `f68d3c2c96aca8883134016567b8ba644dfbada2ec16750d3009df288006e9ec` |
| `--version` | `Limine 12.9.4 ...` (dipotong sebelum kata 'Copyright'; petikan penuh ada di berkas bukti) |
| paket resmi `limine-binary.zip` | terunduh 2110373 B SHA256 `5f6916d5bcae8f13e0cabebb4d6f5ea0e82e505165fa899b38195868d7853655`; digest TERBIT API `5f6916d5bcae8f13e0cabebb4d6f5ea0e82e505165fa899b38195868d7853655` (2110373 B); cocok = YA |
| tanda tangan `.sig` | 566 B; `gpg --verify` exit 0 diukur 2026-10-24T12:13:21Z; `Good signature` = YA; kunci RSA 05D29860D0A0668AAEFB9D691F3C021BECA23821; peringatan `not certified with a trusted signature` dicatat (kunci tidak disertifikasi, tidak diimpor ke Web of Trust) |
| tanggal tanda tangan | Mon Oct 19 2026 23:10:35 SEAST (jam mesin lokal; gpg memakai zona yang sama) |
| published_at rilis v12.9.4 | 2026-10-19T16:10:50Z |
| payload `BOOTX64.EFI` | 385024 B, SHA256 `8a7214bfd8b9a044b780f1e71b2752dc12ecc25c06b9cf39ac6a0a341c802282`, cocok rujukan ledger = YA |
| payload `limine-bios-cd.bin` | 26624 B, SHA256 `05efcd37e04e18dc3e66ecc79c698aa95791f8bcd77abc2e0e994ef5e19a8b14`, cocok rujukan ledger = YA |
| payload `limine-bios.sys` | 339788 B, SHA256 `80468d0c0485447dc790d8be4c2834af3f581d3829867f66a273256ba91b65f8`, cocok rujukan ledger = YA |
| payload `limine-uefi-cd.bin` | 3145728 B, SHA256 `3496a6e858c2ea8fc90d1e208ffa883fe9b18d073d77aa18f146522a4d0e123c`, cocok rujukan ledger = YA |

### 7.1 Yang berubah pada rantai boot (terukur, bukan dugaan)

- `limine bios-install` pada v12.9.4 TIDAK lagi meninggalkan protective GPT seperti v8.7.0; ia mengonversi hybrid ISO ke layout MBR. Akibatnya `scripts/build_kernel.ps1` lama exit 1 dengan pesan 'GPT header not found after bios-install' - ini PERUBAHAN PERILAKU bootloader, bukan kerusakan naskah.
- Byte yang BERBEDA antara dua penguasaan bersih SEBELUM pin layout terpasang (salinan scratch `r3_b3_iso_A.iso` vs `r3_b3_iso_M1.iso`): 52 byte dalam 4 kelompok (0x1B8..0x1BB (4 B); 0x461210..0x46121F (16 B); 0x461290..0x46129F (16 B); 0x461310..0x46131F (16 B)); signature MBR 0x1B8 = dcf29bbf vs 93769861. Sesudah percabangan MBR dipasang di `build_kernel.ps1`, dua penguasaan bersih lagi (`P1` vs `P2`) berbeda 0 byte dan signature MBR = 79e881e1 vs 79e881e1. Naskah fail-closed bila tidak ada GPT header maupun signature MBR 0x55AA.
- `check_limine.ps1` exit 0: 8 baris COCOK = 4 nama payload diperiksa di 2 lokasi (jalur pin dan staging iso_root), ditambah utilitas host dan limine.conf.
- staging payload TIDAK lagi boleh manual: jalankan `scripts/stage_iso_root.ps1` setiap habis `cargo clean`.

### 7.2 Rantai boot diukur ulang SEPENUHNYA pada ISO v12.9.4

| pengukuran | hasil terukur |
|---|---|
| ISO (dua penguasaan bersih) | 4624384 B, SHA256 1569c17157b98c8aa29c9e9d18ccb6aeb931c883d1e1a25983864df4206c668c - identik byte: YA |
| boot QEMU headless | serial 2518 B, stderr 0 B, mata rantai M0:=ADA M1:=ADA M2:=ADA M3:=ADA M4:=ADA M5:=ADA M6:=ADA, prompt satu-sumber `ASL-KERNEL />_` ditemukan: YA |
| lantai memori ISO baru | 3072K (= -m 3072K/3M) GAGAL | 3136K (= -m 3136K) GAGAL | 3200K (= -m 3200K) LULUS | 3264K (= -m 3264K) LULUS | 3328K (= -m 3328K) LULUS | 3360K (= -m 3360K) LULUS | 3392K (= -m 3392K) LULUS |
| janji misi `-m 3M` | TETAP MERAH (deviasi tercatat, TIDAK dihijaukan) |
| lantai terkecil yang LULUS (ISO baru) | 3200K, sedangkan ISO lama (SHA256 `AA77806018D7...`, artefak logs/ram_floor_remeasure_36cf856.txt) baru lulus di 3360K dan gagal di 3328K - lantai TURUN 160 KiB. Ratchet regresi SENGAJA TIDAK dilonggarkan: tetap 3392K dan hasilnya LULUS. |
| ELF rilis | 312912 B, SHA256 `fb4eeb31e26b1cf9a289e4c697e6cea8230cd83a1ad45424cd2312ee5eee5d59` |

### 7.3 Perintah yang sekarang benar

```
powershell -File C:\asl\scripts\check_limine.ps1        # exit 0 = perkakas cocok ledger
powershell -File C:\asl\scripts\stage_iso_root.ps1      # wajib setiap habis cargo clean
powershell -File C:\asl\scripts\build_kernel.ps1 -Action iso
```

Bagian 4 (kandidat v12.9.4) kini TERPILIH; bagian 3 (v8.7.0 nol binari) tetap benar sebagai fakta riwayat, dan blok `aset_resmi_v8_7_0` serta `rujukan_payload_v8_7_0_selamat` TIDAK dihapus dari ledger.

