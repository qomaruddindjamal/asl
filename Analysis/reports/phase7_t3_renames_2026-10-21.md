# PHASE-7 T3 — Rename semantik fungsi test M-1 (delapan rename) + gerbang G4 direplikasi pada tree rename

Tanggal: 2026-10-21 (+0700). Branch `feature/phase7` (tip sebelum commit ini: `adcd250`), PR #4 terbuka.
Status: **SELESAI dengan kriteria exit terukur di host** — delapan rename diterapkan, **nol asersi berubah**,
gerbang G4 direplikasi pada tree ini (30 suite 172/0; release 0 warning / 0 error), roster test dicocokkan
terhadap baseline ter-commit, dan satu **temuan baru yang mengubah asumsi saya sendiri** tercatat lengkap
di §6. Run hosted atas commit T3 belum ada pada saat penulisan — itu diukur pada langkah berikutnya
(ekstrak `3b`/T5), bukan diklaim sekarang.

Klasifikasi bukti keseluruhan: **VERIFIED-ON-HOST** (§3–§6) dan **VERIFIED-ON-REPO** (§7, dihitung dari
`git ls-files`). Tidak ada klaim perangkat keras; jumlah `VERIFIED-ON-HARDWARE` tetap **0**.

---

## 0. Otorisasi, dan mengapa scope-nya harus ditanyakan lebih dulu

Kontrak `.rs` PHASE-7 dari prompt induk pemilik mengizinkan **hanya dua** jenis perubahan kode:
(a) rename fungsi test pada T3, dan (b) satu baris banner versi pada T4. Tidak ada yang lain.

Temuan yang menjadi dasar T3 ("M-1") **tidak terdefinisi di dalam repo**. Pencarian read-only atas
`Analysis/`, `docs/ROADMAP/*.md`, dan `git log --all` tidak menemukan definisi M-1 maupun daftar
fungsi test yang dianggap menyesatkan; istilah itu hanya ada di prompt induk. Alih-alih menebak
isinya, saya melakukan inventaris read-only terhadap 59 test pada tiga crate yang menjadi lingkup
T2, menilai tiap nama terhadap isi asersi sebenarnya, lalu **menanyakan scope kepada pemilik**.
Jawaban pemilik (eksplisit, lewat pertanyaan pilihan):

| Keputusan pemilik | Isi |
|---|---|
| Lingkup T3 | **8 rename hasil inventaris** (bukan seluruh kandidat, bukan nol) |
| Urutan gerbang | **Lokali dulu, lalu commit + push; ekstrak CI di langkah `3b`** |

Delapan rename itulah yang diterapkan di bawah. Tidak ada perubahan `.rs` lain: diff terukur =
8 baris, masing-masing hanya identifier `fn`.

---

## 1. Kriteria T3 vs pemenuhan terukur

| Kriteria misi | Pemenuhan | Bukti |
|---|---|---|
| Hanya **rename fungsi test** yang berubah (kontrak `.rs` T3) | `git diff --numstat` = `1/1`, `1/1`, `6/6` (8 baris); setiap baris `-`/`+` hanya berbeda pada identifier `fn`, tidak ada baris asersi yang tersentuh | §2 dan §3 |
| **Nol asersi berubah** | Jumlah atribut `#[test]` per berkas sama sebelum & sesudah (9 / 14 / 36); pemisah akhir CRLF tidak berubah (452 / 581 / 2000); delta byte hanya dari panjang nama (`+24`, `-1`, `-4`) | §3 |
| Nama baru tidak tabrakan dengan nama yang ada | Sebelum penerapan: tiap nama lama tepat **1** occurrence sebagai identifier utuh, tiap nama baru **0** occurrence di seluruh repo | §3 |
| Jumlah suite & test tidak berubah, hanya nama | Terhadap baseline ter-commit `Analysis/reports/ci_2026-10-20.log.step1.log`: 172 test `ok` pada 30 suite di kedua sisi; multiset jumlah-test-per-suite identik; himpunan roster pasca-pemetaan lama→baru **sama persis**; selisih = tepat 8 nama lama / 8 nama baru | §5 |
| G4 tetap utuh pada tree rename | `cargo test --workspace --lib` exit 0 = 30 suite **172 passed / 0 failed**; `cargo build -p asl-kernel-bin --target x86_64-unknown-none --release` dari-nol = **13 baris `Compiling`**, **0 `warning:`**, **0 `error:`** | §4 |
| G1: nol dependency workspace baru | Tidak ada `Cargo.toml`/`Cargo.lock` yang berubah (`git status` pada commit ini) | §8 |
| G8: nol amend/rebase/reset/force-push | Semua kerja di atas `adcd250` berupa commit baru; koreksi atas asumsi saya sendiri (ELF) dilaporkan apa adanya di §6, bukan dengan menghapus jejak | §6, §8 |
| Proteksi: perubahan `main` lewat PR | Kerja tetap di `feature/phase7`; merge tetap lewat PR #4 pada T5 | §8 |

---

## 2. Peta rename dan alasan terukur (tiap alasan dibaca dari tubuh test itu sendiri)

Baris "yang sebenarnya diassert" diambil dengan mengekstrak tubuh fungsi dan mencetak baris
ber-`assert`/`unwrap` (skrip scratch `logs/t3_test_bodies.py`), bukan dari ingatan nama fungsi.

| # | Nama lama | Nama baru | Lokasi | Yang sebenarnya diassert | Mengapa nama lama menyesatkan |
|---|---|---|---|---|---|
| 1 | `test_valid_handover` | `test_valid_handover_and_usable_memory_total` | `asl-bootloader/src/lib.rs:294` | `h.validate() == Ok(())` **dan** `h.calculate_usable_memory() == 0x1000_0000` | Nama lama hanya menjanjikan validitas handover; uji juga mengunci total memori terpakai — bagian kedua tidak terlihat dari nama |
| 2 | `test_latch_guard_page_degraded_reason_recorded_in_engine` | `test_latch_guard_page_degraded_reason_recorded_in_latch` | `asl-kernel-core-secondary/src/lib.rs:415` | `latch.arm()`, `latch.last_reason() == REASON_STACK_GUARD_DEGRADED`, `latch.fault_count() == 1`, `latch.engine_state() == FaultDetected` | Reason tercatat pada **latch** (`last_reason`), bukan pada engine; engine hanya diamati statusnya. Nama lama memindahkan lokasi rekaman ke objek yang salah |
| 3 | `test_ata_timeout_and_error_handling` | `test_ata_pio_driver_default_fields` | `asl-storage/src/lib.rs:827` | `driver.io_base`, `ctrl_base`, `device_type == AtaDeviceType::None`, `total_sectors() == 0` setelah `AtaPioDriver::new(AtaChannel::Primary, false)` | **Tidak ada timeout dan tidak ada jalur error** yang diuji. Ini uji field default konstruktor; nama lama mengklaim cakupan temporal/error-path yang tidak ada |
| 4 | `test_raid1_member0_failure_behavior_symmetric` | `test_raid1_primary_member_write_failure_degrades_volume` | `asl-storage/src/lib.rs:1159` | Fault di-inject **hanya di member 0**; assert `write_sectors == Err(IoError)`, `member_status(0) == Failed`, `member_status(1) == Healthy`, `volume_state() == Degraded`, `read_sectors == Ok(())` | Tidak ada perbandingan **simetris** (member 1 tidak pernah di-fault-kan). Kata "symmetric" adalah klaim sifat yang tidak diuji |
| 5 | `test_mbr_partition_parsing_and_boundary_validation` | `test_mbr_partition_parsing_of_valid_entries` | `asl-storage/src/lib.rs:101` | MBR dengan 2 entri valid: `count == 2`, `is_gpt == false`, tipe/start_lba/sector_count kedua entri | Semua entri yang diassert adalah entri **valid**; tidak ada penolakan batas/out-of-range. Frasa "boundary validation" tidak punya padanan asersi |
| 6 | `test_raid0_striping_sector_mapping_across_boundaries` | `test_raid0_striped_roundtrip_and_oob_rejection` | `asl-storage/src/lib.rs:908` | `total_sectors() == 64`, tulis-baca 10 sektor yang melintasi 3 stripe (`read_data == write_data`), lalu dua penolakan `InvalidLba` (`read_sectors(64,1)`, `write_sectors(60,5)`) | Yang diuji adalah **roundtrip + penolakan di luar batas**, bukan tabel pemetaan sektor stripe itu sendiri (tidak ada assert yang membandingkan pemetaan LBA→(disk, offset)) |
| 7 | `test_raid_metadata_crc32_known_answer_vector` | `test_crc32_known_answer_vector` | `asl-storage/src/lib.rs:1243` | Satu assert: `computed == 0xCBF4_3926` | Nilai itu adalah known-answer CRC-32 umum; tidak ada **vektor metadata RAID** yang diuji, sehingga prefiks "raid_metadata" menyiratkan objek yang salah |
| 8 | `test_raid_metadata_reservation_and_lba0_protection` | `test_raid_metadata_reservation_lba0_protection_and_update_guards` | `asl-storage/src/lib.rs:1466` (sampai `:1767`) | 301 baris: validasi parameter reservasi (13 `InvalidParameter`), proteksi LBA 0/MBR (pola MBR/GPT ditulis lalu dibaca ulang utuh), **dan** jalur update/guard metadata (12 rujukan `update`, penghitung `generation`, `CorruptedMetadata`, `AlreadyExists`) | Nama lama berhenti di reservation+LBA0 padahal separuh uji menjaga jalur **update**; bagian itu tak terlihat dari nama |

Catatan kejujuran: nomor 3 dan 4 saya baca sendiri baris-per-baris sebelum mengusulkan rename (saat
inventaris); nomor 1, 2, 5–8 verifikasinya adalah ekstrak tubuh fungsi di atas. Tidak ada rename yang
didasarkan pada asumsi "mungkin uji ini melakukan X".

---

## 3. Mekanisme penerapan (gagal-tertutup, byte-level, akhir-akhir CRLF)

Rename diterapkan oleh `logs/apply_t3_renames.py` (scratch, ter-ignore) yang bekerja pada **byte** dan
menghitung occurrence dengan batas identifier `(?<![A-Za-z0-9_])nama(?![A-Za-z0-9_])`, bukan substring.
Prosesnya fail-closed: kalau jumlah occurrence nama lama bukan 1, atau nama baru sudah ada, atau jumlah
`#[test]` berubah, skrip berhenti tanpa menulis.

Hasil terukur (dicetak skrip itu sendiri):

| Berkas | rename | delta byte | CRLF sebelum→sesudah | `#[test]` |
|---|---|---|---|---|
| `asl-kernel/asl-bootloader/src/lib.rs` | 1 | +24 | 452 → 452 | 9 (tetap) |
| `asl-kernel/asl-kernel-core-secondary/src/lib.rs` | 1 | -1 | 581 → 581 | 14 (tetap) |
| `asl-kernel/asl-storage/src/lib.rs` | 6 | -4 | 2000 → 2000 | 36 (tetap) |

Tiga hal yang membuat penerapan ini aman terhadap higiene berkas: (a) jumlah CRLF sama dengan jumlah
LF total sebelum dan sesudah, jadi **tidak ada satu pun LF telanjang** yang masuk — berkas tetap CRLF
seperti keadaan ter-commit-nya; (b) tidak ada baris yang ditambah/dihapus, hanya teks pada satu baris;
(c) nama baru diuji tabrakan di seluruh repo lebih dulu (nol occurrence).

SHA256 berkas pasca-rename (untuk penelusuran balik):
`asl-bootloader/src/lib.rs` = `3376ee8631686824ee75df550cbd011e4ce53515d66a06e0f7f348eb64e14221`,
`asl-kernel-core-secondary/src/lib.rs` = `bcfe95686a77f2ff20c999952afe41629a222698fb02195a460f9b85c1033163`,
`asl-storage/src/lib.rs` = `029e77a04b0d2a902fd95c6af89d601ca4a39f44762d524f103004e667c2f9e6`.

Diff penuh (`git diff -U0 -- '*.rs'`) hanya delapan baris `fn` — diprint di commit ini sebagai bukti.

---

## 4. Gerbang G4 direplikasi LOKAL pada tree rename

Definisi "dari-nol" sama seperti T2 §5b: direktori `asl-kernel/target/x86_64-unknown-none/release`
**dihapus** sebelum langkah [2], karena build yang kena cache hanya mencetak `Finished` dalam 0,05s
tanpa baris `Compiling` dan itu bukan bukti 0-warning.

| Langkah | Perintah | Hasil terukur |
|---|---|---|
| [1] test | `cargo test --workspace --lib` | **30** baris `test result:`; jumlah = **172 passed / 0 failed**; exit **0**; 21 baris `warning:` (profil test, non-blocking — gerbang 0-warning misi berlaku untuk build release) |
| [2] release dari-nol | `cargo build --package asl-kernel-bin --target x86_64-unknown-none --release` | **13** baris `Compiling`, `Finished` release [optimized] **5,14s**, exit **0**, **0 `warning:`**, **0 `error:`** |
| Artefak | `ls -l` + `sha256sum` | ELF **243.568 B** (Windows); SHA256 `9609ab8b1566bda2b9b4040e09a0c1ef3f3a24519c8c6df19dea209bba88e890` |

Keluaran hidup di scratch ter-ignore: `logs/t3_gate_test.log`, `logs/t3_gate_build_full.log`
(transkrip di atas dibaca dari berkas ini, bukan diketik ulang dari ingatan). Tidak ada berkas
bukti ter-commit yang ditimpa. Runner `scripts/ci_local.ps1` **tidak** dijalankan apa adanya —
sesuai aturan standby bahwa `Out-File`-nya menimpa artefak GERBANG AKHIR yang ter-commit.

---

## 5. Roster test dicocokkan terhadap baseline ter-commit

Pembanding: `Analysis/reports/ci_2026-10-20.log.step1.log` (bukti G4 ter-commit, 30 suite / 172 test).
Skrip: `logs/cmp_t3_baseline.py`. Yang dibandingkan dan hasilnya:

| Pemeriksaan | Hasil |
|---|---|
| Jumlah baris `test ... ok` | baseline **172**, tree rename **172** |
| Jumlah suite | 30 vs 30 |
| Multiset jumlah-test-per-suite (12×0, 2×2, 2×3, 5×4, 1×5, 1×8, 1×9, 1×10, 1×12, 1×14, 1×17, 1×31, 1×36) | **identik** |
| Identitas suite (nama crate, suffix hash artefak dinormalisasi) | **identik**, 30 untuk 30 |
| Selisih roster sebagai himpunan | **8 hanya-di-baseline**, **8 hanya-di-baru** — tidak lebih |
| Tiap nama lama yang hilang punya tepat satu pasangan baru dari peta §2 | **True** (jalur modul `tests::`/`safe::tests::` tetap sama) |
| Himpunan baseline setelah pemetaan lama→baru == himpunan baru | **True** |

Rincian 8 pasangan yang dicetak pemeriksaan itu ada di §2. Satu batasan metode yang perlu disebut:
**posisi/urutan** baris test tidak dibandingkan, karena harness `libtest` berjalan paralel dan urutan
tidak stabil antar-invocation (terlihat pada log baseline, yang bahkan memuat blok header suite
terpisah dari blok hasilnya). Yang dibandingkan adalah himpunan dan multiset, dan itu bagian yang
punya makna.

Rincian per-crate pada tree rename (pairing header→hasil sah di log ini karena urutannya ketat):
`asl-bootloader` 9, `asl-kernel-core-secondary` 14, `asl-storage` 36, `asl-hal` 31, `asl-memory` 17,
`asl-kernel-core` 10, `asl-sched` 12, sisanya lebih kecil atau nol — cocok dengan jumlah `#[test]`
per berkas di §3 untuk tiga crate yang disentuh.

---

## 6. TEMUAN — rename di dalam `#[cfg(test)]` mengubah hash ELF release; muatan yang di-boot tidak

### 6a. Apa yang saya asumsikan, dan bahwa asumsi itu salah pada level byte

Rencana T3 saya semula berbunyi: karena seluruh test berada di balik `#[cfg(test)]` dan tidak dikompilasi
ke target `x86_64-unknown-none`, maka rename "tidak akan mengubah biner". **Ukuran membantahnya.**
Build release dari-nol pada tree rename menghasilkan

- `9609ab8b1566bda2b9b4040e09a0c1ef3f3a24519c8c6df19dea209bba88e890` (tree rename), sedangkan
- `ee1274be011e90df57df52fb1944f40839df92206b8240214b0a459b0be2def2` adalah hash yang tercatat di T2 §5b
  untuk tree pra-rename.

Dua-duanya **bukan** cache dan **bukan** keberuntungan waktu, karena:

| Eksperimen | Log scratch | `Compiling` | Waktu | SHA256 ELF |
|---|---|---|---|---|
| Build dari-nol #1 (tree rename) | `logs/t3_gate_build_full.log` | 13 | 5,14s | `9609ab8b…` |
| Build dari-nol #2 (tree rename) | `logs/t3_gate_build_full2.log` | 13 | 4,52s | `9609ab8b…` |
| Build dari-nol #3 (tree rename, setelah eksperimen kontrol) | `logs/t3_gate_build_full3.log` | 13 | 4,52s | `9609ab8b…` |
| **Build kontrol**: tree `adcd250` (pra-rename), berkas `.rs` dikembalikan dari `git checkout HEAD -- …` | `logs/t3_control_build.log` | 13 | 4,49s | **`ee1274be…`** = persis angka yang tercatat di T2 §5b |

Empat build, tiga varian tree, satu faktor yang berubah: delapan identifier. Build kontrol mereproduksi
hash lama **tepat**, jadi delta bukan berasal dari toolchain, bukan dari lingkungan, dan bukan dari
timestamp. Reprodusibilitas per-platform tetap berlaku — tiga build pada satu tree menghasilkan hash
yang sama. Yang salah adalah klaim "rename test tidak menyentuh biner".

Untuk keamanan kerja, berkas `.rs` hasil rename saya cadangkan ke `logs/t3_renames_backup/` **sebelum**
eksperimen kontrol (dengan SHA256 pencocokan sesudahnya: `3376ee86…`, `bcfe9568…`, `029e77a0…`), sehingga
`git checkout HEAD -- <tiga berkas>` saat kontrol tidak pernah mempertaruhkan pekerjaan yang belum
ter-commit.

### 6b. Di mana persisnya byte berubah (dan di mana tidak)

Pembanding byte: `logs/t3_elf_head.bin` (kontrol, `ee1274be…`) vs `logs/t3_elf_renamed.bin` (`9609ab8b…`).
Alat: `logs/t3_elf_sections.py` (parser ELF sendiri) dan `logs/t3_elf_segments.py`.

| Fakta terukur | Nilai |
|---|---|
| Ukuran kedua berkas | 243.568 B keduanya |
| Jumlah byte berbeda | **6.193**, semuanya pada rentang offset file **226.184 … 243.536** |
| String yang isinya berubah | **2** (bukan 6.193): `anon.5dd4cb916819fc9d0023ca56679b5079.78.llvm.13966574243371918884` dan `…79.llvm.13966574243371918884` → `…78/79.llvm.7927364696120587124`; masing-masing **satu digit lebih pendek** |
| `.strtab` | 12.171 B → 12.169 B (**-2**, konsisten dengan dua digit yang hilang) |
| `.symtab` | 5.088 B tetap; isinya berbeda karena field `st_name` bergeser mengikuti tabel string |
| Section yang **identik byte** | `.limine_requests` (264), `.got` (176), **`.text` (142.917)**, `.rodata` (23.875), `.data` (42.000), `.asl_secondary` (4.096), `.comment` (153), `.shstrtab` (98) |
| `.bss` | NOBITS — tidak punya byte di berkas, jadi isinya tidak dibandingkan (dikecualikan secara eksplisit) |
| Program header | 4 `PT_LOAD`, range file-offset identik di kedua build: (4096,+440), (8192,+142.917), (151552,+23.875), (176128,+49.152) |
| Apakah ada byte berbeda di dalam `PT_LOAD` manapun? | **TIDAK** — rentang berbeda 226.184…243.536 berada di luar semua segmen yang dimuat (segmen terakhir berakhir 225.280) |

Kesimpulan yang sah dari angka di atas: **muatan yang benar-benar dimuat bootloader identik bit-for-bit**;
yang berubah hanya metadata simbol (tabel simbol/string) yang tidak ikut dimuat ke memori. `.text` tidak
berubah, jadi tidak ada satupun instruksi kernel yang bergeser.

### 6c. Penjelasan mekanisme — mana yang terukur, mana yang interpretasi

Yang **terukur**: hanya dua string suffiks `…llvm.<19-digit>` yang berganti isi, dan keduanya berada di
tabel simbol. Yang **interpretasi** (tidak saya verifikasi dari sumber LLVM di sini): penamaan
*anonymous global* LLVM menstype hasil hash yang sensitif terhadap isi modul, sehingga teks sumber mana
pun — termasuk di dalam `#[cfg(test)]` yang tidak dikompilasi — dapat mengubahnya lewat jalur hashing
modul/CGU. Saya melaporkan ini sebagai penjelasan yang konsisten dengan bentuk string, **bukan** sebagai
hasil ukur. Nama test itu sendiri **tidak** muncul di ELF (pemeriksaan `grep -c` atas 4 nama: 0 di kedua
biner), jadi yang berubah bukan "test ikut ter-compile", melainkan hash uniquing.

### 6d. Konsekuensi yang harus dipegang fase berikutnya

1. **Hash ELF kernel untuk tree ini adalah `9609ab8b…`**, bukan `ee1274be…`. Angka T2 §5b dibiarkan apa
   adanya karena ia benar untuk tree `489d4ee`/`adcd250` (riwayat tidak ditulis-ulang, G8); entri fault
   log #20 mencatat koreksi asumsi, dan T5 memakai angka baru.
2. **ISO ter-pin T1 (`9cd49028…`) tidak lagi mendeskripsikan tree ini** — dan itu saya **ukur**, tidak
   disimpulkan. Alasannya konkret: `scripts/build_kernel.ps1` menyalin ELF apa adanya ke
   `boot/asl-kernel.elf` (tanpa `strip`/`objcopy`, terverifikasi pada baris 106–112), jadi delta
   `.symtab`/`.strtab` ikut ke dalam ISO. Hasil build ISO pada tree rename: **4.118.528 B** dengan
   SHA256 `f7dd6ecace8d2995944c9370f533a8049447dd73b9d9cd31058030cfa9d05800`. Ukuran tetap, hash
   berubah — persis seperti yang diharapkan dari delta yang hanya bergeser 2 byte di tabel simbol.
   **Determinisme terukur dua kali**: dua invokasi `-Action iso` pada tree yang sama menghasilkan
   `f7dd6eca…` keduanya (`logs/t3_iso_build.log`, `logs/t3_iso_build2.log`; salinan build pertama
   di `logs/t3_iso_build1.iso`). Tanggal/volume dan GPT disk GUID tetap ter-pin
   (`e181e879-31ec-3d41-a200-b4997a320a0d` dari epoch 1767225600), jadi delta ISO bukan artefak waktu.
   Konsekuensi operasional: **T4 wajib menegenerate ISO dari tree pasca-banner dan boot biner itu**, dan
   T5 wajib menuliskan hash yang terukur — jangan membawa `9cd49028…` sebagai "hash ISO sekarang".
3. **Semua perubahan teks `.rs` ke depan berpotensi menggerakkan hash ini**, termasuk satu baris banner
   T4. Ini memperkuat (bukan melemahkan) alasan T1 menegakkan reproduksibilitas **per-platform**: biner
   tetap deterministik untuk satu tree, hanya saja "fingerprint tree" bukan fungsi dari loaded image saja.
4. Batas klaim: tidak ada pengukuran QEMU/boot pada langkah ini. Bukti bahwa perbedaannya tidak
   memengaruhi perilaku boot adalah **struktur ELF** (di luar `PT_LOAD`), bukan run boot. Run G9-lite T4
   tetap yang menjadi bukti boot.

---

## 7. Penelusuran nama lama di artefak historis (append-only)

`logs/t3_traceability.py` memindai 125 berkas tracked non-`.rs` dengan pencocokan batas identifier.
Empat berkas memuat nama lama, total **19 occurrence**, dan **tidak satu pun ditulis ulang**:

| Berkas | occurrence | Status |
|---|---|---|
| `Analysis/reports/ci_2026-10-20.log.step1.log` | 8 (satu per nama) | transkrip keluaran CI/host — bukti historis, dibiarkan |
| `Analysis/reports/coverage_baseline_t2_2026-10-21.log` | 8 | keluaran llvm-cov mentah — bukti historis, dibiarkan |
| `Analysis/reports/QODER_EXECUTION_LOG.md` | 2 (`…_recorded_in_engine`) | log eksekusi ter-append — entri lama dibiarkan, rujukan baru memakai nama baru |
| `audit/asl-core-kernel/TEST_RESULTS.md` | 1 (`test_valid_handover`) | temuan audit 2026-10-13 — artefak pihak-pihak historis, dibiarkan |

Cara pembaca masa depan memetakan lama→baru: **peta §2 di dokumen ini** + pesan commit `[PHASE-7-3]`
+ entri #20 `docs/ROADMAP/ACCOUNTABILITY_FAULT_LOG.json`. Tidak ada perubahan nama di artefak lama,
karena itu akan merusak sifat append-only log (R1) dan menyamarkan fakta bahwa nama lama pernah ada.
Scratch ter-ignore (`logs/*_raw.log`, `logs/t2_*`, `logs/t3_*`) juga tetap memakai nama lama; ia bukan
bukti ter-commit.

---

## 8. Batasan dan pekerjaan lanjutan (utang yang disengaja)

- **Belum ada run hosted atas commit T3.** Sesuai urutan yang dipilih pemilik, commit ini adalah kerja
  lokal + dokumen; dua run (`push` dan `pull_request`, masing-masing berjob `test-and-build` wajib +
  `coverage` advisory) akan diekstrak pada langkah `3b`/T5. G5 tetap berlaku: bila hosted merah, STOP
  dan lapor, tidak menyiasati gerbang.
- **Dua run atas `adcd250` (39499814356 push, 39499821544 pull_request) masih terhutang ekstraknya** —
  itu pekerjaan `3b` berikutnya, bersama ekstrak run T3.
- **Re-pin hash**: `9609ab8b…` (ELF) dan `f7dd6eca…` (ISO) dicatat di sini sebagai angka tree ini.
  Angka resmi rilis baru ditulis di T5 setelah merge, dan T4 akan mengubah keduanya lagi (banner).
- Coverage resmi T2 tidak diukur ulang di sini. Nama fungsi test muncul di transkrip baseline coverage,
  jadi baseline `coverage_baseline_t2_2026-10-21.log` kini memakai nama lama; itu disengaja (bukti pada
  waktunya), dan tidak ada angka coverage yang berubah karena rename tidak mengubah jumlah test.
- `CHECKLIST_STATE.json` tidak disentuh. Nol `Cargo.toml`/`Cargo.lock` berubah; nol dependency baru (G1).
- Nol amend/rebase/reset/force-push; `main` tidak disentuh langsung; branch `feature/phase7` tidak
  dihapus (aturan standby pemilik).

---

## 9. Kategori bukti dan berkas yang dihasilkan commit ini

| Kategori | Klaim |
|---|---|
| VERIFIED-ON-HOST | §3 (penerapan byte-level), §4 (G4 pada tree rename), §5 (pencocokan roster), §6a–§6d (hash, kontrol, lokalisasi byte, ISO) |
| VERIFIED-ON-REPO | §7 (19 occurrence pada 4 berkas tracked, dihitung dari `git ls-files`) |
| VERIFIED-ON-CI | **belum ada untuk T3** — menyusul di `3b`/T5 |
| VERIFIED-ON-VM / ON-HARDWARE | tidak ada klaim baru; `VERIFIED-ON-HARDWARE` tetap **0** |

Berkas bukti baru (ter-commit, semuanya kecuali tiga `.rs` adalah dokumen/scratch-to-evidence):
dokumen ini, anotasi append-only pada `README.md`, `RELEASE_NOTES.md`,
`Analysis/reports/QODER_FINAL_REPORT.md`, entri #20 pada `docs/ROADMAP/ACCOUNTABILITY_FAULT_LOG.json`,
dan pembaruan `docs/ROADMAP/HANDOFF_STATE.json`. Scratch yang tetap ter-ignore dan dapat dibaca ulang:
`logs/t3_gate_test.log`, `logs/t3_gate_build_full.log`, `logs/t3_gate_build_full2.log`,
`logs/t3_gate_build_full3.log`, `logs/t3_control_build.log`, `logs/t3_iso_build.log`,
`logs/t3_iso_build2.log`, `logs/t3_elf_head.bin`, `logs/t3_elf_renamed.bin`, `logs/t3_iso_build1.iso`,
`logs/apply_t3_renames.py`, `logs/cmp_t3_baseline.py`, `logs/t3_test_bodies.py`,
`logs/t3_elf_sections.py`, `logs/t3_elf_segments.py`, `logs/t3_traceability.py`,
`logs/t3_renames_backup/`.

---

## 10. Susulan `[PHASE-7-3b]` - bukti hosted atas commit T3 + dua ekstrak tertunda

Commit T3 (`62c1c31`) hanya mengukur gerbang di host. Sesuai urutan yang dipilih pemilik
("lokali dulu, lalu commit + push, ekstrak di 3b"), bab ini menutup sisi hosted dan sekaligus
membayar dua ekstrak run atas `adcd250` yang tertunda sejak `[PHASE-7-2b]`. Semua angka di bawah
dibaca dari `gh run view <id> --log` (mentah di `logs/`, ter-ignore) oleh penyaring per langkah
yang sama dengan keluarga ekstrak `Analysis/reports/ci_hosted_*.log`.

### 10.1 Empat run, semuanya DUA job dan hijau

| Run | Event | headSha | `Finished` release | ELF Linux (baris `ls -l`) | Ekstrak ter-commit |
|---|---|---|---|---|---|
| 39499814356 | push | `adcd250` | 1,99s | 243.560 B | `ci_hosted_39499814356_2026-10-20.log` (107 baris) |
| 39499821544 | pull_request | `adcd250` | 2,66s | 243.560 B | `ci_hosted_39499821544_2026-10-20.log` (108 baris) |
| 39506682088 | push | `62c1c31` | 1,26s | **243.568 B** | `ci_hosted_39506682088_2026-10-20.log` (107 baris) |
| 39506688219 | pull_request | `62c1c31` | 2,88s | **243.568 B** | `ci_hosted_39506688219_2026-10-20.log` (108 baris) |

`conclusion=success` untuk kedua job pada keempatnya (`test-and-build` wajib + `coverage (non-blocking, advisory)`),
dikonfirmasi dua arah: dari `gh run view --json jobs` (nama job + conclusion) dan dari keberadaan langkah
coverage di dalam log mentah itu sendiri. Angka gerbang identik keempatnya dan identik dengan angka host §4:
30 baris `test result:` dengan total **172 passed / 0 failed**, 172 baris `test <nama> ... ok`,
21 baris warning profil test pada langkah [1/2] (non-blocking), 13 baris `Compiling` + 1 `Finished` pada
langkah [2/2] dengan **0 `warning:` / 0 `error:`**. Baris mentah log: 1.374 (push) dan 1.401 (pull_request).

G5 tidak terpicu: tidak ada run merah pada branch misi.

### 10.2 Rename terbukti TER-ECSEKUSI di runner, bukan hanya terkompilasi di host

Pencocokan memakai batas identifier `(?<![A-Za-z0-9_])nama(?![A-Za-z0-9_])` pada teks langkah [1/2],
bukan substring, supaya pasangan ber-prefix (`test_valid_handover` vs `test_valid_handover_and_usable_memory_total`)
tidak saling ikut menghitung:

| Tree | Nama lama yang hadir | Nama baru yang hadir |
|---|---|---|
| `adcd250` (pra-T3) | 8 | 0 |
| `62c1c31` (T3) | 0 | 8 |

Selisih simetris roster nama test kedua tree = **tepat 16** (8 lama + 8 baru), dan kedua run pada tree yang sama
mempunyai roster yang sama satu sama lain. Pencocokan dibandingkan sebagai himpunan, bukan urutan posisi:
harness libtest paralel sehingga urutan baris tidak stabil antar run (§5 sudah menyatakan ini untuk sisi host).
Job `coverage` memberikan konfirmasi kedua: pada `62c1c31` ia mengompilasi ulang 4 crate (`Compiling asl-driver-pci`,
`asl-bootloader`, `asl-kernel-core-secondary`, `asl-storage`), mencetak `Finished `test` profile` (0,85s push / 0,80s PR),
lalu menjalankan 59 baris `test ... ok` dengan nama baru - jadi jalur instrumentasi juga melihat hasil rename,
bukan hanya jalur gerbang.

### 10.3 Tabel coverage tetap identik WALAU tree berubah - dan penjelasannya diukur, bukan disimpulkan

13 baris tabel ringkasan llvm-cov pada keempat run sama baris-per-baris satu dengan lain, dan sama dengan
baseline host Windows `coverage_baseline_t2_2026-10-21.log` (normalisasi pemisah jalan `\` -> `/` lalu
rapatkan whitespace sebelum membandingkan). TOTAL tetap regions 6027/740 = 87,72%, functions 225/43 = 80,89%,
lines 3718/490 = 86,82%, branches tidak diukur.

Ini bukan kebetulan dan bukan alasan "tree sama", karena untuk `62c1c31` tree-nya justru berbeda:

* `git diff --numstat 489d4ee 62c1c31 -- *.rs` = `+1/-1`, `+1/-1`, `+6/-6` pada tiga berkas (delapan penggantian
  identifier sebaris), dan `wc -l` per berkas identik di kedua rev: **452 / 581 / 2000**.
* Ringkasan llvm-cov dihitung per berkas atas baris/region, bukan atas nama fungsi, sehingga rename tidak
  menggerakkan satu digit pun pada tabel.
* Untuk `adcd250` alasannya berbeda dan lebih kuat: `git diff --numstat 489d4ee adcd250 -- *.rs Cargo.toml Cargo.lock`
  kosong, jadi tree run itu memang identik dengan tree baseline.

Kontras dengan §6: hal yang sama (rename `#[cfg(test)]`) menggerakkan identitas byte ELF tetapi tidak menggerakkan
tabel coverage. Keduanya terukur, dan itu sebabnya keduanya dinyatakan pada level yang diukur.

### 10.4 TEMUAN hosted: delta UKURAN ELF antar-platform menjadi 0 (fault log entri #21)

Baris `ls -l` pada langkah "Bukti footprint ELF hasil build" keluar 243.560 B pada **20 ekstrak ter-commit
sebelumnya** dan pada kedua run `adcd250`, lalu **243.568 B** pada kedua run `62c1c31`. Ukuran ELF Windows pada
tree T3 juga 243.568 B (sebelum rename pun sudah 243.568 B; yang berubah SHA256-nya, `ee1274be...` -> `9609ab8b...`).
Jadi delta antar-platform yang selama ini ditempel ke setiap ekstrak sebagai satu kalimat baku ("Linux 243.560 vs
Windows 243.568 = 8 B") terukur **NOL** pada tree ini.

Akar yang dicatat bukan angka salah, melainkan generalisasi yang tidak diuji: generator ekstrak memakai satu teks
baku untuk semua run. Koreksinya (append-only, 20 ekstrak lama tidak ditulis ulang karena angkanya benar untuk
tree masing-masing): generator kini membangkitkan catatan ELF **per-run**, dan aturan turunannya masuk ke
HANDOFF - footprint ELF dibandingkan pada dua platform setiap kali ada perubahan `.rs`. Kategori ini meneruskan
preseden #19 (retraksi ekspektasi delta antar-platform) dan #20 (invarian yang diasumsikan).

### 10.5 Batas yang dinyatakan terang

* Langkah ISO dan langkah [3]/[4] QEMU tidak berjalan hosted, jadi tidak ada satu angka ISO pun yang diklaim
  berasal dari runner. Angka ISO pada ekstrak T3 (`f7dd6eca...`, 4.118.528 B) adalah pengukuran host Windows §6d
  yang dikutip sebagai konteks.
* Pasangan reproduksibilitas ISO Linux->Linux tetap **belum terukur** (warisan batas T1, tidak berubah di sini).
* Baseline coverage resmi tetap pengukuran host Windows; angka runner Linux dipakai sebagai konfirmasi, tidak
  dipatok sebagai baseline, dan ambang advisory `--fail-under-lines 80` tidak disetel ulang.
* Berkas mentah keempat run tetap hanya ada di `logs/` (`.gitignore:159 **/logs/`); yang dapat diaudit dari repo
  adalah empat ekstrak + bab ini.

### 10.6 Recount rantai dan berkas yang dihasilkan `[PHASE-7-3b]`

Rantai diukur ulang dengan
`gh run list --workflow asl-ci-minimal --limit 100 --json databaseId,headSha,conclusion,event,headBranch`:
**31 baris, 31 `conclusion=success`, 0 bukan-success** = 14 pada rantai yang menjadi `main`
(`8b829d8` -> `6623dd8`) + **17** pada `feature/phase7`. Berkas ekstrak ter-commit = **24**
(20 sebelum commit ini + 4 pada commit ini). Tingkat bukti: 24 ekstrak / 5 log mentah tanpa ekstrak ter-commit /
2 hanya metadata API. Baris `VERIFIED-ON-CI = belum ada untuk T3` pada tabel §9 adalah keadaan saat commit T3
dan tertutup oleh bab ini; §9 tidak ditulis ulang (R1).

Berkas bukti commit ini: empat `Analysis/reports/ci_hosted_*_2026-10-20.log` (107/108 baris, ASCII non-ASCII 0,
LF, CRLF 0), bab ini, entri **#21** `ACCOUNTABILITY_FAULT_LOG.json` (CLOSED, `violations_recorded_delta` 0,
total entri 21), anotasi append-only pada `README.md`, `RELEASE_NOTES.md`, `QODER_FINAL_REPORT.md`, dan
pembaruan `HANDOFF_STATE.json`. Scratch ter-ignore yang dapat dibaca ulang: `logs/ci_<id>_raw.log` +
`logs/ci_<id>_meta.json` (empat run), `logs/ci_chain_2026-10-21.json`, `logs/measure_t3_ci.py`,
`logs/cmp_cov_t3_ci.py`, `logs/cov_detail_t3_ci.py`, `logs/extract_ci.py`, `logs/add_fault21.py`,
`logs/sync_t3b_handoff.py`. Nol berkas `.rs` berubah pada commit ini; `CHECKLIST_STATE.json` tidak disentuh;
nol amend/rebase/reset/force-push; branch `feature/phase7` tidak dihapus; `VERIFIED-ON-HARDWARE` tetap 0.
