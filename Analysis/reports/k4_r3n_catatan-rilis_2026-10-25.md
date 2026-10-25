# ASL Core Kernel v1.0.2: shell Cockpit + rantai boot Limine ter-pin + bukti VM

## 1. Konteks

Rilis SAH yang menggantikan potongan v1.0.2 prematur: pada saat catatan ini disusun hanya ada tepat 1 rilis GitHub ber.tag `v1.0.2` (id `422838293`, diterbitkan 2026-10-24T16:14:39Z) dan tag `v1.1.0` beserta rilisnya sudah tidak ada di origin maupun di API (diarsipkan lalu dihapus; manifes `Analysis/reports/T4R-ARSIP-TAG-V1.1.0-2026-10-24.md`).

**VERIFIED-ON-HARDWARE: 0** (jendela fisik ditunda, otorisasi tertulis pemilik 2026-10-24 --
pemilik bepergian). Rilis ini diterbitkan dari rantai R3 yang berjalan penuh tanpa
konfirmasi, dengan setiap deviasi dicatat di `docs/ROADMAP/ACCOUNTABILITY_FAULT_LOG.json`.

Dipotong pada tag annotated `v1.0.2` objek `79d581f8291650679693e6014e0ce4b73b54faf7` -> commit `700481c22dc98b92be5bbd2059c54ea5bf97feb3`, yaitu tip yang lulus BAGIAN 6 (jendela penerimaan VM tak-disaksikan). Tip `main` setelah merge PR #7 adalah `c6e4c01788789e3126fa3488abb7696f72d0f485`; selisih tree terhadap basis rilis terukur hanya `README.md` dengan nol berkas `.rs`, dan kedua commit hijau hosted CI.

## 2. Apa yang baru (untuk pengguna)

- **Cockpit Shell: 13 perintah kanonik + 10 alias**, disusun dari registry satu-sumber (`REGISTRY` di `asl-kernel/asl-cockpit-shell/src/lib.rs`), bukan dari teks tetap. Urutan sama dengan keluaran `help`: help, clear, where, list, open, up, create-folder, create-file, write-file, rename, delete-file, delete-folder, status.
  Alias yang dikenali: list -> ls, dir; open -> cd, cat, type, read-file, open-file; create-folder -> mkdir; rename -> mv; delete-file -> rm.
  Dokumen `docs/PANDUAN/REFERENSI-PERINTAH.md` Pasal 4 memuat 13 blok perintah dan terukur IDENTIK dengan registry (nama, sintaks, alias, urutan) oleh naskah ukur unit ini; sinkronnya juga dijaga test host `test_docs_command_reference_matches_registry`.
  CATATAN PRECISI (bukan koreksi produk): masukan pemilik untuk restrukturisasi ini menyebut "11 perintah kanonik + 9 alias". Angka itu adalah keadaan unit P3. Konstanta `CANONICAL_COMMAND_COUNT` terukur pernah bernilai 11, lalu 12 ketika perintah `up` masuk (komit `c3a9f2e`, penanda `[V1.0.2-R1-F2-UP]`), lalu 13 ketika `rename` masuk (komit `791a88d`, penanda `[V1.0.2-R1-F6-RENAME]`), dan `mv` menambah alias ke 10. Yang diterbitkan di sini adalah keadaan tree rilis: 13 perintah dan 10 alias.
- **Penyunting baris ANSI pada jalur masukan**: mesin sunting satu-sumber di shell ("PENYUNTING BARIS + JALUR INPUT KANONIK") dengan cincin riwayat (`HIST_MAKS`), pelengkapan Tab yang mengambil kandidat dari registry (`kandidat_perintah`, `perpanjangan_bersama`), pengenalan sekuens escape empat fase (`ESC_GOT_SS3`, Esc, Ctrl+C, Ctrl+L), dan prompt berposisi (`ASL-KERNEL />_ ` di akar, bergerak bersama direktori kerja). Batasnya jujur: unit F7 berstatus PARSIAL - yang terbukti di host adalah mesin sunting dan navigasi tanpa perpindahan kursor di framebuffer; sisa yang menuntut gerak kursor horizontal tidak dikerjakan pada rantai ini.
- **RAMDISK volatil**: kapasitas default 262144 B (256 KiB) = 32 simpul x slot 8192 B (8 KiB); konstanta di `asl-kernel/asl-storage/src/ramdisk.rs`, dan dokumen PANDUAN menyebut angka yang sama: "262144 byte (256 KiB)". Isi hilang ketika mesin dimatikan; tidak ada berkas di disk, tidak ada jaringan.
- **Identitas rilis satu-sumber**: `pub const KERNEL_VERSION: &str = "v1.0.2"` adalah satu konstanta yang dirujuk 9 kali oleh `asl-kernel-core` untuk banner VGA, framebuffer, dan serial - tidak ada string versi terserak. Pada serial jendela VM yang ter-commit, occurrence `v1.0.2` = 3 dan occurrence versi lama `v1.1.0` = 0.
- **Dokumentasi PANDUAN ter-commit** (4 berkas):
- `docs/PANDUAN/KONFIGURASI-MEMORI.md` (5351 B, SHA256 5ed5fad5766cd549b7cdf279abdbe8da36d29c2bcf1be383715c487750ff64d6)
- `docs/PANDUAN/MULAI-CEPAT.md` (6575 B, SHA256 1c097de45514149997223f02338f1bb89edd974f490e6db3308f66e016af95bd)
- `docs/PANDUAN/README.md` (3275 B, SHA256 c67674f5561d969916f2e8315cc4b2a39f87fb1f2c323763ab633369af771a3f)
- `docs/PANDUAN/REFERENSI-PERINTAH.md` (26489 B, SHA256 20593777cfce213f64572738ada57875dcbf8ffe0c3896e5182c22830f8f7ddc)
- **Perbaikan E8 + rantai F1-F7**: rentang `b47c24c..700481c` memuat 52 komit tanpa merge; penanda unit E8 (`[V1.0.2-R1-F8-*]`) terukur pada 19 komit dan unit F1..F7 pada 10 komit. E8 DINYATAAN TERTUTUP oleh pemilik 2026-10-22 dalam `docs/RND/ASL-V1.0.2-E8-LAPORAN-PENUTUPAN.md` (frasa penutupnya terukur ada di berkas 18218 B); status F7 PARSIAL tercatat di `docs/ROADMAP/HANDOFF_STATE.json`.
- **Blocker perkakas bootloader (pin Limine varian A) TERTUTUP secara fungsional.** Yang dimaksud "blocker #50" adalah entri ke-50 di `ACCOUNTABILITY_FAULT_LOG.json` (jenis `CLEAN_TARGET_MENGHAPUS_PERKAKAS_BOOTLOADER_DI_LUAR_VCS_MASTERING_ISO_TERBLOKIR`), BUKAN nomor issue GitHub - terukur `gh api .../issues/50` mengembalikan HTTP 404 (`status=404, message=Not Found`). Field `result` entri ke-50 tetap `OPEN` karena ledger ini append-only dan nol isi lama boleh diubah; penyelesaiannya dicatat pada entri ke-59 (`result=CLOSED`, gerbang BAGIAN 3) dengan alat `scripts/check_limine.ps1` (5625 B, SHA256 `eca58b174b7c1b4e`, komit terakhir `5ad71ae`) yang exit 0, lalu dibuktikan ulang oleh jendela VM BAGIAN 5 dan BAGIAN 6 di atas ISO master yang sama.

## 3. Aset + SHA256

| Aset | Ukuran | SHA256 |
|---|---|---|
| `asl-core-kernel-v1.0.2.iso` | 4624384 B | `1569c17157b98c8aa29c9e9d18ccb6aeb931c883d1e1a25983864df4206c668c` |
| `SHA256SUMS.txt` | 2034 B | `5c35c77f88a060fce723a55e8a857b6464ca241dc988a7e5542188eb655a3355` |

Angka aset dibaca ulang dari audit K-4 BAGIAN 1 yang READ-ONLY: berkas ISO di disk `C:\asl\asl-kernel\target\asl-core-kernel.iso` terukur 4624384 B SHA256 `1569c17157b98c8aa29c9e9d18ccb6aeb931c883d1e1a25983864df4206c668c`, IDENTIK BYTE dengan entri `asl-core-kernel-v1.0.2.iso` di `SHA256SUMS.txt` yang diunduh dari rilis (gerbang `identik_byte=True`, `ukuran_cocok_aset=True`, `cocok_aset_digest=True`). `SHA256SUMS.txt` sendiri 2034 B, 19 baris tidak kosong, 1 entri hash - sisanya komentar `#`. Rilis memuat 2 aset, keduanya ber-state `uploaded` (digest server `1569c17157b98c8aa29c9e9d18ccb6aeb931c883d1e1a25983864df4206c668c`).

## 4. Cara memverifikasi

```
git fetch --tags
git verify-tag v1.0.2
sha256sum -c SHA256SUMS.txt   # di direktori yang memuat ISO
```

Tag diverifikasi tiga jalan pada unit pra-jendela: objek lokal, `git ls-remote origin`, dan API `git/ref/tags/v1.0.2` - ketiganya `79d581f8291650679693e6014e0ce4b73b54faf7` (tipe objek `tag`), peel ke `700481c22dc98b92be5bbd2059c54ea5bf97feb3`. Pesan tag 2921 B SHA256 `7d890006097ca3dbd9e70f92982699ce84f4aeba12322efae3da77e956cdf982` terukur identik byte dengan `Analysis/reports/r3_bagian7_pesan_tag_v1.0.2_2026-10-24.txt`. Perkakas jendela: `python scripts/p8_unattended_run.py --selftest` lalu invokasi tanpa flag; artefak ditulis di `Analysis/reports/` dengan prefiks `p8_unattended_*`.

## 5. Status verifikasi ringkas

Angka gerbang dibaca dari berkas ekstrak hosted CI yang ter-commit (`Analysis/reports/ci_hosted_<run-id>_2026-10-24.log`) dengan parser yang juga ter-commit, bukan dari hitungan lokal. Run terbaru `40137397612` (event `push`, `completed/success`, headSha `a737f1de81f21ca9b0f68662676dd0f888ae1d2f`) mengukur: suite=30, passed=262, failed=0, per-test terhitung=262, warning langkah [1/2]=19, error langkah [1/2]=0, `Compiling`=13, warning langkah [2/2]=0, error langkah [2/2]=0, footprint ELF dilaporkan runner=312968 B. Himpunan nilainya IDENTIK dengan seluruh ekstrak bertanggal 2026-10-24 yang ter-commit (32 berkas termasuk run ini) = nol regresi. Footprint ELF di host terukur 312912 B; selisih 56 B terhadap laporan runner dibiarkan terbuka, tidak dicocokkan paksa (runner Linux, host Windows).

| Kelas bukti | Status | Bukti |
|---|---|---|
| VERIFIED-ON-HOST | Berlaku untuk seluruh angka sisi host dan angka jendela | berkas bukti `r3_bagian*_2026-10-24.txt`, `k4_hardware_*_2026-10-25.txt`, artefak `p8_unattended_*` |
| VERIFIED-ON-CI | Berlaku untuk langkah [1/2] dan [2/2] pipeline minimal | ekstrak hosted CI bertanggal 2026-10-24 di `Analysis/reports/` |
| VERIFIED-ON-CI-API | Berlaku untuk metadata run dan digest aset | `gh api`, keluaran tersimpan di `logs/` (ter-ignore) dan disalin ke berkas bukti |
| VERIFIED-ON-VM | Berlaku untuk rantai boot, shell, dan tangkapan layar | QEMU 10.2.50 headless q35 -vga std; serial 5787 B; 3 PNG QMP; verdict [PASS BERSYARAT]; kode keluar 0 |
| VERIFIED-ON-HARDWARE | **0** - tidak satu pun klaim | jendela fisik K-6/keyboard PS/2 ditunda dengan otorisasi tertulis 2026-10-24 (entri ke-60 dan ke-62); jendela K-4 yang akan datang adalah kandidat pertama untuk kelas ini |
| RANCANGAN | Berlaku untuk rencana yang belum dijalankan | tidak diklaim sebagai bukti |

Isi jendela VM yang terukur: rantai boot M0..M6 lengkap, prompt `ASL-KERNEL />_ ` (dibaca dari konstanta satu-sumber), sesi 17 baris masukan lewat jalur serial ANSI dengan 17/17 baris mendapat prompt baru sebelum batas 60 s, 13/13 penanda lulus (satu lewat varian konstanta `MSG_*` - dicatat sebagai deviasi kriteria, bukan disembunyikan), stderr QEMU 195 B yang terdiri atas 1 baris info chardev dan sisa 0 B.

Janji lantai memori `-m 3M` tetap MERAH: `[MERAH] fase 2: -m 3M tidak mencapai prompt pada ISO ini (serial=0 byte).` Ratchet 3392K lulus. Status ini diwarisi dari entri ke-23 (`result` OPEN) dan tidak diusahakan perbaikannya pada rantai ini karena keputusan P1/P3 pemilik membatasi lingkup. Dinyatakan di sini agar janji misi tidak terlihat lebih hijau dari kenyataannya.

## 6. BATASAN JUJUR

Butir 1-7 dipertahankan utuh dari catatan rilis sebelumnya (tidak ada yang dikurangi); butir 8-11 ditambahkan oleh restrukturisasi ini.

1. Tidak ada satu pun bukti pada perangkat keras pemilik (VERIFIED-ON-HARDWARE = 0).
2. Jendela penerimaan tidak disaksikan manusia (keputusan P4); masukan sesi lewat serial
   ANSI, bukan keyboard PS/2 yang ditekan manusia. Jalur keyboard hanya teruji pada
   jendela tersaksikan 2026-10-21 (artefak `g9_final_p8_*`), yang tidak diklaim ulang di sini.
3. Janji `-m 3M` tetap merah (bagian 5 di atas).
4. K-6 fisik (perbaikan ESP media pemulihan) ditunda berotorisasi; nol byte ditulis ke ESP.
5. Tag `v1.1.0` dihapus dari repo LOKAL setelah diarsipkan
   (`Analysis/reports/T4R-ARSIP-TAG-V1.1.0-2026-10-24.md`); biner ISO v1.1.0 tidak dapat
   diverifikasi ulang lagi karena tidak pernah di-commit dan tidak ada di disk saat diukur.
6. Coverage hosted bersifat advisory (non-blocking, continue-on-error) dan tidak
   diklaim sebagai gerbang lulus.
7. Tangkapan layar adalah bukti kondisi konsol VM pada saat diambil, bukan bukti tampilan
   pada monitor perangkat keras.
8. "Blocker #50" adalah nomor entri ledger internal, bukan nomor issue GitHub (API mengembalikan 404). Field `result` entri ke-50 tetap OPEN secara harfiah karena ledger append-only; yang tertutup adalah akar teknisnya, tercatat di entri ke-59.
9. Unit F7 berstatus PARSIAL: mesin sunting dan navigasi terbukti di host, sedangkan sisa yang menuntut perpindahan kursor horizontal di framebuffer tidak dikerjakan, dan bukti VM untuk F7 pada saat itu TERBLOKIR (belum ada perkakas Limine).
10. `RELEASE_NOTES.md` di repo mempertahankan teks riwayat v1.1.0 dan v1.0.1 apa adanya di bawah bagian v1.0.2; dokumen itu bukan ringkasan keadaan kini untuk dua rilis lama.
11. Catatan rilis ini diubah lewat `gh release edit` (docs-only). Objek tag `79d581f...` dan seluruh komit terbit tidak disentuh, sehingga pesan tag tetap berisi keadaan pada saat tag dipotong - termasuk `VERIFIED-ON-HARDWARE: 0` versi lama.

## 7. Rujukan berkas bukti

- Laporan paling akhir: `Analysis/reports/LAPORAN-FINAL-v1.0.2.md` (25219 B, 286 baris tidak kosong, SHA256 `1226874c74028434c6d04739a0e5f9837c3c1bf64ec79910fd86041b9d55a29d`)
- Bukti pra-jendela K-4 (dibaca dari pohon kerja): `Analysis/reports/k4_hardware_bagian1-audit_2026-10-25.txt` (5768 B), `Analysis/reports/k4_hardware_bagian3-rencana-dan-lembar-2026-10-25.txt` (3721 B)
- Ledger: `docs/ROADMAP/ACCOUNTABILITY_FAULT_LOG.json` (entri ke-60 K-6 fisik ditunda, ke-62 jendela VM, ke-63 rilis, ke-64 LAPORAN-FINAL, ke-65 audit pra-jendela K-4 - penomoran DIUKUR dari field `gate_id` ledger, bukan dari ingatan) dan `docs/ROADMAP/HANDOFF_STATE.json`
- Ekstrak hosted CI bertanggal 2026-10-24: 32 berkas `Analysis/reports/ci_hosted_*_2026-10-24.log` = 31 yang sudah ter-commit di pohon HEAD + SATU yang ikut komit restrukturisasi ini (`ci_hosted_40137397612_2026-10-24.log`)
- Artefak jendela VM: 16 berkas `Analysis/reports/p8_unattended_*` (serial, summary, stderr, hasil JSON, tiga PNG QMP) - semuanya ter-commit, bukan hanya di `logs/` yang ter-ignore
- Snapshot media pra-flash (READ-ONLY, alat ter-commit): `scripts/k4_media_state.py` (5913 B, SHA256 `6d1bd6d1ddaffe19fab9ee649643ebde83493287308f36fac1e1e71672a2152e`, non-ASCII 0, 127 baris; terdaftar di `git ls-files` saat ukur=True). Keluarannya `logs/k4_media_state.json` (1081 B, SHA256 `85d17a94da3b047d2ee67000bb6426462ad4463e816efdb35c71c743c031735b`) ber-`nol_tulis=True` dengan satu volume ber-huruf tunggal dan `dir` exit 0 (8 baris). Cacat versi scratch (`cmd /c dir` dengan satu argumen, exit 1) direproduksi READ-ONLY dan dicatat di ledger entri ke-66.
- Kandidat media yang terukur (read-only, belum di-flash): disk 1 `SanDisk Cruzer Blade USB Device` interface USB, style MBR, volume F:FAT32/Healthy, ukuran Get-Disk 15401484288 B vs Win32_DiskDrive 15397724160 B (selisih 3760128 B dibiarkan terbuka, tidak dicocokkan).
- Catatan ini sendiri: `Analysis/reports/k4_r3n_catatan-rilis_2026-10-25.md`

Riwayat catatan ini.

Body sebelum restrukturisasi: 7086 B, 108 baris, SHA256 `d40dca6f878cced8a22b3b93d769dac124d0775306840f2a2a4c5b4746674ffc`, non-ASCII 0 - dikembalikan IDENTIK BYTE oleh API (`identik_byte=True`) ketika dibaca ulang SEBELUM unit ini mengubahnya; angka keadaan awal itu dibekukan di `logs/r3n_baseline_sebelum_edit.json` (ter-ignore) dan disalin ke berkas bukti unit agar dapat diperiksa ulang. Catatan yang sedang dibaca ini adalah terbitan PALING AKHIR dari unit ini: setiap `gh release edit` menimpa body sebelumnya di API, sehingga terbitan antara - termasuk terbitan sebelum koreksi - tidak dapat dibaca ulang baik dari API maupun dari riwayat komit. Ukuran dan SHA256 TIAP terbitan didaftar di berkas bukti `Analysis/reports/k4_hardware_r3n-restrukturisasi_2026-10-25.txt`, DIUKUR ulang dari arsip byte-per-byte di `logs/` memakai fungsi meter yang sama, bukan dari ingatan. DUA KLAIM RUJUKAN pada terbitan sebelumnya memang TIDAK diukur oleh naskah dan kini dibetulkan: jumlah berkas ekstrak hosted CI dibaca dari `git ls-tree -r HEAD` (31 yang sudah ter-commit + SATU yang ikut komit ini = 32) dan bukan dari indeks `git ls-files` yang sudah memuat berkas ber-staging, dan penomoran entri ledger dibaca dari field `gate_id` - termasuk entri audit pra-jendela K-4 yang terukur ke-65.
