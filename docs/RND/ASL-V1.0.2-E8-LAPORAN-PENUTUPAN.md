# ASL v1.0.2 — E8: LAPORAN PENUTUPAN KERNEL PANIC (P0)

> **LABEL DOKUMEN: LAPORAN PENUTUPAN RESMI E8.** Dokumen kerja dan seluruh
> tabel rinciannya tetap ada di
> `docs/RND/ASL-V1.0.2-E8-AKAR-PENYEBAB-DAN-STATUS.md`; laporan ini adalah
> penutup yang diminta MASTER PROMPT R1-REVISI Bagian 1 LANGKAH 7 dan hanya
> merangkum angka yang sudah ter-commit. Tidak ada satu pun angka di sini yang
> berasal dari ingatan: setiap baris menunjuk artefak atau bagian dokumen kerja.

Tanggal laporan: 2026-10-22. Cabang: `feature/v1.0.2-r1` (PR #7 ke `main`,
belum di-merge). Acuan perangkat uji: ISO `18F40D2D4E18C3B46A88C13090AC1F6FCCE13B4D264A1D5B78B4D6E0A833FAD0`
(4.200.448 byte) dan ELF kernel 325.120 byte
`74405577744896d747b5278fe7ff9a0f8033e0d90ebb2aa5aa8d9b1bdbe287b3`, keduanya
VERIFIED-ON-HOST.

---

## 1. Putusan

**E8 DINYATAAN TERTUTUP oleh pemilik pada 2026-10-22**, dengan empat keputusan
eksplisit pemilik: (1) jendela G9-LITE **sudah disaksikan** dan **LULUS**;
(2) teks panic yang dulu dilaporkan **TIDAK TERSEDIA**; (3) vonis jendela
**LULUS**; (4) temuan produk entri ke-38 **diterima sebagai temuan produk** dan
penanganannya **digabung ke F5** (umpan balik kemajuan di layar untuk pekerjaan
panjang) ditambah dokumentasi varians 64 s versus 310,7 s.

Yang menutup E8 bukan "panic tidak ditemukan", melainkan tiga hal yang
terukur terpisah:

1. **Akar penyebab gejala yang ter-reproduksi terverifikasi sampai berkas:baris**
   — pembekuan yang dilaporkan adalah `test_task` menjalankan rangkaian Gate 3D
   100.000 tukar tanpa titik henti dan tanpa keluaran ke layar (Bagian 3 tabel
   A4 dan Bagian 5.3 dokumen kerja).
2. **Seluruh jalur panic yang dapat diketik satu pengguna sudah ditutup dan
   dijaga test** — A1, A2, A3 diperbaiki; A5 memulihkan bendera interupsi; A6
   membuat panic berikutnya tidak datang tanpa bukti; enam test `panic_guard_*`
   lulus (Bagian 3 dan 4 dokumen kerja).
3. **Pemilik menyaksikan sendiri jendela pengulangan pemicu dan menyatakan
   lulus**, setelah teks panic dinyatakan tidak tersedia.

`E8-STATUS: BELUM-TERREPRODUKSI-SEBAGAI-PANIC` tetap menjadi label jujur untuk
jalur panic: tidak ada satu pun masukan yang saya kirim menghasilkan
`[ASL-CORE-PANIC]`, dan tidak ada teks panic pemilik untuk dicocokkan. Yang
ditutup adalah tuntutan misi (akar penyebab + error anggun + prompt kembali),
bukan klaim bahwa panic laporan pemilik pernah teramati.

## 2. Gejala, pemicu, dan apa yang sebenarnya terjadi

| Aspek | Fakta terukur |
|---|---|
| Pemicu | Satu ketikan `test_task` pada prompt `ASL-KERNEL> ` (boot segar, `-m 256M`) |
| Waktu jalan | **64,0 s sampai 310,7 s** pada ISO yang sama, tergantung kondisi: 310.681 ms saat boot segar, 272.600 ms dan 64.031 ms di tengah sesi penuh, 507 ms pada pemanggilan kedua dalam sesi yang sama |
| Keluaran ke layar | **Nol baris.** Seluruh progres ditulis `early_serial_write_str` dan hanya muncul di berkas serial |
| Panic | **Nol.** Tidak ada `ASL-CORE-PANIC` / `ASL EXCEPTION` / `FATAL` di kelima sesi jendela; satu-satunya kemunculan ketiga frasa itu adalah baris kepala alat yang mendefinisikannya |
| Deadlock | **Tidak.** Pada 300,0 s mesin sudah mencetak 75.004/100.000 tukar dan terus mencetak; dengan batas 600 s rangkaian selesai, TEST 1..15 PASS, prompt kembali |
| Kenapa terasa seperti hang | Operator melihat banner boot + prompt kosong; dua tangkapan layar ter-commit (`Analysis/reports/e8_l6_jendela-sesi-A-token_2026-10-21.png`, `e8_l6_jendela-sesi-A2-hidup_2026-10-22.png`) |
| Harga perbaikan A4 | Poll dua port tiap iterasi membuat satu ketikan menghentikan rangkaian, dan menaikkan waktu boot segar dari 178,8 s (ISO `aa778060`, pra-perbaikan) menjadi 310,7 s (ISO `18f40d2d`). Kedua angka berasal dari ISO dan beban berbeda, jadi yang dilaporkan adalah kisaran, bukan sebab-akibat tunggal |
| Tombol penghenti | Terukur `elapsed_ms=25528` dengan prompt kembali dan `pesan_operator=True` pada sesi `2026-10-21_22-49-02` (Bagian 5.1) |

## 3. Akar penyebab yang diverifikasi (LANGKAH 2)

Enam lokasi, semuanya `asl-kernel/asl-kernel-core/src/lib.rs` kecuali penjaga
regresi; tabel lengkap dengan nomor baris sebelum/sesudah ada di Bagian 3
dokumen kerja.

- **A1** — asersi tata letak bingkai interupsi di TEST 1 membunuh kernel untuk
  perintah yang seharusnya hanya alat uji. Sekarang periksa lunak + pesan galat.
- **A2** — `spawn_task(0x2000).unwrap()` di TEST 11. Sekarang `match Ok/Err`,
  indeks lewat `checked_sub` dan penyaring `MAX_KERNEL_TASKS`.
- **A3** — `GLOBAL_TASK_MANAGER` tidak pernah dinolkan antar-panggilan, jadi
  `test_task` **kedua** dalam satu sesi memicu enam `assert_eq!` keadaan
  penjadwal. Ini jalur paling dekat dengan laporan pemilik.
- **A4** — rangkaian 100.000 tukar tanpa pemantauan masukan = pembekuan yang
  benar-benar disaksikan. Sekarang `suite_masukan_masuk()` (COM1 LSR bit 0 atau
  PS/2 status bit 0) dipoll tiap iterasi; satu ketikan berhenti + keadaan
  dikembalikan + pesan galat + prompt.
- **A5** — `test_apic` meninggalkan bendera interupsi CPU menyala; sekarang
  `cli` dan dilaporkan sebagai `[TEST 7] ... -> PASS`.
- **A6** — panic handler lama membuang `PanicInfo`. Sekarang menulis
  `KONTEKS:` (pesan + berkas:baris:kolom) ke serial **dan** framebuffer,
  `ALAMAT frame penanganan`, dan `JEBAKAN TERAKHIR` (vektor/RIP/CR2/kode galat),
  dengan penanda reentransi.

## 4. Gerbang yang harus hijau sebelum penutupan (LANGKAH 4-5-6)

| Gerbang | Angka terukur | Kelas |
|---|---|---|
| Unit test | **202 lulus / 0 gagal** pada 30 suite (196 lama + 6 `panic_guard_*`) | VERIFIED-ON-HOST + VERIFIED-ON-CI |
| Build rilis | selesai, **nol baris warning / nol galat**, ELF 325.120 byte identik pada dua lingkungan build | VERIFIED-ON-HOST |
| fmt/clippy | tidak hijau harfiah karena **140 blok deviasi** dan **137 diagnostik warisan** yang sudah ada sebelum E8; yang ditegakkan adalah **nol BARU** pada baris hasil kerja, dihitung `scripts/r1_new_only_gate_check.py` | VERIFIED-ON-HOST |
| Lantai memori | ratchet **3392K lulus**; janji Bagian 5 **`-m 3M` tetap MERAH** (serial 0 byte) — deviasi tidak diubah menjadi hijau | VERIFIED-ON-VM |
| Sesi adversarial | Pass A 43 payload keluar 0; Pass B 59/59 [OK] keluar 0; `trap_de` mencetak blok konteks lengkap; tujuh sesi berjendela | VERIFIED-ON-VM |
| CI terkelola | **24 run** atas cabang R1, **12 pasangan** peristiwa, seluruhnya `conclusion=success`; tiap run 202/0, 30 suite, 19 `warning` profil test, 13 `Compiling` 0/0, ELF runner 325.184 byte. Ekstrak: 24 berkas milik run cabang R1; total `ci_hosted_*` ter-commit 73 | VERIFIED-ON-CI / VERIFIED-ON-CI-API |
| Jendela LANGKAH 6 | Lima sesi terdokumentasi di Bagian 5.3; **disaksikan pemilik dan dinyatakan LULUS 2026-10-22** | VERIFIED-ON-VM-VISIBLE + kesaksian pemilik |

## 5. Batas bukti — yang TIDAK boleh dibaca sebagai sudah terbukti

1. **Jalur panic Rust tetap hanya terbukti di tingkat statis dan unit test
   sisi host**, bukan VERIFIED-ON-VM: sesudah A1-A6 tidak ada masukan yang bisa
   diketik pada lintasan kanonik yang mencapai `panic_handler` — itulah isi
   perbaikannya. Yang terbukti di mesin untuk tuntutan "cetak konteks" adalah
   jalur jebakan CPU (`trap_de`).
2. **Konter `VERIFIED-ON-HARDWARE` tetap 0.** Jendela yang disaksikan pemilik
   berlangsung di emulator pada mesin pengembangan; kelasnya
   VERIFIED-ON-VM-VISIBLE. Klaim perangkat keras fisik tidak pernah dibuat dan
   tidak berubah oleh vonis LULUS ini.
3. **CI terkelola tidak menjalankan QEMU maupun alat reproduksi E8.** Hijau di
   CI membuktikan tree lulus uji + build, bukan mereplikasi isi berkas bukti.
4. **Sepuluh commit cabang tidak pernah punya run sendiri** (`007d3dd`,
   `54abf73`, `4e865c0`, `223e7f6`, `6bbd77d`, `939cc95`, `d6c5b12`, `093a7d6`,
   `b9acd5b`, `ca582fa`) karena satu peristiwa push hanya memicu run untuk tip
   saat itu. Ini fakta cara kerja CI, bukan kelalaian.
5. **Utang ekstrak G8 tidak pernah bisa nol pada commit terakhir rantai.** Yang
   tertutup sampai unit 7b = 12 pasangan (24 berkas milik run cabang R1, audit di
   `Analysis/reports/e8_l7b_audit-utang-ekstrak_2026-10-22.txt`); pasangan atas
   commit-unit ini baru bisa masuk commit berikutnya, yaitu unit pertama BAGIAN 2.
6. **Durasi `test_task` tidak stabil dan tidak dapat diprediksi dari dalam
   sesi** (64,0 s sampai 310,7 s). Setiap batas waktu untuk kasus ini adalah
   tebakan yang harus dinyatakan sebagai tebakan; resep resmi kini
   `-CaseTimeoutSeconds 600 -BootTimeoutSeconds 240`.
7. **Dua kelemahan alat dicatat, tidak diperbaiki diam-diam.** (a) Kode keluar
   2 `scripts/e8_adversarial_run.ps1` berarti "penanda crash ATAU shell tidak
   kembali", dan frasa tetap pada baris 563 mencetak "crash terlihat" tanpa
   mengkondisikannya pada penanda crash — membaca ringkasan tanpa membaca
   artefak dapat menyimpulkan panic yang tidak terjadi. (b)
   `scripts/ci_hosted_extract.py` tidak gagal keras ketika log mentah kosong:
   unduhan pertama run 39644006778 mengembalikan 0 byte dan menghasilkan ekstrak
   `suite=0 passed=0`; berkas itu tertangkap sebelum di-stage. Alat E8
   `scripts/g9_final_p8_run.ps1` memang tidak boleh diubah pada rantai R1.
8. **Kolom waktu berkas TSV alat bernama `detik_mm` padahal isinya
   milidetik.** Nama tidak diubah supaya artefak lama dan baru tidak beda
   skema; konversi detik kini ditulis berdampingan di dokumen.

## 6. Tindak lanjut yang diikat oleh keputusan pemilik

- **F5** menerima entri ke-38: umpan balik kemajuan di layar (framebuffer) untuk
  pekerjaan panjang, plus menyatakan varians durasi 64 s versus 310,7 s.
- **F1-F7** mengubah registry kanonik dari 11 menjadi 13 perintah; sesudah F2
  (`up`) dan F6 (`rename`) hidup, kasus C15 dan C26/C27 pada Pass penuh menguji
  jalur aslinya dan **jendela E8 diulang**.
- Janji `-m 3M` tetap MERAH dan menjadi pekerjaan terpisah; tidak ada angka
  gerbang yang diubah menjadi hijau karena penutupan ini.
- Entri ke-38 pada `ACCOUNTABILITY_FAULT_LOG` tetap tertulis OPEN pada
  berkasnya (log ini append-only, entri lama tidak diedit); disposition-nya ada
  di entri ke-39.

## 7. Rujukan silang

- Dokumen kerja + tabel lengkap: `docs/RND/ASL-V1.0.2-E8-AKAR-PENYEBAB-DAN-STATUS.md`
  (Bagian 1-7, termasuk 5.1, 5.2, 5.3).
- Accountability: `docs/ROADMAP/ACCOUNTABILITY_FAULT_LOG.json` entri ke-30
  sampai ke-39; `violations_recorded` tetap 0.
- Bukti mentah: `Analysis/reports/e8_*` (tujuh sesi adversarial, tabel kasus,
  serial, stderr QEMU, kalibrasi batas, dua tangkapan layar, audit utang
  ekstrak) dan `Analysis/reports/ci_hosted_*`.
- Commit yang dirujuk laporan ini adalah commit sebelumnya pada rantai yang
  sama: `1f3e3c2` (unit 5c), `4de1dfb` (unit jendela LANGKAH 6), `b17a175`
  (unit 6b), `4c3e0d2` (unit 7a). Sesuai aturan, hash commit hanya dirujuk di
  commit berikutnya, dan laporan ini tidak merujuk hash dirinya sendiri.
