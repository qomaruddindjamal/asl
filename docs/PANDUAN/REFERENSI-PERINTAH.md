# REFERENSI PERINTAH COCKPIT SHELL v1.0.2

Dokumen referensi perintah shell interaktif ASL Core Kernel. Seluruh baris
`Nama`, `Sintaks`, `Ringkasan`, `Alias`, `Argumen`, dan `Kelompok` di bawah
adalah salinan persis dari registry satu-sumber di dalam kode (`REGISTRY` pada
`asl-kernel/asl-cockpit-shell/src/lib.rs`). Sinkronnya dijaga oleh dua test host:
`test_docs_command_reference_matches_registry` (isi registry, prompt, dan identitas
rilis) serta `test_docs_capacity_figures_match_constants` (angka kapasitas pada
Pasal 2 dan Pasal 6 dibaca dari konstanta RAMDISK). Kalau registry atau konstanta
berubah dan dokumen ini tidak, test menjadi MERAH. Dokumen ini tidak boleh
menambahkan perintah yang tidak ada di registry.

## Pasal 1 - Tujuan dan lingkup

Memberi satu tempat baca bagi operator untuk mengetahui perintah apa yang
dijanjikan shell pada konsol serial dan framebuffer, bagaimana bentuknya, dan
apa yang dilakukan masing-masing. Lingkupnya hanya Cockpit Shell v1.0.2:
dua belas perintah kanonik plus perintah lama yang tetap dikenali. Tidak ada
skrip, tidak ada berkas di disk, tidak ada jaringan.

## Pasal 2 - Prasyarat

- Mesin berjalan dari ISO rilis pada QEMU (`q35`) atau perangkat yang didukung.
- Konsol serial 115200 8N1 terhubung, atau keluaran framebuffer terlihat.
- Prompt siap diterima: `ASL-KERNEL />_ ` pada akar RAMDISK (satu-sumber:
  konstanta `SHELL_PROMPT`, disusun oleh `render_prompt` dari
  `SHELL_PROMPT_PREFIX` + posisi aktif + `SHELL_PROMPT_SUFFIX`). Prompt
  bergerak bersama direktori kerja: sesudah `cd log` ia menjadi
  `ASL-KERNEL /log>_ `.
- Penyimpanan tempat perintah berkas bekerja adalah RAMDISK volatil berkapasitas
  262144 byte (256 KiB). Isi hilang ketika mesin dimatikan.

## Pasal 3 - Prosedur pemakaian

1. Ketik nama perintah atau salah satu aliasnya.
2. Tekan ENTER (`\r` atau `\n` keduanya diterima).
3. Baris dipecah menjadi maksimal dua argumen; argumen terakhir pada
   `write-file` menangkap seluruh sisa baris apa adanya.
4. Bila jumlah argumen di luar yang diizinkan, shell mencetak baris galat berlabel
   `Kesalahan:` yang memuat sintaks dari registry (`Sintaks: ...`), bukan teks yang
   ditulis tangan. Bentuk lengkapnya ada di Pasal 7.
5. Ruang kerja (direktori saat ini) dipakai oleh perintah berkas relatif; jalur
   mutlak diawali `/`. `up` menaiki satu tingkat ke direktori induk; di akar ia
   tidak berpindah dan mengatakan begitu.

## Pasal 4 - Perintah kanonik (dua belas, urutan registry)

Urutan di bawah sama dengan urutan tampil pada perintah `help`.

### help
- Nama: `help`
- Sintaks: `help [perintah]`
- Ringkasan: Daftar perintah yang disusun langsung dari registry; dengan argumen menampilkan satu perintah.
- Alias: -
- Argumen: paling sedikit 0, paling banyak 1
- Kelompok: Bantuan

### clear
- Nama: `clear`
- Sintaks: `clear`
- Ringkasan: Bersihkan area tampilan konsol framebuffer dan kirim sekuens bersih layar ke serial.
- Alias: -
- Argumen: paling sedikit 0, paling banyak 0
- Kelompok: Tampilan

### where
- Nama: `where`
- Sintaks: `where`
- Ringkasan: Tampilkan direktori kerja aktif; posisi yang sama juga tampil pada prompt.
- Alias: -
- Argumen: paling sedikit 0, paling banyak 0
- Kelompok: Ruang kerja

### list
- Nama: `list`
- Sintaks: `list [jalan]`
- Ringkasan: Daftar isi direktori pada RAMDISK volatil; tanpa argumen memakai direktori kerja.
- Alias: `ls`, `dir`
- Argumen: paling sedikit 0, paling banyak 1
- Kelompok: Ruang kerja

### open
- Nama: `open`
- Sintaks: `open <jalan>`
- Ringkasan: Masuk ke direktori bila tujuannya direktori, atau tampilkan isi berkas bila tujuannya berkas.
- Alias: `cd`, `cat`, `type`, `read-file`, `open-file`
- Argumen: paling sedikit 1, paling banyak 1
- Kelompok: Ruang kerja

### up
- Nama: `up`
- Sintaks: `up`
- Ringkasan: Naik satu tingkat ke direktori induk; sudah di akar berarti tetap di akar.
- Alias: -
- Argumen: paling sedikit 0, paling banyak 0
- Kelompok: Ruang kerja

### create-folder
- Nama: `create-folder`
- Sintaks: `create-folder <nama>`
- Ringkasan: Buat direktori baru di dalam direktori kerja saat ini.
- Alias: `mkdir`
- Argumen: paling sedikit 1, paling banyak 1
- Kelompok: Berkas dan direktori (RAMDISK volatil)

### create-file
- Nama: `create-file`
- Sintaks: `create-file <nama>`
- Ringkasan: Buat berkas kosong di dalam direktori kerja saat ini.
- Alias: -
- Argumen: paling sedikit 1, paling banyak 1
- Kelompok: Berkas dan direktori (RAMDISK volatil)

### write-file
- Nama: `write-file`
- Sintaks: `write-file <nama> <teks>`
- Ringkasan: Tulis satu baris teks ke berkas dan memotong sisa isi lama; isi hidup selama RAMDISK.
- Alias: -
- Argumen: paling sedikit 1, paling banyak 2
- Kelompok: Berkas dan direktori (RAMDISK volatil)

### delete-file
- Nama: `delete-file`
- Sintaks: `delete-file <nama>`
- Ringkasan: Hapus satu berkas dari direktori kerja saat ini; slot datanya ternol dan bisa dipakai ulang.
- Alias: `rm`
- Argumen: paling sedikit 1, paling banyak 1
- Kelompok: Berkas dan direktori (RAMDISK volatil)

### delete-folder
- Nama: `delete-folder`
- Sintaks: `delete-folder <nama>`
- Ringkasan: Hapus satu direktori kosong dari direktori kerja saat ini; direktori berisi ditolak.
- Alias: -
- Argumen: paling sedikit 1, paling banyak 1
- Kelompok: Berkas dan direktori (RAMDISK volatil)

### status
- Nama: `status`
- Sintaks: `status`
- Ringkasan: Ringkas identitas rilis, berkas kerja shell, direktori kerja, dan pemakaian RAMDISK.
- Alias: -
- Argumen: paling sedikit 0, paling banyak 0
- Kelompok: Status dan identitas

## Pasal 5 - Perintah lama yang tetap dikenali

Dua belas perintah kanonik di atas dibangun di atas kerangka registry baru.
Perintah warisan TIDAK diubah dan TIDAK dihapus pada v1.0.2; keduanya tetap
bekerja lewat jalur pengalihan lama: `meminfo`, `cpuinfo`, `uname`, `lspci`,
`lsblk`, `fdisk`, `mount`, `failover`, `reboot`, `halt`, `wfi`, `hlt`,
`test_*`, `trap_*`. Keluarannya tetap seperti sebelumnya.

## Pasal 6 - Batasan dan risiko yang diketahui

- RAMDISK adalah penyimpanan volatil: tanpa daya, tanpa isi.
- Kapasitas total 262144 byte dengan satu slot berkas 8192 byte; satu berkas
  tidak dapat melebihi satu slot pada build ini.
- Satu token argumen paling panjang 64 byte, sama dengan batas nama berkas.
  Yang melebihi batas DITOLAK dengan baris galat (nama) atau diumumkan
  potongannya (isi) — lihat Pasal 7; tidak ada pemotongan diam-diam lagi.
  Pada satu baris perintah, nama yang mengandung spasi tidak dapat diketik
  karena spasi adalah pemisah argumen. Sejak unit F5 nama berspasi BISA
  diketik lewat mode berpanduan, karena baris panduan dibaca apa adanya
  sebagai satu nama — lihat MODE BERPANDUAN di Pasal 7.
- Pembuatan/penghapusan berkas pada FAT32 bukan bagian v1.0.2 (lingkup STRETCH
  yang diputuskan dilepas; tercatat di laporan akhir misi).
- Tombol panah dan Home/End dikirim sebagai sekuens escape; bila jalur masukan
  tidak mengirim sekuens escape, karakter penggerak tetap dimasukkan ke baris.
  Mesin penyuntingnya sudah teruji; yang bergantung pada perangkat adalah
  pengiriman sekuens.
- Riwayat baris (panah atas/bawah) sengaja tidak dijanjikan dan tidak
  diimplementasikan.

## Pasal 7 - Keluaran shell: satu label galat dan jumlah yang diumumkan

Prinsip (unit F3): operator tidak boleh menebak apa yang barusan terjadi.
Setiap baris keluaran kanonik disusun dari konstanta teks satu-sumber di
`asl-kernel/asl-cockpit-shell/src/lib.rs` (`MSG_*`), dan teks yang sama
tercatat di bawah ini; test `test_f3_labels_keluaran_satu_sumber` dan
`test_f3_dokumen_memuat_setiap_label_keluaran` menjadi MERAH bila kode dan
dokumen berpisah.

#### Aturan galat

Satu label untuk semua kegagalan: `Kesalahan: ` disusul sasaran lalu
penjelasan. Tidak ada lagi label kedua (`Galat:` sudah hilang dari shell).

| Keadaan | Baris yang tercetak |
| --- | --- |
| Tujuan tidak ada | `Kesalahan: <jalan> tidak ditemukan` |
| Tujuan sudah ada | `Kesalahan: <jalan> sudah ada` |
| Berkas diperlakukan sebagai direktori | `Kesalahan: <jalan> objek ini bukan direktori` |
| Direktori diperlakukan sebagai berkas | `Kesalahan: <jalan> objek ini direktori` |
| Direktori masih berisi | `Kesalahan: <jalan> direktori masih berisi` |
| RAMDISK penuh | `Kesalahan: <jalan> ruang RAMDISK habis` |
| Sistem berkas hanya-baca | `Kesalahan: <jalan> sistem berkas hanya-baca` |
| Path tidak sah | `Kesalahan: <jalan> path tidak sah` |
| Perangkat penyimpanan gagal | `Kesalahan: <jalan> galat perangkat penyimpanan` |
| Hasil normalisasi path terlalu panjang | `Kesalahan: <argumen> - path hasil normalisasi melebihi batas 128 byte` |
| Argumen nama terpotong saat pembacaan baris | `Kesalahan: <64 byte pertama>... - argumen melebihi batas 64 byte, perintah tidak dijalankan` |
| Jumlah argumen tidak sesuai | `Kesalahan: jumlah argumen tidak sesuai - Sintaks: <sintaks registry>` |
| Nama perintah tidak dikenal (`help <nama>`) | `Kesalahan: perintah tidak dikenal: <nama>` |

Sasaran (`<jalan>`) adalah jalur hasil normalisasi yang benar-benar dipakai ke
VFS, jadi operator tahu objek mana yang gagal tanpa mengingat ketikannya.

#### Keluaran sukses

| Perintah | Baris yang tercetak |
| --- | --- |
| `where` | `Ruang kerja: <jalan>` |
| `up` (berhasil) | `Ruang kerja: <jalan>` |
| `up` di akar | `Ruang kerja tetap di akar: /` |
| `open <direktori>` | `Ruang kerja: <jalan>` |
| `open <berkas>` | `Isi <jalan> (<n> byte):` lalu isi; berkas nol byte mencetak `(kosong)` |
| `list` | `Daftar isi <jalan> (<n> entri):` lalu satu baris per entri |
| `create-folder` | `Direktori dibuat: <jalan> (0 entri)` |
| `create-file` | `Berkas dibuat: <jalan> (0 byte)` |
| `write-file` berkas baru | `Tertulis <n> byte di <jalan> (berkas baru)` |
| `write-file` menimpa, isi lama lebih panjang | `Tertulis <n> byte di <jalan> - <m> byte isi lama dihapus` |
| `write-file` menimpa, isi lama tidak dipotong | `Tertulis <n> byte di <jalan>` |
| `write-file` dengan teks melebihi batas argumen | `Tertulis 64 byte di <jalan> - isi dipotong pada batas 64 byte; sisa tidak tertulis` |
| `delete-file` | `Berkas dihapus: <jalan> (<n> byte dibebaskan)` |
| `delete-folder` | `Direktori dihapus: <jalan> (0 entri)` |

#### Batas panjang argumen dan nama berkas (F4)

Satu token argumen pada satu baris perintah paling panjang 64 byte. Batas ini
sengaja sama dengan batas nama berkas pada RAMDISK, jadi nama yang muat
diketik selalu muat dipakai dan tidak ada batas kedua yang diam-diam memotong.

Perilaku lama (bug F4): token yang lebih panjang dari 64 byte DIPOTONG tanpa
pengumuman, sehingga `write-file laporan-...-terakhir-sebelum-terbit.txt ...`
menciptakan berkas ber-nama 64 byte tanpa ekstensi. Sekarang potongan itu
BERLABEL:

- Nama/path (argumen pertama) terpotong: perintah TIDAK dijalankan sama
  sekali, dan barisnya `Kesalahan: <potongan>... - argumen melebihi batas 64
  byte, perintah tidak dijalankan`. Tidak ada berkas bernama rusak yang
  tercipta.
- Isi (argumen kedua `write-file`) terpotong: bagian yang muat tetap ditulis
  dan sisanya diumumkan pada baris yang sama, `- isi dipotong pada batas 64
  byte; sisa tidak tertulis`.

Isi berkas yang ingin lebih dari 64 byte ditulis bertahap (`open`/`write-file`
ulang) — batas ini batas satu baris ketikan, bukan batas ukuran berkas; ukuran
berkas dibatasi satu slot RAMDISK 8192 byte (Pasal 6). Mode berpanduan F5
memakai jalur panjang yang sama: satu nama paling panjang 64 byte, dan isi
berhenti pada kapasitas satu slot.

#### Janji `write-file` tanpa argumen teks

`write-file <nama>` tanpa teks SAH: berkas (dibuat bila belum ada) ditulis
dengan nol byte, sehingga isi lama hilang. Karena pemangkasan itu tidak dapat
diambil kembali, shell mengumumkannya pada baris yang sama: angka `<m> byte
isi lama dihapus` hanya muncul bila tulisan lebih pendek daripada isi lama,
dan `write-file` tanpa teks pada berkas berisi selalu mencetaknya.

#### MODE BERPANDUAN: label konteks sesudah galat argumen (F5)

Prinsip (keputusan pemilik): pintu mode berpanduan terbuka HANYA sesudah baris
galat arity tercetak. Baris `Kesalahan: jumlah argumen tidak sesuai - Sintaks:
<sintaks registry>` tetap persis seperti sebelumnya dan tetap menjadi
penjelasan pertama yang dilihat operator; shell tidak pernah menghapus atau
menunda galat itu. Sesudah baris itu, untuk tiga perintah pembuatan shell
menambahkan satu label konteks dan memakai baris berikutnya sebagai argumen
yang hilang.

| Perintah | Label yang dicetak | Baris kosong berarti |
| --- | --- | --- |
| `create-folder` (tanpa nama) | `Nama folder baru: ` | batal |
| `create-file` (tanpa nama) | `Nama berkas: ` | batal |
| `write-file` (tanpa nama) | `Nama berkas: ` | batal |
| `write-file` (nama sudah diterima) | `Isi <nama> (baris <n>): ` dengan `<n>` mulai dari satu | selesai, isi ditulis |

Baris panduan dibaca apa adanya, jadi nama boleh mengandung spasi
(`laporan tahunan 2026`), dan baris isi dijaga urutan serta spasinya; tiap baris
isi yang diterima disusun dengan pemisah ganti baris sehingga berkas berisi
multi-baris. `create-file` berhenti pada nama: perintah itu memang dibuat untuk
berkas kosong.

Baris keluaran mode berpanduan:

| Keadaan | Baris yang tercetak |
| --- | --- |
| Nama direktori kosong | `Dibatalkan: nama folder kosong - direktori tidak dibuat` |
| Nama berkas kosong | `Dibatalkan: nama berkas kosong - tidak ada yang ditulis` |
| Nama panduan melebihi satu token | `Kesalahan: nama lebih panjang dari batas 64 byte - perintah tidak dijalankan` |
| Isi panduan memenuhi satu slot | `Isi berhenti pada kapasitas satu slot 8192 byte; sisa baris tidak diterima`, disusul baris hasil `Tertulis ...` |

Contoh lengkap satu sesi (keluaran serial, satu baris per tulis):

```
ASL-KERNEL />_ create-folder
Kesalahan: jumlah argumen tidak sesuai - Sintaks: create-folder <nama>
Nama folder baru: laporan tahunan 2026
Direktori dibuat: /laporan tahunan 2026 (0 entri)
ASL-KERNEL />_ write-file
Kesalahan: jumlah argumen tidak sesuai - Sintaks: write-file <nama> <teks>
Nama berkas: notes.txt
Isi notes.txt (baris 1): baris pertama
Isi notes.txt (baris 2): baris kedua
Isi notes.txt (baris 3):
Tertulis 26 byte di /notes.txt (berkas baru)
ASL-KERNEL />_
```

Nama dan isi yang tertolak tidak pernah berjalan setengah: penolakan terjadi
sebelum VFS disentuh. Setelah aksi terminal (dibuat/ditulis/batal), mode
berpanduan berhenti sendiri dan prompt kembali seperti biasa. Perintah baca dan
hapus (`open`, `list`, `delete-file`, `delete-folder`) sengaja TIDAK dibimbing:
baris kosong pada perintah penghapusan akan berarti menghapus tanpa sasaran.

#### Penanda kursor saat idle (F5)

Prompt kanonik `ASL-KERNEL <posisi>>_ ` sudah memuat glif `_` sebagai penanda
tempat ketikan (satu-sumber: `SHELL_PROMPT_SUFFIX`). Unit F5 membuat penanda itu
hidup: sel tempat `_` pada layar digambar ulang setiap detik mengikuti jam
perangkat keras (CMOS RTC), menyala satu detik dan padam satu detik
(`kursor_nyala`: `detik % 2 == 0`).

Batas yang dinyatakan jujur:

- Blink hanya menyentuh framebuffer. Tidak ada satu byte pun yang bertambah
  pada keluaran serial, jadi log serial, cincin bukti `test_ram_floor_3mb`, dan
  pemarca penanda alat jendela tetap byte-identik dengan sebelum F5.
- Sumber detiknya jam CMOS, bukan pencacah tick APIC, karena galang shell idle
  berjalan dengan interrupt dimatikan sehingga pencacah tick tidak naik.
- Sel penanda mengikuti titik tulis: sesudah prompt, sesudah label panduan, dan
  sesudah satu karakter diketik. Selama satu baris sedang diedit, penanda tidak
  mengikuti kursor di dalam baris (panah kiri/kanan, Home/End adalah lingkup
  unit F7).
- Tidak ada sinkronisasi dengan terminal serial: yang disaksikan adalah layar
  framebuffer (QEMU `screendump`), bukan jendela HyperTerminal.



