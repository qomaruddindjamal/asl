# ASL v1.0.2 — E8: AKAR PENYEBAB DAN STATUS PEKERJAAN

> **LABEL DOKUMEN: STATUS PEKERJAAN — BUKAN LAPORAN FINAL E8.**
> E8 **belum ditutup**. Penutupan menuntut LANGKAH 6 (jendela G9-LITE disaksikan
> pemilik) dan teks panic dari pemilik seperti yang diminta MASTER PROMPT
> R1-REVISI Bagian 1. Angka di dokumen ini berasal dari artefak yang
> tercantum di Bagian 6, bukan dari tebakan.

Tanggal kerja: 2026-10-21. Cabang: `feature/v1.0.2-r1`. Kelas bukti yang
dipakai: VERIFIED-ON-HOST (kompilasi/unit test di mesin pengembangan),
VERIFIED-ON-VM (QEMU pada mesin ini, berjendela atau lewat berkas serial),
RANCANGAN (dinyatakan sebagai rencana, belum diukur).

---

## 1. Status satu paragraf

Lampiran bukti pemilik untuk E8 kosong, jadi jalan resmi adalah reproduksi
mandiri (LANGKAH 1). Hasil terukur: **tidak ada satu pun baris masukan
pengguna yang menghasilkan panic pada jalur perintah kanonik**, tetapi ada
**satu kegagalan yang ter-reproduksi dan ber-akar penyebab terverifikasi**,
yaitu pembekuan shell selama **178.779 ms** akibat `test_task`. Di luar itu,
audit statis menemukan **empat lokasi panic yang dapat dipicu satu ketikan**
di rangkaian Gate 3D dan **satu pemulihan keadaan yang bolong** di
`test_apic`. Semuanya sudah diubah menjadi pesan galat + prompt kembali
(LANGKAH 3), dijaga enam test regresi baru (LANGKAH 4), dan panic handler
kernel kini mencetak konteks penuh (pesan + lokasi sumber + alamat + rekaman
jebakan terakhir) supaya panic berikutnya tidak datang tanpa bukti.

**E8-STATUS: BELUM-TERREPRODUKSI-SEBAGAI-PANIC.** Pemilik diminta menempel
teks panic yang dilihatnya (LANGKAH 6) supaya cocok dengan salah satu jalur
di Bagian 3 atau membuka jalur baru.

---

## 2. LANGKAH 1 — reproduksi sistematis

Alat: `scripts/e8_adversarial_run.ps1` (satu-satunya alat yang dibuat untuk
E8). QEMU dihidupkan **berjendela** (`-machine q35 -vga std -cdrom ISO`),
masukan diumpan ke COM1 lewat `chardev socket` karena kernel mempoll port
serial itu, dan setiap kasus diukur: byte dikirim, milidetik sampai prompt
kembali, balasan terpotong, dan pemindaian penanda crash
(`ASL-CORE-PANIC`, `ASL EXCEPTION`, `FATAL`).

| Lintasan | Isi | Hasil terukur |
|---|---|---|
| Pass A | 41 kasus utama + 2 pembersih (43 payload) = daftar pemilik: baris kosong, spasi saja, 512/1024/4096+ karakter, argumen `..`, `//`, trailing slash, titik, kutip, hubung; open/create/write/rename/delete pada segala sudut; perintah tak dikenal; argumen berlebih; rentetan cepat; byte kendali `00 07 08 127 27 91 65` | KELUAR 0. Tidak ada panic, tidak ada hang. Semua kasus kembali ke prompt. |
| Pass B | Pass A + 16 diagnostik perintah lama (`diagnostik=True`, 59 kasus) | KELUAR 2. Satu kasus **GANTUNG**: `D13 test_task` tidak mengembalikan prompt dalam batas 150 s. |
| Pass D | Satu token `test_task` di boot segar `-m 256M` | Prompt kembali setelah **178.779 ms** (tercatat pada kolom `detik_mm` berkas TSV). Tidak ada penanda crash. |

Artifak (waktu setempat pada saat tulis):
`Analysis/reports/e8_adversarial_{summary,cases,serial,qemu_stderr}_2026-10-21_19-42-51.*`,
`..._2026-10-21_20-00-37.*`, `..._2026-10-21_20-09-52.*`.

Yang **tidak** dapat direproduksi: panic yang dilaporkan pemilik. Empat lokasi
panic di Bagian 3 adalah temuan audit statis (kode benar-benar memakai
`assert_eq!`/`unwrap()` pada keadaan yang bisa berbeda), bukan tangkapan layar.

---

## 3. LANGKAH 2 — tabel akar penyebab, lokasi, risiko, tindakan

Nomor "sebelum" merujuk tip cabang sebelum pekerjaan E8 (`git show HEAD:...`);
nomor "sesudah" merujuk berkas kerja saat dokumen ini ditulis.

| # | Lokasi sebelum | Kondisi pemicu | Kelas | Risiko | Tindakan (lokasi sesudah) |
|---|---|---|---|---|---|
| A1 | `asl-kernel/asl-kernel-core/src/lib.rs:2617-2621` | Ketikan `test_task`; TEST 1 membandingkan tata letak bingkai interupsi dengan `assert_eq!` | `[ASL-CORE-PANIC]` | Satu asersi tata letak membunuh kernel padahal perintah itu hanyalah alat uji yang dapat diketik siapa pun | Diperiksa lunak + `return`; cetak `Kesalahan: tata letak frame interupsi tidak sesuai sangkaan - rangkaian Gate 3D dihentikan, shell tetap hidup` (sesudah: `lib.rs:2700-2708`) |
| A2 | `.../lib.rs:2678` | Ketikan `test_task`; `spawn_task(0x2000).unwrap()` pada TEST 11 | `[ASL-CORE-PANIC]` | `Result` dibuang ke `unwrap()`; kegagalan pembuatan tugas = panic | `match` atas `Ok/Err`, galat dicetak sebagai pesan, TEST 11 dilewati; indeks dihitung dengan `checked_sub` + penyaring batas `MAX_KERNEL_TASKS` (sesudah: `lib.rs:2765-2802`) |
| A3 | `.../lib.rs:2759-2764` | Ketikan `test_task` untuk **kedua kalinya** dalam satu sesi; `GLOBAL_TASK_MANAGER` tidak pernah dinolkan antar-panggilan, jadi enam `assert_eq!` keadaan penjadwal tidak lagi terpenuhi | `[ASL-CORE-PANIC]` | Ini jalur paling mungkin di balik laporan pemilik: panic tanpa pesan di layar karena panic handler lama tidak mencetak konteks | Enam sangkaan jadi satu periksa lunak; bila kotor: `preemptive_enabled=false`, `cli`, cetak `Kesalahan: keadaan penjadwal global tidak bersih - ...`, `return` (sesudah: `lib.rs:2868-2883`) |
| A4 | `.../lib.rs:2794` (loop target 100.000 tukar) | Ketikan `test_task` | **Pembekuan** (bukan panic) | Shell tidak menerima masukan selama ~179 s; tanpa jalan keluar; pada `-m 3M` belum tentu selesai | Loop memantau masukan operator tiap iterasi lewat penolong baru `suite_masukan_masuk()` (COM1 LSR bit 0 atau PS/2 status bit 0); bila ada ketikan: hentikan, kembalikan keadaan tugas, habiskan satu scancode menggantung, cetak `Kesalahan: uji praemption Gate 3D dihentikan operator sebelum 100.000 tukar - kembali ke prompt, shell tetap hidup`, `return` (sesudah: `lib.rs:2684-2689`, `2912-2921`, `2947-2959`) |
| A5 | `.../lib.rs:2262-2277` (cabang `test_apic`) | Ketikan `test_apic` | Kebocoran keadaan | Bendera interupsi CPU (IF) dibiarkan menyala sesudah satu perintah; sisa sesi berjalan dengan keadaan yang berubah diam-diam dan dapat memunculkan tick di tempat yang tidak diuji | `cli` + satu baris laporan `[TEST 7] Bendera interupsi CPU dipulihkan (cli) -> PASS` sesudah observasi tick (sesudah: `lib.rs:2275-2288`) |
| A6 | `.../lib.rs:916-924` (panic handler lama) | Panic apa pun, dari jalur mana pun | Bukti hilang | Handler lama mencetak satu baris tetap lalu berputar; `PanicInfo` (pesan + `berkas:baris:kolom`) dibuang, jadi pemilik tidak punya bahan untuk laporan | Handler baru menulis `KONTEKS:` dari `PanicInfo` ke serial **dan** framebuffer lewat `PanicContextSink`, lalu `ALAMAT frame penanganan` dan `JEBAKAN TERAKHIR` (vektor/RIP/CR2/kode galat) dari `asl_hal::x86_fault::LAST_TRAP`; penanda reentransi mencegah panic dalam pencetakan konteks berputar tanpa bukti (sesudah: `lib.rs:913-980`) |

### 3.1 Lokasi yang diaudit dan dinyatakan aman (nol tindakan)

- Jalur kanonik shell → jembatan → VFS → RAMDISK **fail-closed**: tidak ada
  `panic!`/`unwrap()`/`expect()`/indeks lepas di cabang masukan pengguna.
  `parse_line` mengembalikan `NotCanonical | Ready | BadArity`
  (`asl-cockpit-shell/src/lib.rs:707-768`), argumen dipotong ke
  `MAX_ARG_LEN` (`:684-701`), dan RAMDISK menolak nama/path lewat
  `VfsError` (`asl-storage/src/ramdisk.rs:321-363`, `:371-391`).
- `RamFileSystem::<0, N>::new()`/`ensure_root()` akan mengakses `nodes[0]`
  pada tabel nol simpul (`ramdisk.rs:211-240`). **Tidak dapat dicapai**:
  kernel hanya menginstansiasi `RAMDISK_DEFAULT_NODES = 32`, dan menambah
  penjaga untuk keadaan yang tidak mungkin bukan bagian cakupan E8.
- `Invocation::entry()` jatuh ke `REGISTRY[0]` bila registry kosong
  (`asl-cockpit-shell/src/lib.rs:660-669`). **Tidak dapat dicapai**:
  registry adalah susunan malaran panjang `CANONICAL_COMMAND_COUNT`.

### 3.2 Temuan tanpa panic (didaftarkan, tidak diubah di E8)

| Temuan | Lokasi | Mengapa tidak ditutup di E8 |
|---|---|---|
| `write-file <nama>` tanpa isi **memangkas berkas menjadi 0 byte** | `asl-storage/src/vfs.rs:233-245` | Ini perilaku, bukan crash. Masuk cakupan F3 (keluaran eksplisit) dan F4; harus diumumkan ke pemilik sebelum diterima sebagai janji. |
| Isi `write-file` lebih dari 64 byte dipotong diam-diam; karena itu tepi 8192/8193 byte tidak dapat diketik dari shell | `asl-cockpit-shell/src/lib.rs:684-701` | Yang bisa dilakukan E8 adalah membuktikan tepi 8192/8193 di tingkat RAMDISK (dilakukan, lihat Bagian 4). Batas potong = keputusan F1-F7. |
| RAMDISK menerima nama berisi byte NUL | `asl-storage/src/ramdisk.rs:331-336` | Validasi tambahan di luar cakupan E8; tercatat sebagai keputusan terbuka untuk F2/F3. |
| Teks balasan lama masih berbahasa Inggris (`Unknown command. Type 'help' ...`) | `asl-cockpit-shell` jalur lama | Aturan G10 (100% bahasa Indonesia) ditegakkan pada F1-F7, bukan pada E8. |

---

## 4. LANGKAH 3 + LANGKAH 4 — perbaikan dan penjaga regresi

Perbaikan (.rs, semuanya dalam cakupan kernel/shell/VFS/ramdisk/test):
`asl-kernel-core/src/lib.rs` (A1-A6) dan penjaga regresi di
`asl-cockpit-shell/src/lib.rs` serta `asl-storage/src/ramdisk.rs`.

Enam test baru, semua bernama `panic_guard_*` (aturan penamaan misi):

| Test | Berkas | Menjaga |
|---|---|---|
| `panic_guard_baris_sangat_panjang_dari_pengguna` | `asl-cockpit-shell/src/lib.rs` | 4096 ketikan tanpa jeda dan baris 100.000 karakter: penyangga editor berhenti tumbuh, parser tetap mengembalikan nilai, argumen terpotong ke `MAX_ARG_LEN` |
| `panic_guard_byte_kendali_nul_dan_utf8_rusak` | `asl-cockpit-shell/src/lib.rs` | Byte `00 07 08 09 1B 5B 7F 80 C3 FF` lewat tiga jalur masuk (`edit`, `handle_char`, `handle_input`) dan NUL/ESC di tengah argumen |
| `panic_guard_argumen_berlebih_kutip_dan_indeks_di_luar` | `asl-cockpit-shell/src/lib.rs` | Sepuluh kata ekstra untuk **setiap** perintah registry, kutip/garis miring menggantung, indeks argumen di luar `argc` |
| `panic_guard_isi_tepat_satu_slot_dan_satu_lebih` | `asl-storage/src/ramdisk.rs` | Tepi yang tidak dapat diketik dari shell: 8192 byte sukses, satu byte lebih (`offset=1`) `NoSpaceLeft`, `truncate` 8193 `NoSpaceLeft`, offset `u64::MAX` dan `u64::MAX-4` menghasilkan 0 byte / `NoSpaceLeft`, ukuran tidak berubah oleh percobaan gagal |
| `panic_guard_nama_dan_path_adversarial` | `asl-storage/src/ramdisk.rs` | Nama wajib tolak (`""`, `.`, `..`, `/`, `//`, `./`, `a/b`, `/a`, `a/`, 65 byte) + nama yang diterima apa adanya (`..x`, `.x`, `x..`, NUL, `a b`) + dua belas path menantang lewat `resolve_path`/`lookup` (termasuk 400 garis miring) |
| `panic_guard_tulis_dan_hapus_pada_jenis_salah` | `asl-storage/src/ramdisk.rs` | `write-file` pada folder, `read_file`/`truncate` pada folder, `delete-file` pada berkas yang sudah hilang, `delete-folder` pada berkas, induk tidak ada, berkas tidak ada |

Test ini lulus tanpa `catch_unwind`: fakta bahwa proses uji selesai adalah
buktinya (panic akan menggagalkan test).

---

## 5. Gerbang yang sudah diukur pada langkah ini

| Gerbang | Perintah | Hasil | Dasar terima |
|---|---|---|---|
| Unit test host | `cargo test --workspace --lib` | **202 lulus, 0 gagal** (196 lama + 6 baru) | ambang misi: ≥196 + test baru |
| Build kernel | `cargo build --package asl-kernel-bin --target x86_64-unknown-none --release` | selesai, **nol baris warning** | 0/0 |
| rustfmt | `rustfmt --edition 2021 --check` per berkas yang saya ubah (hanya-baca, di tempat), diiris dengan rentang baris milik `git diff -U0`; alat: `scripts/r1_new_only_gate_check.py fmt` | **nol deviasi BARU**: 0 irisan dari 130 baris `asl-cockpit-shell/src/lib.rs`, 0 dari 167 baris `asl-kernel-core/src/lib.rs`, 0 dari 154 baris `asl-storage/src/ramdisk.rs`. Warisan pada berkas yang sama: rustfmt masih ingin mengubah 28 / 107 / 19 baris masing-masing | lihat catatan dasar di bawah |
| clippy | `cargo clippy --workspace --lib --target x86_64-unknown-none --message-format=short -- -A clippy::not_unsafe_ptr_arg_deref` disimpan ke `Analysis/reports/E8-CLIPPY-WORKSPACE-ALLOW-2026-10-21.txt` (193 baris), diiris dengan `scripts/r1_new_only_gate_check.py clippy` | **0 diagnostik di baris hasil kerja saya**; 137 diagnostik warisan di seluruh workspace (terbanyak: `asl-storage/src/fat32.rs` 16, `asl-kernel-core/src/lib.rs` 13 semuanya di baris lama, `asl-memory/src/paging.rs` 11, `asl-cockpit-shell/src/lib.rs` 10 semuanya di baris lama) | lihat catatan dasar di bawah |
| rustfmt seluruh repositori | `cargo fmt --all -- --check` disimpan ke `Analysis/reports/E8-FMT-SELURUH-WORKSPACE-CHECK-2026-10-21.txt` (1.914 baris keluaran, 140 blok `Diff in`) | **140 blok deviasi warisan**: `asl-kernel-core/src/lib.rs` 50, `asl-hal/src/vtd.rs` 22, `asl-kernel-core/src/vtd_boot.rs` 12, `asl-storage/src/ramdisk.rs` 10, `asl-memory/src/paging.rs` 10, `asl-cockpit-shell/src/lib.rs` 9, sisanya tersebar | angka pembuktian, bukan target |

**Catatan dasar pengukuran (dibaca bersama, penting untuk akuntabilitas).**
Dua gerbang misi tidak dapat dijalankan secara harfiah pada cabang ini:
`cargo fmt --all -- --check` melaporkan 140 blok deviasi dan
`cargo clippy -D warnings` merah karena diagnostik yang **seluruhnya sudah ada
sebelum pekerjaan E8** (versi rustfmt/clippy 1.99 lebih ketat
daripada versi yang dipakai saat berkas-berkas itu ditulis; CI tidak
menjalankan fmt maupun clippy). Menyapu semuanya berarti mengubah puluhan
berkas di luar cakupan, yang dilarang aturan misi. Karena itu gerbang
dijalankan sebagai **"nol deviasi/diagnostik BARU"**, dengan alat ukur yang
menyebutkan Irisan (baris saya × baris yang ingin diubah perkakas), bukan
klaim angka nol mutlak. Angka warisan dicatat apa adanya di sini.
Dua hal yang diperlukan agar clippy bisa menyelesaikan lintasan ke crate saya
juga dicatat: `not_unsafe_ptr_arg_deref` (deny bawaan, ada di
`asl-kernel-core-secondary/src/lib.rs:156`) harus di-`-A` di tingkat baris
perintah supaya dependensi mau dikompilasi; tanpa itu crate yang saya ubah
tidak pernah ikut ter-lint.

---

## 6. Kejadian akuntabilitas selama pekerjaan ini

`rustfmt` yang dijalankan saya pada satu salinan berkas di dalam direktori
`src` **merekursi deklarasi `mod`** dan ikut memformat
`asl-kernel-core/src/vtd_boot.rs` (69 baris bergeser, murni tata letak, tanpa
perubahan makna). Berkas itu berada di luar cakupan perubahan saya.
Tindakan: diff disimpan sebagai bukti di
`Analysis/reports/E8-KECELAKAAN-RUSTFMT-REKURSI-VTD-BOOT.patch` (148 baris;
53 baris tambahan, 16 penghapusan = 69 baris bergeser), lalu `git restore` atas
berkas tersebut; `git status` mengonfirmasi hanya tiga berkas yang berubah.
Peristiwa ini dicatat sebagai entri tersendiri di
`docs/ROADMAP/ACCOUNTABILITY_FAULT_LOG.json` (lapor diri sendiri, zero-amend:
riwayat tidak ditulis ulang). Sejak kejadian itu `rustfmt` hanya dijalankan
dengan `--check` (hanya-baca), dan karena `--check` pun melaporkan submodul
hasil rekursi, alat irisan menyaring header `Diff in <jalur>:<baris>:` supaya
hunk milik berkas lain tidak ikut dihitung.

---

## 7. Yang belum selesai (jangan dibaca sebagai selesai)

1. **LANGKAH 5**: gerbang penuh pada artefak — build ISO di dua lingkungan,
   ukur ulang lantai memori, boot QEMU 256M berjendela sampai prompt tampil.
2. **LANGKAH 6**: jendela G9-LITE disaksikan pemilik. Pemilik mengulang persis
   pemicu (`test_task` tunggal, lalu sesi adversarial), tangkapan layar +
   berkas serial disimpan, dan pemilik menyatakan lulus. **VERIFIED-ON-HARDWARE
   tetap 0** sampai jendela ini disaksikan.
3. **Teks panic dari pemilik**: tanpa itu E8 berstatus
   BELUM-TERREPRODUKSI-SEBAGAI-PANIC dan tidak boleh ditutup.
4. Verifikasi perilaku perbaikan di mesin (satu per satu, tiap boot): `test_task`
   dapat dihentikan tombol, `test_apic` mengembalikan IF, panic handler
   mencetak konteks (suntikan lewat diagnostik `-DemoCommand`/fault demo).
5. Extraksi hosted CI atas commit pekerjaan ini dan penutupan utang ekstraksi
   G8 yang tertunda (empat peristiwa CI atas dua commit terdahulu).
