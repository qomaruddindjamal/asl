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
sebelas perintah kanonik plus perintah lama yang tetap dikenali. Tidak ada
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
4. Bila jumlah argumen di luar yang diizinkan, shell mencetak baris `Sintaks:`
   yang diambil dari registry, bukan dari teks yang ditulis tangan.
5. Ruang kerja (direktori saat ini) dipakai oleh perintah berkas relatif; jalur
   mutlak diawali `/`.

## Pasal 4 - Perintah kanonik (sebelas, urutan registry)

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

Sebelas perintah kanonik di atas dibangun di atas kerangka registry baru.
Perintah warisan TIDAK diubah dan TIDAK dihapus pada v1.0.2; keduanya tetap
bekerja lewat jalur pengalihan lama: `meminfo`, `cpuinfo`, `uname`, `lspci`,
`lsblk`, `fdisk`, `mount`, `failover`, `reboot`, `halt`, `wfi`, `hlt`,
`test_*`, `trap_*`. Keluarannya tetap seperti sebelumnya.

## Pasal 6 - Batasan dan risiko yang diketahui

- RAMDISK adalah penyimpanan volatil: tanpa daya, tanpa isi.
- Kapasitas total 262144 byte dengan satu slot berkas 8192 byte; satu berkas
  tidak dapat melebihi satu slot pada build ini.
- Pembuatan/penghapusan berkas pada FAT32 bukan bagian v1.0.2 (lingkup STRETCH
  yang diputuskan dilepas; tercatat di laporan akhir misi).
- Tombol panah dan Home/End dikirim sebagai sekuens escape; bila jalur masukan
  tidak mengirim sekuens escape, karakter penggerak tetap dimasukkan ke baris.
  Mesin penyuntingnya sudah teruji; yang bergantung pada perangkat adalah
  pengiriman sekuens.
- Riwayat baris (panah atas/bawah) sengaja tidak dijanjikan dan tidak
  diimplementasikan.
