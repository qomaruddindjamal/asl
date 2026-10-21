# BUKTI JENDELA PENERIMAAN P8 - G9 FINAL (v1.0.2)

Tanggal pengukuran: 2026-10-21 (zona waktu host build). Kelas bukti utama: **VERIFIED-ON-VM-VISIBLE**
(jendela QEMU terlihat selama sesi; angka di bawah dihitung dari berkas serial, bukan dari ingatan).

## 1. IDENTITAS YANG DIUKUR

| Item | Nilai terukur | Kelas |
| :--- | :--- | :--- |
| Tree yang di-boot | tip `75aaf93` branch `feature/v1.0.2-shell` (commit T1 aturan NO-OS-NAME) | VERIFIED-ON-HOST |
| ISO_P8 (yang di-boot) | 4.200.448 byte, SHA256 `aa77806018d7a9009bb705e29e71d33724a6b50fc3a5cf62de24133359c74de9` | VERIFIED-ON-HOST |
| ELF di dalam ISO | 325.528 byte, SHA256 `9203864ea6feae5bd3d51abd0cd3dc69ddcec0b8e91cd5401b1c63074a27b216` | VERIFIED-ON-HOST |
| QEMU | `QEMU emulator version 10.2.50 (v10.2.0-14086-g840320663e)`, mesin `q35`, `-vga std`, `-m 256M` | VERIFIED-ON-VM |
| Skrip | `scripts/g9_final_p8_run.ps1` (363 baris, SHA256 `2a866129...`), invokasi `-AcceptDeviation3M` | VERIFIED-ON-HOST |
| Exit code akhir | **0** (baris `EXIT_CODE=0` pada artefak stdout) | VERIFIED-ON-HOST |

Prinsip bagian T2e berlaku: ISO yang di-boot pada dokumen ini adalah ISO yang akan dirilis pada
tahap T4, jadi SHA256 di atas harus muncul kembali pada build di tip `main` pasca-merge. Bila berbeda,
pelepasan berhenti (STOP GLOBAL).

## 2. METODOLOGI JENDELA (dilaporkan apa adanya)

- Jendela QEMU **terlihat** sepanjang sesi fase 1. Tidak ada `-display none`, tidak ada
  `-WindowStyle Hidden`, tidak ada mode tanpa-antarmuka.
- Pemilik hadir di layar dan **memilih** agar naskah sesi diketik oleh alat, bukan oleh jari pemilik
  (keputusan pemilik 2026-10-21, tercatat dalam sesi kerja ini). Yang disaksikan pemilik adalah
  perilaku shell pada jendela itu sendiri.
- Alat ketik: `logs/t2b_type_keys.ps1` (scratch, tidak di-track) memakai `user32.dll` bawaan
  (`keybd_event` dengan kode scan set-1) setelah aktivasi + klik jendela display QEMU. Tidak ada
  aplikasi pihak ketiga yang dipasang. `SendKeys`/`SendInput` terbukti **ditolak** oleh antarmuka input host build karena
  proses pengirim bukan pemilik foreground; ini terukur, bukan asumsi.
- Proses harness dijalankan terlepas dari konsol (stdout ke berkas) supaya agen bisa mengirim tombol
  sambil mesin penunggu membaca serial. Konsekuensinya tercatat sebagai **perbedaan prosedur** terhadap
  huruf aturan G9 ("dilarang eksekusi background untuk sesi ini"): yang dilarang secara terang
  (jendela tersembunyi/tanpa jendela) tidak dipakai, tetapi pelepasan proses induk dari konsol tetap
  merupakan latar. Angka kelulusan tidak bergantung pada hal ini - mesin penunggu hanya membaca
  13 penanda dari berkas serial.
- Bukti kelulusan tetap fail-closed: mesin penunggu menghitung kemunculan string pada serial dan
  mengembalikan kode keluar 2 bila satu pun penanda kurang (itu terjadi pada empat jendela pertama).

## 3. TUJUH JENDELA YANG DIJALANKAN (semua pada tree dan ISO yang sama)

| # | cap waktu serial | byte | penanda | baris perintah | hasil |
| :--- | :--- | ---: | :---: | :---: | :--- |
| 1 | 13-22-57 | 6.988 | 6/13 | 18 | exit 2 - yang diketik eksplorasi bebas, bukan naskah |
| 2 | 13-29-30 | 5.066 | 6/13 | 12 | exit 2 - sebab sama |
| 3 | 13-34-54 | 2.531 | 0/13 | 1 | exit 2 - hampir tidak ada yang diketik |
| 4 | 17-24-19 | 2.596 | 0/13 | 1 | exit 2 - dihentikan setelah diagnosis |
| 5 | 17-38-36 | 2.516 | 0/13 | 0 | dihentikan; serial identik byte dengan baseline cold boot |
| 6 | 17-40-41 | 7.275 | 13/13 | 20 | fase 1 HIJAU; perlu dua baris dikirim ulang (cacat alat ketik, lihat §5) |
| 7 | 17-51-59 | 5.304 | 13/13 | 17 | **dijadikan bukti utama**: satu lintasan bersih, `EXIT_CODE=0` |

Artefak jendela 7 yang ikut ter-commit: `g9_final_p8_serial_2026-10-21_17-51-59.log` (5.304 byte,
SHA256 `f5483667ada8f5280deb31378883c2d077b9e511ec5c48bc8c628823181e4c19`),
`g9_final_p8_summary_2026-10-21_17-51-59.log` (4.385 byte, SHA256 `4b5bd248fd2efc9497a8825894e75bf99c5cb0f489ea1b68b5999d8b60c5c28e`),
`g9_final_p8_stdout_2026-10-21_17-51-59.log` (4.337 byte, SHA256 `6b1afaff1e19a93875af9c531b9b79a75fce102b3bc29ef0f67d2dca8f46447e`,
memuat `EXIT_CODE=0`), dan tiga berkas nol byte
(`g9_final_p8_qemu_stderr_2026-10-21_17-51-59.log`, `g9_final_p8_qemu_stderr_3m_2026-10-21_17-51-59.log`,
`g9_final_p8_serial_3m_2026-10-21_17-51-59.log`; SHA256 `e3b0c442...b855` = hash kosong).

Angka byte dan SHA256 di atas adalah panjang **berkas di disk** (konvensi yang ditetapkan
ACCOUNTABILITY_FAULT_LOG entri ke-25). Karena `core.autocrlf=true`, blob yang tersimpan di repo
memakai bentuk LF, jadi hash konten repo berbeda: serial 5.186 byte SHA256
`95d6f7b9526328f0f1f881275fde14355a729c07283ff8ff8f938725a8b7660a`, summary 4.320 byte SHA256
`ad5da7b8e4a3e1b787377bdd0cdeb4a7d1f84020cfe0a90ed7efec6623e8c32f`, stdout 4.271 byte SHA256
`ad370a939ceee63a43bb8712e122d92981de31dbd1c4996ee027d9a165ddd137`. Ini satu artefak yang sama
dengan dua konvensi garis baru, bukan dua pengukuran yang saling bertentangan.
Artefak jendela 1-6 sengaja **tidak** di-commit dan tetap ada di disk sebagai jejak; ukurannya terdaftar
di tabel ini supaya tidak ada angka yang hilang.

## 4. HASIL FASE 1 DAN FASE 2 (dikutip dari artefak stdout jendela 7)

- Boot: `M6=True prompt=True crash=False` tercapai setelah 6 detik.
- 13/13 penanda sesi terpenuhi pada serial (hitung kemunculan, bukan kehadiran tebak-tebak):
  registry `help` 2, `status` 1, `Direktori dibuat:` 1, `Daftar isi /` 2, `Ruang kerja:` 2,
  `Berkas dibuat:` 1, `Tertulis ` 2, isi berkas via alias `cat` 2, `Berkas dihapus:` 1,
  `Direktori dihapus:` 1, `Perintah tidak dikenal:` 1, `Sintaks:` 1, sekuens ANSI `clear` 1.
- Gerbang versi: occurrence `v1.0.2` = 3, occurrence `v1.1.0` = 0 (gerbang: >= 1 dan = 0). **HIJAU.**
- `stderr` QEMU = 0 byte pada kedua fase.
- Fase 2 (`-m 3M`): **MERAH** - `M6=False prompt=False crash=False serial=0 byte`. Ini deviasi yang
  sudah tercatat sebelumnya (ACCOUNTABILITY_FAULT_LOG entri ke-23, result OPEN), bukan temuan baru, dan
  **diterima pemilik** lewat keputusan A dengan `-AcceptDeviation3M`. Janji pemangkasan footprint memori
  boot masuk daftar utang iterasi berikutnya dan wajib muncul di LAPORAN-FINAL.
- Keputusan akhir skrip: `[PASS BERSYARAT]` dengan jalur `exit 0`; angka exit code yang terukur ada pada
  baris `EXIT_CODE=0` artefak stdout.

## 5. CACAT ALAT KETIK YANG SAYA TEMUKAN DAN PERBAIKI (bukan cacat kernel)

Peta karakter ke kode scan set-1 pada alat ketik scratch saya awalnya tidak memuat digit, sehingga
baris `write-file catatan.txt halo dari sesi p8` terkirim tanpa `8` dan penanda "isi berkas via alias
cat" tetap 0/2 pada jendela 6. Perbaikannya menambah kode scan digit; jendela 7 lolos dalam satu
lintasan. Tidak ada berkas `.rs`, `Cargo.*`, maupun skrip ter-track yang berubah karena hal ini -
alat ketik berada di `logs/` (di-ignored).

## 6. TANGKAPAN LAYAR

Tidak ada tangkapan layar yang diajukan sebagai bukti penerimaan, sesuai disiplin kategori bukti
proyek ini (VERIFIED-ON-VM-VISIBLE diukur dari serial + kehadiran pemilik, bukan dari gambar). Satu
tangkapan layar pernah diambil **sekali** untuk mendiagnosis jendela mana yang menerima tombol;
berkasnya ada di `logs/` yang tidak di-track dan tidak dipakai sebagai angka di dokumen ini.

## 7. YANG MASIH TERBUKA SETELAH TAHAP INI

1. Run hosted atas commit bukti ini belum bisa diekstrak di dalam commit-nya sendiri (aturan G8);
   ekstraknya masuk commit berikutnya.
2. Entri ke-23 tetap result OPEN sampai tahap T5 menutupnya dengan
   `ACCEPTED_WITH_WRITTEN_JUSTIFICATION` (penambahan field resolusi, field lama tidak dihapus).
3. Operasi tag (hapus rilis lama, terbitkan `v1.0.2`) adalah tahap T4 dan akan tercatat sebagai
   entri ke-28.
4. Empat ekstrak CI atas commit `9bc205e` dan `75aaf93` ikut ter-commit pada commit bukti ini dan
   menutup utang ekstraksi kedua tip tersebut.
