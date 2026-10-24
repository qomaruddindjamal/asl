# ASL Core Kernel v1.0.2: shell Cockpit + rantai boot Limine ter-pin + bukti VM

**VERIFIED-ON-HARDWARE: 0** (jendela fisik ditunda, otorisasi tertulis pemilik 2026-10-24 --
pemilik bepergian). Rilis ini diterbitkan dari rantai R3 yang berjalan penuh tanpa
konfirmasi, dengan setiap deviasi dicatat di `docs/ROADMAP/ACCOUNTABILITY_FAULT_LOG.json`.

Dipotong pada tag annotated `v1.0.2` objek `79d581f8291650679693e6014e0ce4b73b54faf7` -> commit `700481c22dc98b92be5bbd2059c54ea5bf97feb3`,
yaitu tip yang lulus BAGIAN 6 (jendela penerimaan VM tak-disaksikan). Tip `main` setelah
merge PR #7 adalah `c6e4c01788789e3126fa3488abb7696f72d0f485`; selisih tree terhadap basis rilis terukur hanya `README.md` dengan
nol berkas `.rs`, dan kedua commit hijau hosted CI.

## 1. Angka gerbang final

Angka di bawah dibaca dari berkas ekstrak hosted CI yang ter-commit
(`Analysis/reports/ci_hosted_<run-id>_2026-10-24.log`), bukan dari hitungan lokal.

| Metrik | Nilai | Cara ukur |
|---|---|---|
| suite host test | 30 | 4 run ekstrak, himpunan nilai identik |
| test lulus | 262 | 4 run ekstrak, himpunan nilai identik |
| test gagal | 0 | 4 run ekstrak, himpunan nilai identik |
| per-test terhitung | 262 | 4 run ekstrak, himpunan nilai identik |
| baris `warning` langkah [1/2] | 19 | 4 run ekstrak, himpunan nilai identik |
| baris `error` langkah [1/2] | 0 | 4 run ekstrak, himpunan nilai identik |
| baris `warning` langkah [2/2] | 0 | 4 run ekstrak, himpunan nilai identik |
| baris `error` langkah [2/2] | 0 | 4 run ekstrak, himpunan nilai identik |
| baris `Compiling` langkah [2/2] | 13 | 4 run ekstrak, himpunan nilai identik |
| footprint ELF dilaporkan runner (B) | 312968 | 4 run ekstrak, himpunan nilai identik |
| Footprint ELF di host (B) | 312912 | `sha256sum` + `ls` pada `target/x86_64-unknown-none/release/asl-kernel` |

Empat run hijau yang menjadi dasar tabel ini:
- `40109001959` event `push` di `700481c` (completed/success) -> ekstrak `Analysis/reports/ci_hosted_40109001959_2026-10-24.log` 11192 B SHA256 `9e1a48518d19571c3a56bd4188d406ece328f42445f39163d49a1d7f0a42c7b9`
- `40109006618` event `pull_request` di `700481c` (completed/success) -> ekstrak `Analysis/reports/ci_hosted_40109006618_2026-10-24.log` 11208 B SHA256 `c7ef1ea88f47bb0026374c997e6d4c2bd9abad5c996749aef02ada0d6e5d50de`
- `40109170099` event `push` di `4aa17a3` (completed/success) -> ekstrak `Analysis/reports/ci_hosted_40109170099_2026-10-24.log` 11192 B SHA256 `a3b9cf27513f323065549a9081337aa12014141b16a9fc2d0a747830e060a266`
- `40109173018` event `pull_request` di `4aa17a3` (completed/success) -> ekstrak `Analysis/reports/ci_hosted_40109173018_2026-10-24.log` 11208 B SHA256 `c00deef6cd5c50e75035209de3605622eab936d24fb2103dc983d29e013b456e`

Himpunan nilai keempat run identik = **nol regresi** di antara dua event pemicu dan dua
commit (`700481c`, `4aa17a3`). Selisih ELF runner vs host `56 B` dibiarkan terbuka, tidak
dicocokkan paksa (runner Linux, host Windows).

## 2. Tabel kelas bukti

| Kelas | Status | Bukti |
|---|---|---|
| VERIFIED-ON-HOST | Berlaku untuk seluruh angka sisi host dan angka jendela | berkas bukti `r3_bagian*_2026-10-24.txt` dan artefak `p8_unattended_*` |
| VERIFIED-ON-CI | Berlaku untuk langkah [1/2] dan [2/2] pipeline minimal | empat ekstrak hosted CI di atas |
| VERIFIED-ON-CI-API | Berlaku untuk metadata run dan (setelah unggahan) digest aset | `gh api`, keluaran tersimpan di `logs/` |
| VERIFIED-ON-VM | Berlaku untuk rantai boot, shell, dan tangkapan layar | QEMU emulator version 10.2.50 (v10.2.0-14086-g840320663e) headless q35 -vga std; serial 5787 B; 3 PNG QMP; verdict [PASS BERSYARAT]; kode keluar 0 |
| VERIFIED-ON-HARDWARE | **0** -- tidak satu pun klaim | jendela fisik K-6/keyboard PS/2 ditunda dengan otorisasi tertulis 2026-10-24 (entri ke-60 dan ke-62) |
| RANCANGAN | Berlaku untuk rencana yang belum dijalankan | tidak diklaim sebagai bukti |

Isi jendela VM yang terukur: rantai boot M0..M6 lengkap, prompt `ASL-KERNEL >_ ` (dibaca dari
konstanta satu-sumber `asl-cockpit-shell/src/lib.rs`; bentuk terposisi `ASL-KERNEL <posisi>>_ `),
sesi 17 baris masukan lewat jalur serial ANSI dengan 17/17 baris mendapat prompt baru
sebelum batas 60 s, 13/13 penanda lulus (satu lewat varian konstanta `MSG_*` -- dicatat
sebagai deviasi kriteria, bukan disembunyikan), versi kernel `v1.0.2`3 kali dan versi lama
`v1.1.0` 0 kali, stderr QEMU 195 B yang terdiri atas 1 baris info chardev dan sisa 0 B
(gerbang dibaca dari sisa, bukan dari keseluruhan).

## 3. Blocker #50 (perkakas bootloader)

TERTUTUP sisi host dan sisi VM: pin Limine varian A dipasang pada BAGIAN 3,
`scripts/check_limine.ps1` exit 0, dan rantai boot diukur ulang sepenuhnya; dibuktikan
ulang oleh jendela VM BAGIAN 5 dan BAGIAN 6 di atas ISO master yang sama (hash identik).

## 4. Janji lantai memori -m 3M: MERAH

Baris alat pada fase 2 jendela: `[MERAH] fase 2: -m 3M tidak mencapai prompt pada ISO ini (serial=0 byte).`
Serial fase 2 terukur 0 B. Ratchet 3392K lulus. Status merah ini diwarisi dari
ACCOUNTABILITY_FAULT_LOG entri ke-23 (`result` OPEN); tidak diusahakan perbaikannya pada
rantai ini karena keputusan P1/P3 pemilik membatasi lingkup. Ini dinyatakan di rilis agar
janji misi tidak terlihat lebih hijau dari kenyataannya.

## 5. Aset

| Aset | Ukuran | SHA256 host |
|---|---|---|
| `asl-core-kernel-v1.0.2.iso` | 4624384 B | `1569c17157b98c8aa29c9e9d18ccb6aeb931c883d1e1a25983864df4206c668c` |
| `SHA256SUMS.txt` | 2034 B | `5c35c77f88a060fce723a55e8a857b6464ca241dc988a7e5542188eb655a3355` |

ISO yang diunggah adalah berkas yang sama dengan yang di-boot pada jendela BAGIAN 6
(terukur identik byte; salinan pra-unggah diverifikasi hash-nya sebelum unggah).

## 6. BATASAN JUJUR

1. Tidak ada satu pun bukti pada perangkat keras pemilik (VERIFIED-ON-HARDWARE = 0).
2. Jendela penerimaan tidak disaksikan manusia (keputusan P4); masukan sesi lewat serial
   ANSI, bukan keyboard PS/2 yang ditekan manusia. Jalur keyboard hanya teruji pada
   jendela tersaksikan 2026-10-21 (artefak `g9_final_p8_*`), yang tidak diklaim ulang di sini.
3. Janji `-m 3M` tetap merah (bagian 4).
4. K-6 fisik (perbaikan ESP media pemulihan) ditunda berotorisasi; nol byte ditulis ke ESP.
5. Tag `v1.1.0` dihapus dari repo LOKAL setelah diarsipkan
   (`Analysis/reports/T4R-ARSIP-TAG-V1.1.0-2026-10-24.md`); biner ISO v1.1.0 tidak dapat
   diverifikasi ulang lagi karena tidak pernah di-commit dan tidak ada di disk saat diukur.
6. Coverage hosted bersifat advisory (non-blocking, continue-on-error) dan tidak
   diklaim sebagai gerbang lulus.
7. Tangkapan layar adalah bukti kondisi konsol VM pada saat diambil, bukan bukti tampilan
   pada monitor perangkat keras.

## 7. Cara memeriksa ulang

```
git fetch --tags
git verify-tag v1.0.2
sha256sum -c SHA256SUMS.txt   # di direktori yang memuat ISO
```
Perkakas jendela: `python scripts/p8_unattended_run.py --selftest` lalu invokasi tanpa
flag; artefak ditulis di `Analysis/reports/` dengan prefiks `p8_unattended_*`.
