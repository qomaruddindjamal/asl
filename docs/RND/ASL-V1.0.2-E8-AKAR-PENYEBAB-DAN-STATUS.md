# ASL v1.0.2 — E8: AKAR PENYEBAB DAN STATUS PEKERJAAN

> **LABEL DOKUMEN: STATUS PEKERJAAN — SUDAH DITUTUP, BUKAN LAGI DOKUMEN HIDUP.**
> E8 **ditutup oleh pemilik pada 2026-10-22**: jendela G9-LITE disaksikan dan
> dinyatakan LULUS, dan teks panic **dinyatakan tidak tersedia**. Laporan
> penutupannya ada di `docs/RND/ASL-V1.0.2-E8-LAPORAN-PENUTUPAN.md`; dokumen ini
> tetap dibiarkan apa adanya sebagai rekaman kerja (koreksi tidak dilakukan
> dengan menghapus riwayat, hanya dengan catatan baru). Angka di dokumen ini
> berasal dari artefak yang tercantum di Bagian 6, bukan dari tebakan.

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
yaitu pembekuan shell akibat `test_task` — **178.779 ms = 178,8 detik** pada ISO
pra-perbaikan, dan **64,0 sampai 310,7 detik** pada ISO final tergantung kondisi
(Bagian 5.3). Di luar itu,
audit statis menemukan **empat lokasi panic yang dapat dipicu satu ketikan**
di rangkaian Gate 3D dan **satu pemulihan keadaan yang bolong** di
`test_apic`. Semuanya sudah diubah menjadi pesan galat + prompt kembali
(LANGKAH 3), dijaga enam test regresi baru (LANGKAH 4), dan panic handler
kernel kini mencetak konteks penuh (pesan + lokasi sumber + alamat + rekaman
jebakan terakhir) supaya panic berikutnya tidak datang tanpa bukti.
**Gerbang penuh LANGKAH 5 sudah diukur** (Bagian 5.1 dan 5.2): 202 unit test
lulus, build nol warning pada dua lingkungan, ISO `0391c411...` lalu **ISO baru
`18f40d2d...`** sesudah higiene compiler pada penanganan panic, lantai memori
diukur ulang pada kedua ISO (ratchet 3392K lulus, janji 3M tetap **MERAH**),
prompt boot tampil pada tujuh sesi adversarial VM berjendela (empat di antaranya
di atas ISO final), dan perbaikan A4/A5/A3 terbukti
di mesin di atas ISO final. Yang belum pada saat dokumen ini ditulis: jendela yang disaksikan pemilik
(LANGKAH 6) dan teks panic dari pemilik. **Keduanya sudah diselesaikan pada
2026-10-22** — jendela disaksikan dan dinyatakan LULUS, teks panic dinyatakan
TIDAK TERSEDIA — sehingga E8 berstatus TERTUTUP dengan label jujur
`BELUM-TERREPRODUKSI-SEBAGAI-PANIC`: tidak ada masukan yang dapat diketik yang
menghasilkan `[ASL-CORE-PANIC]`, dan tidak ada teks panic untuk dicocokkan dengan
jalur di Bagian 3. Rincian putusan dan batas-bukti ada di
`docs/RND/ASL-V1.0.2-E8-LAPORAN-PENUTUPAN.md`.

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

Catatan satuan yang berlaku untuk SELURUH dokumen ini (dibuat eksplisit karena
kolom waktu berkas TSV bernama `detik_mm` padahal isinya MILIDETIK): angka
berpemis titik seperti `178.779 ms` adalah 178.779 milidetik = **178,8 detik**,
bukan sepersekian detik. Bagian 5.3 menulis kedua bentuk berdampingan.

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
pwsh -File scripts/e8_adversarial_run.ps1 -IncludeDiagnostics -CaseTimeoutSeconds 600 -BootTimeoutSeconds 240
pwsh -File scripts/e8_adversarial_run.ps1 -DemoCommand trap_de
```

Dua bendera pada baris Pass B bukan hiasan dan bukan pilihan: kasus D13
(diagnostik `test_task`) memerlukan **245.054 ms** pada sesi 22-50-35 dan
**272.600 ms** pada sesi 23-22-59, sedangkan bawaan alat adalah 45 s per
kasus. Menjalankan `-IncludeDiagnostics` tanpa `-CaseTimeoutSeconds 300`
menghasilkan kode keluar 2 dengan D13 berstatus [GANTUNG] — bukti nyata hal
itu tersimpan sebagai sesi 23-19-23 dan dilaporkan sebagai entri
`ACCOUNTABILITY_FAULT_LOG` ke-36. Sejak commit `1e2035f` tiap artefak
mencetak batas yang sedang berlaku pada baris kepala `batas`, jadi jalur
invokasi dapat dibaca dari bukti, bukan dari ingatan.

**Kenaikan batas 300 s -> 600 s (2026-10-22, jendela LANGKAH 6).** Angka 300 di
atas diturunkan dari pengukuran D13 di TENGAH sesi penuh; pengukuran kasus yang
sama pada BOOT SEGAR - kondisi yang diminta jendela G9-LITE - ternyata butuh
**310.681 ms = 310,7 s**, sehingga sesi 2026-10-21_23-58-19 keluar 2 dengan V01
[GANTUNG] pada 300.033 ms padahal serial masih mencetak 75.004/100.000 tukar.
Resep pada blok di atas karena itu memakai 600 s; tabel kelima sesi dan
perhitungan kisarannya ada di Bagian 5.3, dan kesalahannya dicatat sebagai entri
ke-37 result CLOSED. Dua sebutan `-CaseTimeoutSeconds 300` yang MASIH terbaca di
dokumen ini (satu pada paragraf di atas, satu pada Bagian 6) sengaja TIDAK
ditulis ulang: keduanya melaporkan batas yang benar-benar berlaku pada sesi yang
dilaporkan, dan mengubahnya menjadi angka hari ini akan membuat bukti invokasi
tidak dapat dipercaya lagi.

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

**Konsekuensi alat.** `scripts/e8_adversarial_run.ps1` membawa nilai bawaan
`-ExpectedIsoSha` yang adalah penanda **per-tree**: `0391c411...` pada sesi
22-49-02 sampai 23-04-58, lalu dikalibrasi ulang menjadi `18f40d2d...` pada
commit `1e2035f` karena delta `.rs` higiene mengubah ISO. Setiap bangun ulang
ISO menuntut nilai itu dioper lewat baris perintah atau dikalibrasi ulang pada
commit berikutnya. Aturan gagal-tutup tidak dilonggarkan: sidik jari yang tidak
cocok membuat sesi keluar 1 sebelum mesin dinyalakan.

### 5.2 Ukur ulang di atas ISO 18F40D2D (LANGKAH 5b, sesudah higiene compiler)

Commit `53e226f` menghapus tiga diagnostik yang lahir dari kode saya sendiri
(impor `core::panic::PanicInfo` dihapus dan ditulis penuh pada tanda tangan
`fn panic`; `struct PanicContextSink` beserta `impl core::fmt::Write`-nya diberi
`#[cfg(not(test))]`). Perubahan itu mengubah **sidik jari artefak**, jadi seluruh
angkanya diukur ulang dan seluruh sesi mesin diulang pada ISO baru. Tidak ada
angka 5.1 yang dipindahkan ke 5.2 tanpa pengukuran ulang.

| Gerbang LANGKAH 5b | Hasil terukur pada tree `53e226f` | Kelas bukti | Artefak |
|---|---|---|---|
| Unit test sisi host build | **202 lulus, 0 gagal**; baris `warning` profil test **19**, **nol** dari `asl-kernel-core` (sebelum higiene: 22, dua di antaranya dari kode saya) | VERIFIED-ON-HOST | `e8_l5b_uji-host_2026-10-21.txt` |
| Irisan tata letak dan lint | fmt: TOTAL deviasi **BARU 0** (baris saya 3 dari 107 yang ingin diubah di `lib.rs`; 0 dari 28 dan 0 dari 19 pada dua berkas lain). clippy: 137 diagnostik ber-baris workspace, TOTAL **BARU 0** (13 di `lib.rs`, semuanya baris lama) | VERIFIED-ON-HOST | `e8_l5b_irisan-{fmt,clippy}_2026-10-21.txt` |
| Build rilis | 0 `warning`, 0 `error`; ELF **325.120 byte** SHA256 `74405577744896d747b5278fe7ff9a0f8033e0d90ebb2aa5aa8d9b1bdbe287b3` | VERIFIED-ON-HOST | `e8_l5b_build-rilis_2026-10-21.txt` |
| ISO | **4.200.448 byte** SHA256 `18F40D2D4E18C3B46A88C13090AC1F6FCCE13B4D264A1D5B78B4D6E0A833FAD0` (berubah dari `0391c411...` tepat karena delta `.rs`) | VERIFIED-ON-HOST | `e8_l5b_iso_2026-10-21.txt` |
| Lantai memori | kode keluar **2**: `-m 3M` tidak sampai prompt, serial **0 byte**; `-m 3392K` lulus, serial 2.498 byte. Janji Bagian 5 `-m 3M` **tetap MERAH** | VERIFIED-ON-VM | `e8_l5b_lantai-memori_2026-10-21.txt`, `e8_l5b_lantai-{3M,3392K}-serial_2026-10-21.log` |
| A4 — satu ketikan menghentikan `test_task` | `-StopKeyDelaySeconds 25`: keluar 0, prompt kembali=True, **elapsed_ms=25499**, pesan_operator=True, crash=TIDAK | VERIFIED-ON-VM | `e8_adversarial_*_2026-10-21_23-18-46.*` |
| Pass B penuh | 59 dari 59 kasus **[OK]**, nol penanda crash, keluar 0; D13 `test_task` **272.600 ms** (pembanding 22-50-35: 245.054 ms), D14 pengulangan 473 ms, D15 `test_apic` sesudah `test_task` 882 ms (pembanding 605 ms); A5 terukur lagi di serial baris 246 dan 377 | VERIFIED-ON-VM | `e8_adversarial_*_2026-10-21_23-22-59.*`, `e8_l5b_sesi-diagnostik-konsol_2026-10-21.txt` |
| A3 — laporan kegagalan berisi konteks | `-DemoCommand trap_de`, keluar 2 = crash yang dipanggil: blok `[ASL EXCEPTION]` baris 58-65 memuat VECTOR 0x0000 (#DE), ERROR 0, RIP `0xFFFFFFFF8011DB82`/CS 0x0028, RSP `0xFFFFFFFF801AAFC0`/SS 0x0030, RFLAGS 0x0000000000010046/RBP 0, lalu Controlled Halt Active; RIP identik dengan sesi 23-04-58 di ISO lama | VERIFIED-ON-VM | `e8_adversarial_*_2026-10-21_23-28-45.*`, `e8_l5b_trap-de-konsol_2026-10-21.txt` |
| CI terkelola atas tree ini | dua peristiwa atas `53e226f` (push 39634616449, pull_request 39634624636) sama-sama success: 202 lulus/0 gagal pada 30 suite, **19** baris `warning` (turun dari 22 pada run `97d1834`), 13 `Compiling` 0 warning/0 galat, ELF sisi runner 325.184 byte | VERIFIED-ON-CI-API | `ci_hosted_39634616449_2026-10-21.log`, `ci_hosted_39634624636_2026-10-21.log` |

Dua hal yang sengaja tidak disamakan. Pertama, ELF sisi runner CI (325.184
byte) berbeda dari ELF sisi host build (325.120 byte); itu perbedaan
lingkungan kompilasi dan dicatat, bukan diratakan. Kedua, tabel llvm-cov pada
run `53e226f` menghasilkan hash konvensi gabungan-LF
`1bebd7ef13583fb639714662ed572e162126dcc96fa19af8e0beccb3f9893377` — **identik**
dengan run `97d1834`. Keidentikan itu justru yang diharapkan: higiene ini
menghapus diagnostik, bukan baris yang dieksekusi, sehingga cakupan tidak
bergeser.

Batas yang tetap berlaku sesudah 5.2: jalur panic Rust masih **tidak** dapat
diketik sampai `panic_handler` pada lintasan perintah kanonik (lihat "Batas
yang jujur untuk A6" di atas), sehingga bukti jalur itu tetap statis + unit
host, bukan VERIFIED-ON-VM. E8 tetap BELUM DITUTUP selama teks panic dari
pemilik belum diterima.

---

### 5.3 Jendela LANGKAH 6: pemicu diulang pada boot segar dan hasilnya mencabut kata "nyaman"

Sesi pertama jendela ini dijalankan persis seperti yang diminta misi: satu token
`test_task` pada boot segar `-m 256M`, jendela terlihat, serial byte-per-byte.
Hasilnya tidak cocok dengan angka yang saya publikasikan lebih dulu.

| Sesi (cap berkas) | ISO | Cara | Batas kasus berlaku | Hasil | Waktu terukur |
|---|---|---|---|---|---|
| 2026-10-21_20-09-52 | `aa778060` (pra-perbaikan E8) | boot segar, satu token | tidak tercetak di kepala (format pra-`1e2035f`); waktu 178,8 s tercapai jadi batasnya lebih tinggi | OK | 178.779 ms = 178,8 s |
| 2026-10-21_23-22-59 | `18f40d2d` | tengah sesi penuh (D13) | tidak tercetak di kepala (format pra-`1e2035f`); resep yang dilaporkan saat itu 300 s | OK | 272.600 ms = 272,6 s |
| 2026-10-21_23-58-19 | `18f40d2d` | boot segar, satu token | 300 s (tercetak di baris kepala) | **GANTUNG** | 300.033 ms = 300,0 s; progres terakhir 75.004/100.000 tukar |
| 2026-10-22_00-06-29 | `18f40d2d` | boot segar, satu token | 600 s (tercetak) | OK | 310.681 ms = 310,7 s; TEST 1..15 PASS, prompt kembali |
| 2026-10-22_00-12-18 | `18f40d2d` | sesi penuh 59 kasus (D13) | 600 s (tercetak) | OK | 64.031 ms = 64,0 s; 59 dari 59 kasus OK, keluar alat 0 |

Kolom "batas kasus berlaku" sengaja dibedakan: sejak commit `1e2035f` baris
kepala `batas` pada berkas ringkasan mencetak angka yang sedang berlaku, jadi
tiga sesi terbawah terbaca langsung dari artefak. Dua sesi teratas memakai format
kepala yang belum mencetaknya, sehingga yang boleh dilaporkan hanyalah resep yang
berlaku pada saat itu plus fakta bahwa waktu terukur berhasil dilampaui.

Tiga fakta yang harus dibaca bersama, bukan dipilih satu:

1. **Tidak ada panic.** Kelima sesi tidak memunculkan `ASL-CORE-PANIC`,
   `ASL EXCEPTION`, maupun `FATAL`, dan proses QEMU tidak keluar sendiri. Yang
   ter-reproduksi adalah **waktu jalan yang panjang**, bukan crash.
2. **Bukan deadlock.** Pada 300 s pertama mesin sudah mencetak 75.004 dari
   100.000 tukar dan terus mencetak; dengan batas 600 s rangkaian selesai,
   TEST 1 sampai TEST 15 PASS, dan prompt kembali.
3. **Durasi tidak stabil dan bergantung kondisi** pada ISO yang sama: 310,7 s
   saat boot segar, 272,6 s dan 64,0 s di tengah sesi penuh, dan pemanggilan
   KEDUA `test_task` dalam sesi yang sama kembali dalam 473-507 ms (D14).
   Artinya setiap batas tetap untuk kasus ini adalah tebakan yang harus
   dinyatakan sebagai tebakan.

Label alat harus dibaca persis, tidak boleh dipilih-pilih. Sesi 23-58-19 keluar
dengan kode 2 dan baris `HASIL: REPRODUKSI POSITIF - 1 kasus`, disusul
`E8-STATUS: MODE VERIFIKASI token 'test_task' - crash terlihat`. Berdasarkan
baris 27 `scripts/e8_adversarial_run.ps1`, kode 2 berarti "penanda crash ATAU
shell tidak kembali ke prompt" — di sesi ini yang menyala adalah cabang kedua,
karena nol baris `ASL-CORE-PANIC` / `ASL EXCEPTION` / `FATAL` terukur di seluruh
berkas (satu-satunya kemunculan ketiga frasa itu adalah baris kepala
`penanda crash` yang mencantumkan definisinya sendiri). Frasa "crash terlihat"
pada baris 563 adalah teks tetap yang tidak dikondisikan pada penanda crash,
sehingga membaca ringkasan tanpa membaca artefaknya dapat menyimpulkan panic yang
tidak pernah terjadi. Alat tidak saya ubah pada unit ini supaya jendela pemilik
berjalan di alat yang sama dengan bukti yang sudah ada; ketidaktepatan label ini
dilaporkan ke pemilik bersama entri ke-38.

Dua hal yang sengaja tidak disamakan. Perbaikan A4 (`suite_masukan_masuk()` di
`asl-kernel/asl-kernel-core/src/lib.rs:2684`, dipakai loop di `:2918`) membaca
dua port pada SETIAP iterasi supaya satu ketikan menghentikan rangkaian; harga
fitur itu adalah waktu jalan rangkaian yang naik dari 178,8 s (pra-perbaikan) ke
310,7 s (boot segar sesudah perbaikan). Kedua angka berasal dari ISO dan beban
latar yang berbeda, jadi yang dilaporkan adalah **kisaran**, bukan satu angka
sebab-akibat.

**Kenapa operator melihat "shell dibekukan".** Seluruh kemajuan rangkaian ditulis
dengan `early_serial_write_str` (`:2692` dan seterusnya) sehingga hanya muncul di
berkas serial. Pada jendela yang dilihat manusia, layar tetap berisi banner boot
dan prompt kosong selama 64 sampai 311 detik. Dua tangkapan layar disimpan sebagai bukti:
`Analysis/reports/e8_l6_jendela-sesi-A-token_2026-10-21.png` (1040x832, gema
`test_task` masih terlihat, diambil sebelum Enter) dan
`Analysis/reports/e8_l6_jendela-sesi-A2-hidup_2026-10-22.png` (1040x832, diambil
saat rangkaian sudah berjalan: prompt kosong, nol baris keluaran). Dari kursi
operator kedua keadaan itu tidak dapat dibedakan dari hang.

Tindakan yang diambil pada unit ini: batas kasus pada Bagian 5.1 dinaikkan ke
600 s dengan alasan terukur, konversi detik ditulis berdampingan dengan setiap
angka milidetik, dan temuan produk didaftarkan sebagai ACCOUNTABILITY_FAULT_LOG
entri ke-38 result OPEN. Yang TIDAK saya lakukan: mengubah kode kernel untuk
mempercepat rangkaian atau memindahkan keluarannya ke framebuffer (di luar
cakupan E8 yang diizinkan misi R1; keputusan penerimaan ada pada pemilik), dan
mengganti nama kolom `detik_mm` pada alat (akan membuat artefak lama dan baru
berbeda skema; dicatat sebagai pekerjaan higiene, entri ke-37).

Status jendela: **DISAHKAN PEMILIK pada 2026-10-22** — pemilik menyaksikan
pengulangan pemicu dan menyatakan **LULUS**, teks panic dinyatakan **tidak
tersedia**, dan temuan ke-38 diterima sebagai temuan produk yang penanganannya
digabung ke F5. Konter VERIFIED-ON-HARDWARE **tetap 0**: yang disaksikan
berlangsung di emulator pada mesin pengembangan (kelasnya
VERIFIED-ON-VM-VISIBLE), sehingga vonis LULUS ini menutup tuntutan jendela G9-LITE
tanpa mengubah klaim perangkat keras. Rincian putusan: Bagian 1 laporan
penutupan E8.

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

Kejadian ketiga (LANGKAH 5b, lapor diri sendiri). Setelah ISO berganti menjadi
`18f40d2d...`, saya menjalankan ulang Pass B dengan **baris perintah yang tertulis
di Bagian 5.1 dokumen ini** — yaitu `... -IncludeDiagnostics` tanpa bendera batas
waktu — dan sesi itu keluar 2 dengan D13 berstatus [GANTUNG] setelah 45.030 ms
(artefak `2026-10-21_23-19-23`). Tidak ada satu pun baris dokumen yang menyebut
bahwa sesi lulus 22-50-35 memakai `-CaseTimeoutSeconds 300 -BootTimeoutSeconds 240`;
baris `kasus` pada kepala ringkasan lama juga tidak mencetak batas yang sedang
berlaku. Jadi akarnya dua lapis: (a) saya merekonstruksi invokasi dari ingatan
alih-alih dari artefak sesi yang lulus, dan (b) dokumen 5.1 sendiri tidak lengkap
sebagai resep. Yang pertama saya akui sebagai entri ke-36
`ACCOUNTABILITY_FAULT_LOG`; yang kedua diperbaiki di tempat (blok perintah 5.1
kini membawa bendera wajib dan alasan angkanya dijelaskan di bawahnya), dan
pencegahan ketiga dibangun ke dalam alat: commit `1e2035f` mencetak baris
`batas` di kepala setiap artefak sehingga invokasi dapat dibaca dari bukti.
Artefak sesi keliru 23-19-23 **tidak dibuang dan tidak diberi label lain**; ia
menjadi bukti bahwa bawaan 45 s tidak memadai untuk `test_task`, dan pembanding
langsung bagi sesi 23-22-59 yang menjalankan 59 kasus sampai selesai.
Sesudah kejadian ini, setiap kali alat dijalankan dengan kasus berat, baris
`batas` pada artefak diperiksa lebih dahulu sebelum keluarannya dibaca.

---

## 7. Yang belum selesai (jangan dibaca sebagai selesai)

Sudah selesai pada langkah ini: **LANGKAH 5** tertutup seluruhnya (Bagian 5,
5.1 dan 5.2) — unit test 202 lulus, irisan fmt/clippy nol BARU, build rilis nol
warning dan nol galat pada dua lingkungan, dua ISO (0391C411 lalu 18F40D2D sebagai
artefak final rantai ini), lantai memori diukur ulang pada keduanya, boot QEMU
256M sampai prompt tampil pada tujuh sesi adversarial berjendela, verifikasi
A3/A4/A5 di mesin di atas ISO final, dan pernyataan batas bukti untuk A6.

Yang tersisa:

1. **LANGKAH 6 (SELESAI — disahkan pemilik 2026-10-22)**: jendela G9-LITE. Sesi pemicu dan
   sesi penuh sudah dijalankan berjendela di atas ISO `18f40d2d` dan hasilnya
   tercatat di Bagian 5.3 - termasuk deviasi waktu jalan yang memaksa batas
   kasus dinaikkan ke 600 s. Yang belum ada adalah **pengesahan pemilik**, jadi
   langkah ini TIDAK tertutup dan **VERIFIED-ON-HARDWARE tetap 0**. Yang
   keempat permintaan itu sudah dijawab pemilik: jendela **disaksikan dan
   dinyatakan LULUS**, teks panic **dinyatakan TIDAK TERSEDIA**, dan entri ke-38
   **diterima sebagai temuan produk dengan penanganan digabung ke F5**. Alat
   `scripts/g9_final_p8_run.ps1` sengaja tidak diubah pada rantai R1.
2. **Teks panic dari pemilik**: **dinyatakan TIDAK TERSEDIA oleh pemilik pada
   2026-10-22**, dan dengan pernyataan itu E8 ditutup atas nama pemilik — label
   sisanya tetap BELUM-TERREPRODUKSI-SEBAGAI-PANIC (aturan
   "DILARANG menutup E8 tanpa akar penyebab terverifikasi"; yang terverifikasi
   sekarang adalah akar penyebab pembekuan `test_task`, bukan panic yang
   dilaporkan).
3. **Utang ekstrak hosted CI (G8)** — daftarnya dihitung ulang dari `gh api`
   dan `git ls-tree -r HEAD Analysis/reports`, bukan dari ingatan. Pada HEAD
   `c94f8b0` (commit-unit EKSTRAK pasca-F1, unit yang tidak menyentuh `.rs`)
   terukur TIGA PULUH run = LIMABELAS pasangan peristiwa atas LIMABELAS
   commit yang punya run, seluruhnya `conclusion=success`; SEPULUH
   commit cabang tidak pernah punya run sendiri (rincian di bawah). Audit
   per-commit tersimpan sebagai bukti di
   `Analysis/reports/e8_l9_audit-utang-ekstrak_2026-10-22.txt` (pengganti
   `e8_l8_audit-utang-ekstrak_2026-10-22.txt`,
   `e8_l7b_audit-utang-ekstrak_2026-10-22.txt`,
   `e8_l7_audit-utang-ekstrak_2026-10-22.txt` dan
   `e8_l6b_audit-utang-ekstrak_2026-10-22.txt`), dijalankan oleh
   `scripts/r1_debt_audit.py` pada HEAD `c94f8b0` sebelum unit F2 ini;
   EMPAT baris `BELUM` pada berkas itu adalah DUA pasangan dan bukan kegagalan:
   yang atas `5812c77` dan yang atas `c94f8b0`, kedua-duanya ikut commit-unit F2.
   *Sudah tertutup*: LIMABELAS pasangan peristiwa — atas `17570f6`, `a141640`,
   `97d1834`, `08ddd71`, `53e226f`, `1e2035f`, `8f56625`, `12adcf1`, `1f3e3c2`,
   commit jendela `4de1dfb`, commit unit 6b `b17a175`, commit unit 7a `4c3e0d2`,
   commit unit 7b `d8c002a`, commit unit F1 `5812c77` dan commit unit ekstrak
   `c94f8b0` — = TIGA PULUH
   berkas `Analysis/reports/ci_hosted_<id>_*.log`, masing-masing membawa blok
   CATATAN PER-RUN. Total berkas `ci_hosted_*` yang ter-commit di
   `Analysis/reports` menjadi 79. Empat ekstrak yang ikut unit F2 (dua pasangan
   terakhir) terukur identik di langkah [1/2] dan [2/2]: 205 lulus / 0 gagal pada
   30 suite, 19 baris `warning` profil test, 13 baris `Compiling` dengan 0
   `warning` / 0 `error`, ELF sisi runner CI 329.360 byte.
   *Masih terbuka*: pasangan peristiwa atas commit-unit F2 ini sendiri dan atas
   commit-unit berikutnya, yang baru tercipta sesudah push. Aturan G8 melarang
   ekstrak sebuah run masuk commit milik run itu, jadi penutupnya wajib ada di
   unit BERIKUTNYA; itu konsekuensi aturan, bukan kekeliruan. Run atas
   commit-unit F2 menjadi pasangan PERTAMA di cabang ini yang diukur pada tree
   dengan registry berposisi 12 perintah; angka lulus/gagal untuk pasangan itu
   HARUS dibaca dari log run setelah ia ada, bukan ditulis lebih dulu dari
   angka host.
   *Satu kejadian higiene bukti, dilaporkan apa adanya*: unduhan pertama log
   mentah run push `39644006778` mengembalikan 0 byte dan
   `scripts/ci_hosted_extract.py` tetap menulis ekstrak berisi `suite=0
   passed=0` tanpa gagal keras. Berkas itu saya hapus sebelum di-stage; unduhan
   kedua menghasilkan 169.256 byte dan ekstrak yang sah. Generator blok kini
   menolak berkas yang tidak memuat baris `TOTAL TERHITUNG DARI 30 SUITE: 202
   passed, 0 failed`. Pada unit ini kedua unduhan log langsung non-kosong dan
   terukur (`gh run view --log` = 206.163 byte untuk `39698870788` dan 210.312
   byte untuk `39698875695`), jadi guard itu tidak perlu menolah apa pun.
   Kelemahan alat dicatat di laporan E8.
   *Tidak dapat ditutup, dan ini fakta cara kerja CI bukan kelalaian*: SEPULUH
   commit di cabang ini tidak pernah punya run sendiri, karena satu peristiwa
   push hanya memicu run untuk tip saat itu — `007d3dd`, `54abf73`, `4e865c0`
   (tiga commit BAGIAN 0) serta `223e7f6`, `6bbd77d`, `939cc95`, `d6c5b12`,
   `093a7d6`, `b9acd5b`, `ca582fa`. Yang terverifikasi pada run atas tip adalah
   KUMPULAN perubahan sampai tip itu, bukan tree tiap commit secara individual,
   dan itu dinyatakan apa adanya. Acuan angka: run ter-commit terdekat atas tip
   pertama cabang (`17570f6`) mengukur 196 lulus / 0 gagal karena tree itu belum
   memuat `.rs` E8, sedangkan run atas `97d1834` dan seterusnya mengukur **202
   lulus / 0 gagal** karena perbaikan `.rs` E8 hidup sejak `6bbd77d` dan penjaga
   regresinya sejak `939cc95`.
4. **Laporan E8 final (LANGKAH 7)**: **sudah ditulis** di
   `docs/RND/ASL-V1.0.2-E8-LAPORAN-PENUTUPAN.md` setelah jendela disahkan dan
   setelah teks panic dinyatakan tidak tersedia. Dokumen ini tetap berlabel
   STATUS PEKERJAAN sebagai rekaman kerja.
5. Setelah E8 ditutup: BAGIAN 2 (F1-F7). Registry kanonik pada ISO F2 sudah
   **12 perintah** karena `up` masuk pada unit ini, jadi kasus C15 (`up`) tidak
   lagi menempuh jalur "perintah tak dikenal"; jalur aslinya terukur pada bagian
   [7] `Analysis/reports/f2_up_uji_2026-10-22.txt` (sesi QEMU khusus `up`,
   11 dari 11 deret prompt). Kasus C26/C27 (`rename`) MASIH menguji jalur
   "perintah tak dikenal" sampai F6 menambahkan `rename` dan registry menjadi
   13, dan pada F6 jendela E8 wajib diulang lagi — dengan catatan jendela itu
   baru bisa sah setelah pemecah prompt alat disesuaikan (butir 7 di bawah).
6. **Entri ke-38 (temuan produk; keputusan pemilik 2026-10-22: diterima,
   penanganan digabung ke F5)**: `test_task` pada ISO ini
   berjalan 64-311 s dan **senyap total pada konsol yang dilihat manusia**
   karena seluruh kemajuannya hanya ditulis ke serial. Ini bukan panic dan
   bukan deadlock. Butuh keputusan pemilik: diterima sebagai biaya alat
   diagnostik, atau dibuka pekerjaan terpisah untuk memindahkan kemajuan ke
   framebuffer dan/atau memangkas target 100.000 tukar. Tidak ada angka gerbang
   yang diubah menjadi hijau karena entri ini. Teks asli entri ke-38 tetap
   tertulis OPEN pada berkas fault log (log ini append-only, entri lama tidak
   diedit); disposition-nya dicatat pada entri ke-39.
7. **Jendela E8 yang diulang pada ISO F2 — TIDAK SAH, dan ini temuan alat**:
   pemilik mewajibkan jendela G9-LITE/E8 diulang setelah F2 dan setelah F6
   karena registry bertambah. Pengulangan setelah F2 sudah dijalankan pada ISO
   F2 (sidik jari 4.204.544 byte `5E5433F6…` tercantum pada kepala summary dan
   cocok dengan ISO yang dibangun unit ini) dan berhenti di kasus keempat dari
   59: S01 OK, S02 `GANTUNG` 600.021 ms, S03 `GANTUNG` 600.010 ms, S04
   `MESIN-KELUAR`, kode keluar alat 2.
   *Akar penyebabnya di alat, bukan di shell*: `scripts/e8_adversarial_run.ps1:374`
   menghitung kemunculan literal `SHELL_PROMPT` = `"ASL-KERNEL />_ "` — bentuk
   akar yang sengaja saya pertahankan pada F1 agar keempat alat gerbang tetap
   membaca konstanta yang sama. Sejak prompt berposisi, begitu ruang kerja
   berpindah ke `/e8uji` bentuk akar tidak pernah muncul lagi sehingga alat
   menunggu sampai batas, padahal kedua balasan tercetak penuh dan benar:
   `Ruang kerja: /e8uji` dan `Berkas dibuat: /e8uji/a.txt`, masing-masing
   ditutup `ASL-KERNEL /e8uji>_ `. Yang gagal adalah penglihatan alat, bukan
   ketaatan shell.
   *Label `MESIN-KELUAR` alat tidak terbukti*: pada seluruh 4.461 byte serial
   tidak ada satu pun `ASL-CORE-PANIC`, `ASL EXCEPTION`, maupun `FATAL`; 136
   byte terakhir adalah pesan `… kembali ke prompt, shell tetap hidup` diikuti
   prompt sehat sebagai penutup berkas. Sebab berakhirnya proses emulator tidak
   dapat disimpulkan dari arsip ini, dan saya tidak mengklaimnya.
   *Sesi ini juga bukan sesi terkontrol*: byte keluar alat 78 = tepat empat baris
   setup, tetapi serial memperlihatkan ketikan `clear`, `restart`, `shutdown`,
   `test-task`, `task`, `test_task`, backspace, dan baris kosong berulang yang
   masuk lewat papan ketik pada jendela yang terlihat. Shell menanggapi dengan
   benar (`clear` membersihkan layar, empat token tak dikenal menjawab
   `Unknown command`, `test_task` berjalan lalu berhenti oleh satu ketikan
   dengan pesan operator), tetapi kenyataan itu dicatat, tidak dijadikan uji.
   *Keputusan pemilik 2026-10-22* (ditanyakan dengan tiga opsi sebelum alat
   disentuh): hentikan, catat, tunda penyesuaian alat ke R2. Berkas alat tidak
   ada yang diubah; sesi sudah berakhir sendiri. Konsekuensinya saya tulis apa
   adanya: **kewajiban "jendela diulang setelah F2" BELUM TERPENUHI dan
   berpindah ke R2** dengan alasan tercatat (entri ke-42 pada fault log), bukan
   hilang; penutupnya satu suntingan kecil pada pencocok prompt
   `e8_adversarial_run.ps1` lalu jalankan ulang 59 kasus dari awal pada ISO yang
   berlaku dengan pemilik menyaksikan. Kalimat statis `CATATAN bangun` pada alat
   (baris 415-417) juga masih menyebut "11 perintah" dan "`up` BELUM ada" —
   salah untuk ISO F2, dan ikut ditunda ke R2.
