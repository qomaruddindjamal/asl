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
**Gerbang penuh LANGKAH 5 sudah diukur** (Bagian 5.1): 202 unit test lulus,
build nol warning pada dua lingkungan dengan ELF identik, ISO baru
`0391c411...`, lantai memori diukur ulang (ratchet 3392K lulus, janji 3M tetap
MERAH), prompt boot tampil pada tiga sesi VM, dan perbaikan A4/A5/A3 terbukti
di mesin. Yang belum adalah jendela yang disaksikan pemilik (LANGKAH 6) dan
teks panic dari pemilik.

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

### 5.1 Gerbang artefak + verifikasi di mesin (LANGKAH 5, di atas ISO baru)

Perintah yang dijalankan (sisi host build, kemudian QEMU 256M berjendela):

```
# lingkungan 1: build di direktori target bawaan
cargo build --package asl-kernel-bin --target x86_64-unknown-none --release
# lingkungan 2: build DARI NOL di direktori target terpisah (direktori itu
# dihapus lebih dulu; seluruh rentang crate ikut terkompilasi ulang)
CARGO_TARGET_DIR=<direktori-scratch> cargo build --package asl-kernel-bin --target x86_64-unknown-none --release
pwsh -File scripts/build_kernel.ps1 -Action iso                              # ISO + penanda waktu + GPT embedding
pwsh -File scripts/test_ram_floor_3mb.ps1 -TimeoutSec 150                    # -m 3M lalu -m 3392K
pwsh -File scripts/e8_adversarial_run.ps1 -StopKeyDelaySeconds 25
pwsh -File scripts/e8_adversarial_run.ps1 -IncludeDiagnostics
pwsh -File scripts/e8_adversarial_run.ps1 -DemoCommand trap_de
```

| Gerbang LANGKAH 5 | Hasil terukur | Kelas bukti | Artefak |
|---|---|---|---|
| Build kernel pada **dua lingkungan** | lingkungan 1 memakai target bawaan (4 baris `Compiling`), lingkungan 2 dari nol di direktori target terpisah (13 baris `Compiling`); dua-duanya selesai dengan **nol baris warning dan nol galat**; ELF 325.120 byte dengan SHA256 **identik** `3ff9f2a659502d8f2b178bbb6473a217340f0fa5c5ef9b616477e55a4f010363` (diverifikasi ulang terhadap dua berkas ELF sesudah sesi) | VERIFIED-ON-HOST | `e8_l5_build-lingkungan-{1,2}_2026-10-21.txt` |
| ISO | 4.200.448 byte, SHA256 `0391C41149EC66A565F03E9CADA637EF7281179C5F6C820E6044DC22300B74DE`. Berubah dari `aa778060...` karena delta `.rs` E8 — konsisten dengan sifat sidik jari **per-tree** | VERIFIED-ON-HOST | `e8_l5_iso_2026-10-21.txt` |
| Boot QEMU 256M sampai prompt tampil | `M6=True prompt=True crash=TIDAK` pada tiga sesi | VERIFIED-ON-VM | tiga berkas `e8_adversarial_summary_*` di bawah |
| Ukur ulang lantai memori | **kode keluar 2**: `-m 3M` tidak sampai prompt (serial **0 byte** — berkas log kosong itu adalah buktinya), `-m 3392K` lulus penuh (serial 2.498 byte). Janji Bagian 5 `-m 3M` **tetap MERAH**; deviasi entri ke-23 tidak diubah menjadi hijau | VERIFIED-ON-VM | `e8_l5_lantai-memori_2026-10-21.txt`, `e8_l5_lantai-{3M,3392K}-serial_2026-10-21.log` |
| A4 — satu ketikan menghentikan `test_task` | boot segar + `-StopKeyDelaySeconds 25`: `prompt kembali=True`, **elapsed_ms=25528**, `pesan_operator=True`, crash=TIDAK, kode keluar alat 0. Pra-perbaikan: **178.779 ms** tanpa pemulihan kendali (Bagian 2, Pass D) | VERIFIED-ON-VM | `e8_adversarial_*_2026-10-21_22-49-02.*`, `e8_l5_tombol-penghenti-konsol_2026-10-21.txt` |
| Pass B penuh sesudah perbaikan | 59 kasus (41 utama + 2 pembersih + 16 diagnostik) **seluruhnya [OK]**, kode keluar alat 0; pembanding pra-perbaikan: Pass B yang sama keluar **2** dengan D13 menggantung. D13 `test_task` 245054 ms, D14 pengulangan 473 ms, D15 `test_apic` sesudah `test_task` 605 ms | VERIFIED-ON-VM | `e8_adversarial_*_2026-10-21_22-50-35.*`, `e8_l5_sesi-diagnostik-konsol_2026-10-21.txt` |
| A5 — bendera interupsi dipulihkan | serial log baris 246 dan 377 memuat `[TEST 7] Bendera interupsi CPU dipulihkan (cli) -> PASS` | VERIFIED-ON-VM | berkas serial 22-50-35 |
| A3 — laporan kegagalan berisi konteks | `-DemoCommand trap_de` (crash yang **dipanggil sendiri**, jadi kode keluar 2 adalah hasil yang diharapkan): blok `[ASL EXCEPTION]` mencetak vektor, kode galat, RIP/CS, RSP/SS, RFLAGS, RBP lalu `Controlled Halt Active` | VERIFIED-ON-VM | `e8_adversarial_*_2026-10-21_23-04-58.*`, `e8_l5_trap-de-konsol_2026-10-21.txt` |

**Batas yang jujur untuk A6 (panic handler Rust).** Setelah A1-A6 tidak ada
satu pun masukan yang dapat diketik pada lintasan perintah kanonik yang
mencapai `panic_handler` — itulah isi perbaikan itu sendiri — sehingga sesi
22-49-02 dan 22-50-35 memang tidak menghasilkan penanda crash. Karena itu
jalur panic **dinyatakan teruji di tingkat statis dan unit test sisi host**
(202 lulus, enam di antaranya penjaga `panic_guard_*`), bukan VERIFIED-ON-VM.
Yang terbukti di mesin untuk tuntutan "cetak konteks" adalah jalur jebakan CPU
(baris `trap_de` di atas). Perbedaan dua jalur ini tidak boleh dibaca sebagai
satu kelulusan.

**Konsekuensi alat.** `scripts/e8_adversarial_run.ps1` kini membawa nilai
bawaan `-ExpectedIsoSha` = `0391c411...`; karena sidik jari ISO per-tree,
setiap bangun ulang ISO menuntut nilai itu dioper lewat baris perintah (atau
alat dikalibrasi ulang pada commit berikutnya). Aturan gagal-tutup tidak
dilonggarkan: sidik jari yang tidak cocok membuat sesi keluar 1.

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

Kejadian kedua (LANGKAH 5, lapor diri sendiri). Pesan commit `b9acd5b`
(`[V1.0.2-R1-F8-UKUR] ... DI ATAS ISO 0391C411`) menyebut sesi
`2026-10-21_22-50-35` sebagai **"Pass A penuh"** padahal yang dijalankan
adalah **Pass B** — 59 kasus termasuk 16 diagnostik (`-IncludeDiagnostics`).
Angka dan label di dalam artefak tidak salah; yang salah adalah nama jalur di
kalimat saya, dan nama itu penting karena Bagian 2 mendefinisikan Pass A (43
payload, keluar 0) dan Pass B (59 kasus, dulu keluar 2) sebagai dua keadaan
yang berbeda. Koreksi tidak dilakukan dengan menulis ulang pesan commit
(aturan G6/G8: nol-amend), melainkan lewat entri tersendiri di
`docs/ROADMAP/ACCOUNTABILITY_FAULT_LOG.json` (ke-35) plus catatan ini.
Sejak itu setiap klaim kelulusan menyebut bendera alat yang benar, dan
baris `kasus ...` pada kepala ringkasan dipakai sebagai dasar penamaan.

---

## 7. Yang belum selesai (jangan dibaca sebagai selesai)

Sudah selesai pada langkah ini: **LANGKAH 5** tertutup seluruhnya (Bagian 5
dan 5.1) — unit test 202, irisan fmt/clippy nol BARU, build nol warning pada
dua lingkungan dengan ELF identik, ISO baru, ukur ulang lantai memori, boot
QEMU 256M sampai prompt, verifikasi A3/A4/A5 di mesin, dan pernyataan batas
bukti untuk A6.

Yang tersisa:

1. **LANGKAH 6**: jendela G9-LITE disaksikan pemilik. Pemilik mengulang persis
   pemicu (`test_task` tunggal pada boot segar, lalu sesi adversarial penuh),
   tangkapan layar + berkas serial disimpan, dan pemilik menyatakan lulus.
   **VERIFIED-ON-HARDWARE tetap 0** sampai jendela ini disaksikan. Alat
   `scripts/g9_final_p8_run.ps1` sengaja tidak diubah pada rantai R1.
2. **Teks panic dari pemilik**: tanpa itu E8 berstatus
   BELUM-TERREPRODUKSI-SEBAGAI-PANIC dan **tidak boleh ditutup** (aturan
   "DILARANG menutup E8 tanpa akar penyebab terverifikasi"; yang terverifikasi
   sekarang adalah akar penyebab pembekuan `test_task`, bukan panic yang
   dilaporkan).
3. **Utang ekstrak hosted CI (G8)**: empat peristiwa CI atas dua commit E8
   terdahulu sudah tertutup pada commit ekstrak sebelumnya; yang masih terbuka
   adalah peristiwa CI atas `97d1834`, `d6c5b12`, dan tiga commit LANGKAH 5
   (`093a7d6`, `b9acd5b`, `ca582fa`). Ekstrak sebuah commit hanya bisa masuk
   pada commit BERIKUTNYA, jadi penutupan ini adalah pekerjaan pertama setelah
   push. Tree pada `97d1834` dan seterusnya sudah memuat perbaikan `.rs` E8
   (berbeda dari tip-tip tanpa-`.rs` yang diekstrak sebelumnya), jadi angka
   yang diharapkan di runner CI adalah **202 lulus / 0 gagal**, bukan 196.
4. **Laporan E8 final (LANGKAH 7)**: dokumen ini berlabel STATUS PEKERJAAN;
   laporan penutupan ditulis setelah jendela pemilik dan sesudah teks panic
   pemilik ada atau dinyatakan tidak tersedia.
5. Setelah E8 ditutup: BAGIAN 2 (F1-F7). Registry kanonik pada ISO yang di-boot
   masih 11 perintah, jadi kasus C15 (`up`) dan C26/C27 (`rename`) pada Pass
   penuh masih menguji jalur "perintah tak dikenal"; jalur aslinya diuji lagi
   setelah F2 dan F6, dan jendela E8 diulang.
