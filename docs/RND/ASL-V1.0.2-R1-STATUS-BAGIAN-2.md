# STATUS PEKERJAAN BAGIAN 2 (E8 + F1-F7) - BUKAN LAPORAN-FINAL

Label berkas ini adalah bagian dari isinya. Ini STATUS PEKERJAAN pada satu tip
cabang, ditulis oleh agen sebelum satu pun jendela pada tip terakhir disaksikan
pemilik. Berkas ini BUKAN `LAPORAN-FINAL-v1.0.2.md`, BUKAN sertifikat, dan BUKAN
dasar untuk menyatakan rilis siap. Rilis v1.0.2 sampai detik ini tidak diterbitkan
dan tag tidak dipotong.

Ruang lingkup: rantai R1 pada cabang `feature/v1.0.2-r1`, dari BAGIAN 0, BAGIAN 1
(E8), sampai BAGIAN 2 (F1-F7) dan gerbang akhirnya. Angka yang dikutip di bawah
disalin dari berkas bukti milik unit yang bersangkutan; angka hari ini ada di
Bagian 5.

## 1. Status per unit

| Unit | Isi yang dikerjakan | Uji host tercatat | Kelas bukti | Berkas bukti |
| --- | --- | --- | --- | --- |
| BAGIAN 0 | Penutupan rantai lama: dua entri fault, revert tag v1.0.2, pembukaan cabang R1 | tidak ada perubahan kode | VERIFIED-ON-HOST (docs) | `docs/ROADMAP/ACCOUNTABILITY_FAULT_LOG.json` entri #28-#31 |
| BAGIAN 1 (E8) | Enam akar penyebab masukan adversarial ditutup di jalur ketikan; enam penjaga regresi `panic_guard_*`; laporan penutupan atas keputusan pemilik | 218 lulus pada masa F4; enam penjaga baru | VERIFIED-ON-HOST, VERIFIED-ON-VM (boot dan sesi serial pada ISO E8), jendela langkah 6 disaksikan pemilik | `docs/RND/ASL-V1.0.2-E8-AKAR-PENYEBAB-DAN-STATUS.md`, `docs/RND/ASL-V1.0.2-E8-LAPORAN-PENUTUPAN.md` |
| F1 | Posisi aktif pada prompt dari satu sumber; tiga test; tiga dokumen sinkron | 205 lulus | VERIFIED-ON-HOST, VERIFIED-ON-VM; ISO 4.204.544 B SHA256 `9CCE9521...` | `Analysis/reports/f1_prompt-berposisi_uji_2026-10-22.txt` |
| F2 | `up` masuk registry kanonik (11 menjadi 12); dua pasangan ekstrak; jendela E8 diulang | 207 lulus | VERIFIED-ON-HOST, VERIFIED-ON-VM; ISO SHA256 `5E5433F6...` | `Analysis/reports/f2_up_uji_2026-10-22.txt` |
| F3 | Dua puluh empat label keluaran satu sumber; jalur dispatch eksplisit; lima test | 212 lulus | VERIFIED-ON-HOST, VERIFIED-ON-VM; ISO 4.212.736 B SHA256 `8BC56058...` | `Analysis/reports/f3_keluaran_uji_2026-10-22.txt` |
| F4 | Akar bug argumen terpotong pada `write-file`: pemotongan ditandai, guard sebelum VFS, enam test | 218 lulus | VERIFIED-ON-HOST, VERIFIED-ON-VM, VERIFIED-ON-CI-API | `Analysis/reports/f4_argumen_terpotong_uji_2026-10-22.txt` |
| F5 | Mode berpanduan sesudah galat arity; penanda kursor idle hanya pada framebuffer; cermin layar entri #38; empat belas test | 232 lulus | VERIFIED-ON-HOST, VERIFIED-ON-VM, VERIFIED-ON-VM-VISIBLE (piksel screendump terukur); ISO 4.177.920 B SHA256 `E4CA4ABC...` | `Analysis/reports/f5_pemandu_uji_2026-10-22.txt`, `Analysis/reports/f5_kursor_uji_2026-10-22.txt` |
| F6 | Perintah `rename` berkas dan folder dengan subtree bergeser, dilarang menimpa; registry 12 menjadi 13; tujuh belas test; posisi penanda kursor terukur | 249 lulus | VERIFIED-ON-HOST, VERIFIED-ON-VM, VERIFIED-ON-VM-VISIBLE; ISO 4.184.064 B SHA256 `6AF6D9F5...` | `Analysis/reports/f6_rename_uji_2026-10-22.txt`, `Analysis/reports/f6_sidik_masukan_uji_October-22.txt` |
| F7 (PARSIAL) | Riwayat baris (cincin 4), Tab satu dan dua kali, Ctrl+C, Ctrl+L, Esc, fallback Indonesia satu-sumber, dokumen sinkron dengan audit G11; tiga belas test | 262 lulus | VERIFIED-ON-HOST dan VERIFIED-ON-CI-API SAJA. TIDAK ada VERIFIED-ON-VM untuk unit ini | `Analysis/reports/f7_navigasi_keyboard_uji_2026-10-22.txt` |

Angka "uji host tercatat" pada baris F1 sampai F6 adalah angka yang dicatat berkas
bukti unit itu sendiri, bukan hasil pengukuran ulang hari ini; ia tidak dapat
dijumlahkan satu sama lain. Angka yang berlaku sekarang adalah angka gerbang akhir
di Bagian 5.

## 2. Kenapa F7 berhenti di tengah mandat

Mandat F7 penuh menuntut perpindahan kursor horizontal pada framebuffer untuk
panah kiri/kanan, Home/End, dan gambar ulang kolom untuk Delete maju. Pemilik
memberi keputusan scope yang tertulis:

> "Saya kerjakan hanya bagian F7 yang tidak menuntut perpindahan kursor di
> framebuffer (riwayat, Tab, Ctrl+C/L, Esc, fallback Indonesia) plus dokumen, dan
> STOP sebelum F7 rampung supaya Bapak putuskan lagi."

Konsekuensi yang diterima bersama keputusan itu: unit F7 tidak menghasilkan ISO
baru, jadi tidak ada sesi QEMU, tidak ada layar yang disaksikan, dan tidak ada
angka lantai memori untuk tree F7. Bagian VM unit ini dinyatakan TERBLOKIR, dan
nyataan itu ditulis sebagai keterbukaan, bukan sebagai kelulusan yang diganti
namanya.

## 3. Blokade yang masih OPEN

- Perkakas bootloader penguasaan ISO tidak ada di pohon kerja. Keluaran persis:
  `ls: cannot access 'asl-kernel/target/limine_v8_binary': No such file or
  directory`. `scripts/build_kernel.ps1:100` dan
  `scripts/test_hv2_stack_overflow_qemu.ps1:28` menyusun jalur
  `target\limine_v8_binary\limine.exe`. Selama berkas itu tidak ada, TIDAK ADA
  unit yang bisa diberi bukti VERIFIED-ON-VM, sekecil apa pun perubahannya.
  Tercatat sebagai entri fault ke-50 (OPEN). Entri ke-49 juga masih OPEN.
- Gerbang regresi memori boot `-m 3M` TETAP MERAH pada cabang ini dan itu adalah
  Keputusan 2 pemilik (diterima dengan justifikasi tertulis, bukan diam-diam):
  pohon F5 dan F6 tercatat `TIDAK boot (serial 0 B)` pada 3 MB. Angka lantai
  memori yang beredar di dokumen unit adalah milik tree tempat angka itu diukur.
- ISO yang ada di disk adalah hasil unit F6, sehingga
  `scripts/test_ram_floor_3mb.ps1:47` tidak mengukur tree F7 maupun tree gerbang
  akhir ini. Tidak ada angka yang diwarisi diam-diam.
- Kelas VERIFIED-ON-HARDWARE tetap 0 sampai satu jendela disaksikan pemilik.

## 4. Kelas bukti yang dipakai dokumen ini

- VERIFIED-ON-HOST: terukur di mesin kerja oleh test unit, gerbang fmt atau
  clippy, atau build; tidak ada pengamatan mesin sasaran.
- VERIFIED-ON-CI-API: terukur pada hosted CI dan dibuktikan lewat ekstrak log run
  yang di-commit; runner bukan perangkat sasaran.
- VERIFIED-ON-VM: boot dan sesi serial terukur di mesin virtual pada ISO yang
  sidik jarinya tercatat.
- VERIFIED-ON-VM-VISIBLE: tambahan di atas VERIFIED-ON-VM berupa piksel
  screendump yang diukur, bukan dilihat sekilas.
- RANCANGAN: niat yang belum diimplementasikan.
- VERIFIED-ON-HARDWARE: nol pada seluruh rantai ini.

## 5. GERBANG AKHIR R1 (sisi host) - angka hari ini

Berkas: `Analysis/reports/r1_gerbang-akhir_host_2026-10-22.txt`.

- Diukur pada DUA rentang: unit terakhir `235d44b..HEAD` dan SELURUH cabang
  `b47c24c..HEAD`. Rentang penuh itulah yang menemukan tiga diagnostik clippy di
  baris hasil kerja, termasuk satu yang lolos dari laporan gerbang unit F4.
- Setelah perbaikan higiene: `cargo test --workspace --lib` = 30 suite, **262
  lulus, 0 gagal**; fmt = 0 deviasi BARU pada kedua rentang (warisan yang sengaja
  tidak disapu: 78, 28, dan 19 baris pada tiga berkas cakupan); clippy = 0
  diagnostik BARU pada kedua rentang dengan 134 peringatan warisan (turun dari
  137); build host = 0 error, 14 peringatan pada empat crate luar cakupan.
- ELF rilis: 312.912 B, SHA256
  `FB4EEB31E26B1CF9A289E4C697E6CEA8230CD83A1AD45424CD2312EE5EEE5D59`; tiga build
  bersih byte-identik, salah satunya sesudah sumber listrik kerja terputus di
  tengah unit dan pulih. Perbandingan bagian ELF dengan tip sebelumnya
  membuktikan `.text`, `.rodata`, dan `.data` identik isi: perbaikan tanda tangan
  dan `return` yang dibuang tidak mengubah satu pun byte kode mesin. Ukuran
  berkas sendiri bukan indikator dampak, karena dua build pada SATU tree yang
  sama pernah berbeda 16 byte di tabel nama simbol.
- hosted CI: pasangan atas tip `9750724` (run 39788285251 dan 39788292764,
  keduanya success, 262 lulus 0 gagal, ELF runner 312.968 B, coverage advisory
  lines 89,90 persen) sudah diekstrak dan di-commit pada unit gerbang akhir,
  sehingga utang G8 atas tip itu tertutup. Pasangan atas tip gerbang akhir itu
  sendiri (run 39844518151 dan 39844527147, keduanya success, 262 lulus 0 gagal,
  ELF runner 312.968 B sama seperti tip sebelumnya - cocok dengan `.text` yang
  terbukti identik - dan coverage TOTAL lines 89,90 persen) diekstrak bersama
  dokumen ini. Satu-satunya utang yang tersisa adalah pasangan atas tip tempat
  dokumen ini di-commit; itu bukan sesuatu yang bisa ditutup di dalam commit yang
  sama.
- Bagian VM pada unit ini: TERBLOKIR, seperti Bagian 3.

## 6. Backlog jujur (tidak dikerjakan, tidak dijanjikan)

Navigasi dan layar: PageUp/Down, Insert, F2-F12, penempelan (paste), timeout Esc
sehingga Esc tunggal terasa segera, penanda kursor untuk panah kiri/kanan, Home,
dan End (menuntut perpindahan kursor horizontal di framebuffer), gambar ulang
kolom untuk Delete maju, dan penerjemahan scan code terextend pada galang PS/2
sehingga riwayat baris bisa dipakai lewat galang, bukan hanya lewat jalur serial
ANSI.

Ukur dan alat: angka sekitar 550 byte struktur shell belum pernah terukur di
mesin; mengikat artefak gerbang ke hash tree supaya klaim "0 baru" sebuah unit
dapat dibuktikan berasal dari tree yang sama; menyapu 134 diagnostik clippy dan
deviasi fmt warisan; penguasaan ISO supaya setiap unit bisa diberi bukti VM.

## 7. Keputusan yang dibutuhkan dari pemilik

1. Apakah sisa F7 yang menuntut perpindahan kursor horizontal di framebuffer
   dikerjakan sebagai unit baru, atau mandat F7 ditutup sampai di sini.
2. Apakah blocker perkakas bootloader (entri #50) diselesaikan lebih dulu, supaya
   fitur baru dapat bukti VM. Tanpa itu, semua kerja fitur berikutnya hanya
   VERIFIED-ON-HOST.
3. Apakah penyesuaian alat dibuka dari backlog R2 sekarang (ikatan artefak ke
   hash tree dan penyapuan warisan), atau tetap menunggu fitur beku.
4. Kapan jendela berikutnya dijadwalkan untuk disaksikan, karena tanpa satu
   jendela pun kelas VERIFIED-ON-HARDWARE tetap 0.

## 8. Yang TIDAK dilakukan dan tidak akan dilakukan tanpa perintah

PR #6 dan PR #7 tidak di-merge. Tag v1.0.2 tidak dipotong. Tag dan rilis v1.1.0
tidak dihapus. `LAPORAN-FINAL-v1.0.2.md` tidak ditulis. `scripts/ci_local.ps1`
tidak dijalankan apa adanya. `scripts/g9_final_p8_run.ps1`, `CHECKLIST_STATE.json`,
dan 20 artefak `Analysis/reports/g9_final_p8_*` tanggal 2026-10-21 tidak disentuh.
Tidak ada commit yang diamend, tidak ada commit yang dibuang, dan tidak ada force
push: koreksi selalu berupa commit baru, termasuk koreksi atas klaim gerbang unit
F4 yang dibantah oleh gerbang akhir.

Rantai eksekusi mandat: BAGIAN 0, BAGIAN 1 (E8), BAGIAN 2 (F1-F7), lalu BERHENTI.
Dokumen ini adalah akhir dari rantai itu. Agen berhenti di sini dan menunggu
keputusan pemilik pada Bagian 7.
