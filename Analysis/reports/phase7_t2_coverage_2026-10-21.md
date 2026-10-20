# PHASE-7 T2 — Coverage terukur `cargo-llvm-cov` + job CI non-blocking (K6)

Tanggal: 2026-10-21 (+0700). Branch `feature/phase7`, PR #4 terbuka.
Status: **SELESAI dengan kriteria exit terukur** — baseline coverage tercantum dan
ter-commit, job CI `coverage` dipasang NON-BLOCKING, ambang ditetapkan **setelah**
baseline diketahui (bukan sebelum), satu berkas `.yml` berubah, **nol `.rs` berubah**.

Klasifikasi bukti keseluruhan: **VERIFIED-ON-HOST** untuk baseline (diukur di mesin
ini) dan **VERIFIED-ON-CI** untuk job baru (hijau di runner hosted). Tidak ada
klaim perangkat keras; jumlah `VERIFIED-ON-HARDWARE` tetap **0**.

---

## 1. Kriteria T2 vs pemenuhan terukur

| Kriteria misi | Pemenuhan | Bukti |
|---|---|---|
| Coverage **terukur** untuk `asl-storage`, `asl-bootloader`, `asl-kernel-core-secondary` | Diukur pada host: regions 87,72% / functions 80,89% / lines 86,82% (TOTAL tiga crate, 6.027 region, 3.718 baris) | `Analysis/reports/coverage_baseline_t2_2026-10-21.log` §1–§2 (keluaran mentah 99 baris dikutip utuh) |
| **Tanpa angka coverage yang dikarang** (G7) | Setiap digit dibaca dari blok mentah; agregat per-crate dihitung ulang dari baris per-berkas dan di-assert sama dengan baris `TOTAL` | transkrip §2 ("CHECK aritmetika … LULUS") |
| Job CI **non-blocking** | Job `coverage` ditambahkan dengan `continue-on-error: true` **dan** tidak didaftarkan sebagai required status check | `.github/workflows/asl-ci-minimal.yml` (butir 4 di bawah) |
| Ambang **hanya setelah** baseline | Baseline diukur lebih dulu (03:43), ambang ditetapkan kemudian (80% lines) | §3 |
| NOL `.rs`, nol dependency workspace baru (G1) | `git status --porcelain` saat commit mengukur: 1 `.yml` berubah, 6 berkas bukti/fase baru, 5 dokumen governance dianotasi, nol `.rs` dan nol `Cargo.*`. `cargo-llvm-cov` = tooling host di-install, bukan crate dependency | §7 |
| G4 tetap utuh | **Direplikasi LOKAL pada tree commit ini** (bukan hanya dipercaya dari run lama): 30 suite **172 passed / 0 failed** (exit 0, 21 baris warning profil test non-blocking) dan release `x86_64-unknown-none` **0 warning / 0 error** dari build dari-nol (13 baris `Compiling`, `Finished` 6.30s, ELF 243.568 B, SHA256 identik dengan biner sebelum dibersihkan). Run hosted atas commit T2 belum ada pada saat penulisan — itu diukur dan diekstrak di T5 | §5b; angka hosted terdahulu di §5 |

Catatan cakupan yang tidak boleh dilewatkan: **59 test pada 3 suite** dieksekusi di
bawah instrumen, sedangkan gerbang G4 adalah **172 test pada 30 suite**. Dua angka
itu berbeda objek ukur dan tidak saling menggantikan.

---

## 2. Baseline yang terukur

Dikeluarkan `cargo llvm-cov 0.9.1` (di-install agen 2026-10-21 atas izin eksplisit
pemilik lewat pertanyaan tooling; `rustc 1.99.0`, target host
`x86_64-pc-windows-msvc`, **tanpa** `--target` dan **tanpa** env `RUSTFLAGS`):

| Metrik (TOTAL 3 crate) | Nilai |
|---|---|
| Regions | 6.027 total, 740 missed → **87,72%** |
| Functions | 225 total, 43 missed → **80,89%** |
| Lines | 3.718 total, 490 missed → **86,82%** |
| Branches | 0 region cabang → **TIDAK DIUKUR** (butuh nightly `-Zcoverage-options=branch`) |

Agregat per crate (dihitung dari baris per-berkas, bukan dari alat):

| Crate | Regions | Functions | Lines |
|---|---|---|---|
| `asl-bootloader` | 289/315 = 91,75% | 13/14 = 92,86% | 233/255 = 91,37% |
| `asl-kernel-core-secondary` | 448/487 = 91,99% | 35/40 = 87,50% | 305/329 = 92,71% |
| `asl-storage` | 4.550/5.225 = 87,08% | 134/171 = 78,36% | 2.690/3.134 = 85,83% |

Penyebab terendah yang **diukur dari source** (transkrip §3): `asl-storage/src/ahci.rs`
0 dari 5 fungsi (modul uji tidak menyebut simbol AHCI mana pun; jalurnya PCI descriptor +
MMIO HBA), `asl-storage/src/ata.rs` 2 dari 14 fungsi (modul uji hanya menyentuh
konstruktor `AtaPioDriver::new(AtaChannel::Primary, false)`).

Stabilitas: run kedua pada rev yang sama menghasilkan **13 baris data tabel byte-identik**;
setelah baris `Compiling`/`Finished` dibuang, kedua run sama sebagai multiset baris.
Tidak seperti hash ISO di T1, angka coverage tidak butuh pen-pin-an.

---

## 3. Keputusan ambang (dan kenapa 80)

Misi menyatakan ambang **hanya boleh ditetapkan setelah baseline diketahui**.
Baseline host: **lines 86,82%**. Ambang yang dipasang: `--fail-under-lines 80`.

Alasan angka, berurutan sesuai bobot bukti:

1. **Delta platform nyata dan belum diukur untuk coverage.** Pada metrik lain repo ini
   delta Windows↔Linux terbukti ada (ELF 243.568 B vs 243.560 B). Runner hosted memakai
   Linux dan build LLVM sendiri, jadi angka coverage CI **diduga** berbeda beberapa digit;
   belum ada pasangan pengukuran Linux↔Linux. Margin 6,82 poin di bawah baseline adalah
   tempat untuk dugaan itu sampai bukti masuk.
2. **Advisory, bukan gerbang rilis.** Job ini tidak menahan merge (lihat §4), jadi
   ambang tidak boleh membusuk menjadi "status sertifikasi".
3. **Bukan 86% dan bukan tanpa ambang.** Tanpa ambang job hanya mencetak angka dan tidak
   bisa menangkap regresi; ambang 86% akan membaca baseline host sebagai fakta
   lintas-platform, yang justru dilarang oleh disiplin bukti repo ini.

**Mekanisme ambang diuji, bukan diasumsikan** (host, rev yang sama):

| Perintah tambahan | Exit code |
|---|---|
| `... --summary-only --fail-under-lines 80` | **0** (80 ≤ 86,82) |
| `... --summary-only --fail-under-lines 90` | **1** (90 > 86,82) |

Kejanggalan yang dicatat jujur: pada `cargo-llvm-cov 0.9.1` ini jalur gagal **tidak
mengeluarkan pesan teks** — keluaran stderr hanya log build/test biasa, dan tabel ringkasan
pergi ke stdout. Yang menandai pelanggaran ambang adalah **exit code 1**, bukan teks.
Siapa pun yang mengparsir log CI (termasuk ekstrak bukti otomatis di repo ini) karena itu
tidak boleh mencari kata "threshold"; ia harus membaca tabel `TOTAL` dan status job.

---

## 4. Job CI: non-blocking dua lapis

```
jobs:
  coverage:
    name: coverage (non-blocking, advisory)
    runs-on: ubuntu-latest
    continue-on-error: true
    steps: checkout, rust-toolchain@stable (components: llvm-tools),
           cargo install cargo-llvm-cov --locked,
           cargo llvm-cov -p asl-storage -p asl-bootloader
                        -p asl-kernel-core-secondary --lib --summary-only
                        --fail-under-lines 80
```

Kenapa "non-blocking" ditegakkan **dua** kali:

- `continue-on-error: true` — kegagalan langkah (termasuk ambang tidak tercapai atau
  `cargo install` gagal) tidak membuat **job** merah.
- Branch protection `main` (keputusan pemilik, "PR + 1 approval") mengunci
  `required_status_checks.contexts = ["test-and-build"]` dengan `strict: true`. Nama
  `coverage` **tidak** ada di daftar itu, jadi ia tidak pernah menjadi syarat merge.
  Konsekuensi yang diterima: `strict: true` hanya menuntut konteks yang tercantum, maka
  penambahan job baru tidak menuntut perubahan proteksi — dan agen **tidak** menyentuh
  proteksi branch.

Cakupan job sengaja tetap **tiga crate** (kontrak T2), bukan seluruh workspace.
`llvm-tools` dipasang sebagai komponen toolchain, bukan dependency crate.

**Risiko yang diakui sebelum push:** file YAML **tidak bisa divalidasi di host ini**
(`python -c "import yaml"` → ModuleNotFoundError; tidak ada parser YAML lain, dan tidak
ada alat dipasang hanya untuk itu). Gerbang format adalah Actions sendiri. Mitigasi yang
dipilih: struktur job baru meniru `test-and-build` baris-per-baris (indentasi, bentuk
`steps`, `with:`), karena itu kemungkinan kerusakan struktural kecil; dan bila ternyata
rusak, akibatnya adalah `test-and-build` **tidak terdaftar** (bukan merah) sehingga
terlihat sebagai check yang hilang pada PR — bukan kegagalan yang tersamar. Kalau itu
terjadi, aturannya berlaku: **STOP dan lapor**, tanpa menyentuh `main`.

---

## 5. Bukti hosted + empat ekstrak CI yang tertunda

Dua run atas commit T2 ini (push dan pull_request) diekstrak pada T5/closeout; pola
yang sama sudah dipakai sejak T1. Selain itu, **empat ekstrak yang menjadi utang sejak
[PHASE-7-1b]/[PHASE-7-1c] ikut ditutup pada commit ini** (koreksi anti-regres, bukan
penulisan-ulang berkas lama):

| Run | Event | Commit teruji | Angka yang terbaca |
|---|---|---|---|
| `39492616260` | push | `7876749` [PHASE-7-1b] | 30 suite **172/0**, 13 `Compiling`, release 0 warning / 0 error, ELF Linux 243.560 B |
| `39492625070` | pull_request | `7876749` | identik |
| `39493034754` | push | `489d4ee` [PHASE-7-1c] | identik |
| `39493041973` | pull_request | `489d4ee` | identik |

Berkas: `Analysis/reports/ci_hosted_<id>_2026-10-20.log` (penamaan mengikuti tanggal run
UTC; keempatnya ASCII, LF, 67–68 baris). Keenam ekstrak terdahulu **tidak ditulis ulang** —
maka generator ekstrak (`logs/extract_ci.py`, scratch ter-ignore) kini difilter per **job**
dan hanya memuat blok coverage bila workflow memang punya job itu.

Recount rantai (terukur `gh run list --workflow asl-ci-minimal --limit 60`):
**25 run `asl-ci-minimal` `conclusion=success`** = 14 pada rantai commit yang menjadi
`main` (`8b829d8` → `6623dd8`) + **11** pada `feature/phase7` (`7e2ac24`, `0a49ba7`×2,
`b9a4057`×2, `eaea57c`×2, `7876749`×2, `489d4ee`×2). Tingkat bukti tetap dibedakan dan
jumlah berkas `Analysis/reports/ci_hosted_*.log` ikut diukur, bukan diingat: kini **18**
= 14 ekstrak yang sudah ada sebelum T2 + 4 berkas baru pada commit ini, dan
18 + 5 (log mentah pernah ditarik dan diukur tapi tidak punya ekstrak ter-commit) +
2 (hanya metadata API) = **25** run hijau. Angka 25 ini adalah **status sebelum** dua run
atas commit T2 masuk; ia akan bertambah dan diukur ulang di T5.

---

## 5b. Gerbang G4 direplikasi LOKAL pada tree commit ini

Karena commit ini mengubah satu berkas workflow
(`.github/workflows/asl-ci-minimal.yml`) dan menambah/menanotasi dokumen bukti,
gerbang G4 diulang di host pada tree ini **sebelum commit** — persis
seperti yang dilakukan pada T1 (§6d bukti T1). Keluaran hidup di scratch ter-ignore
`logs/t2_gate_test.log` dan `logs/t2_gate_build_full.log`; angka di bawah adalah hasil ukur
dari berkas itu, dan transkrip ini adalah bukti ter-commit-nya.

| Langkah | Perintah | Hasil terukur |
|---|---|---|
| [1] test | `cargo test --workspace --lib` | **30** baris `test result:`; jumlah = **172 passed / 0 failed**; exit **0**; 21 baris `warning:` (profil test, non-blocking — gerbang 0-warning misi berlaku untuk build release) |
| [2] release dari-nol | `cargo build --package asl-kernel-bin --target x86_64-unknown-none --release` | **13** baris `Compiling`, `Finished` release [optimized] **6.30s**, exit **0**, **0 `warning:`**, **0 `error:`** |
| Artefak | `ls -l` + `sha256sum` | ELF **243.568 B** (Windows); SHA256 `ee1274be011e90df57df52fb1944f40839df92206b8240214b0a459b0be2def2` |

Definisi "dari-nol" dinyatakan eksplisit: direktori
`asl-kernel/target/x86_64-unknown-none/release` **dihapus** sebelum langkah [2] sehingga
baris `Compiling` benar-benar muncul (pada run pertama, tanpa penghapusan direktori, cargo
hanya mencetak `Finished` dalam 0,05s tanpa satu pun baris `Compiling` — itu build cache
dan tidak dipakai sebagai bukti). Build
berulang ini sekaligus menghasilkan satu temuan yang berguna bagi jejak audit: SHA256 ELF
hasil build dari-nol **identik** dengan SHA256 ELF yang ada sebelum direktori itu dihapus
(`ee1274be…`, tercatat sebelum penghapusan). Artinya biner `x86_64-unknown-none` pada tree
ini **reproducible per-platform** di host ini — sejalan dengan reproduksibilitas ISO yang
di-pin pada T1, dan menjadi pembanding untuk delta 8 B Windows↔Linux yang sudah
didokumentasikan (243.568 B vs 243.560 B).

Kategori bukti langkah ini: **VERIFIED-ON-HOST**. Yang belum terukur di sini dan tetap
menjadi pekerjaan T5: (a) run hosted pertama dengan **dua job** (`test-and-build` wajib +
`coverage` advisory) atas commit T2, dan (b) angka coverage pada runner **Linux** —
pasangan Windows↔Linux untuk coverage belum ada, jadi baseline §2 tidak dipindahkan ke CI.

---

## 6. Batasan dan pekerjaan lanjutan (ditinggalkan sebagai utang yang disengaja)

1. Coverage ini adalah "berapa baris berkas yang pernah dijalankan", **bukan** "berapa
   logika produksi yang teruji": modul uji berada di dalam `lib.rs` yang sama, sehingga
   `asl-storage/src/lib.rs` 98,74% sebagian berasal dari baris test itu sendiri.
2. Angka **produksi-murni** (test dipisah lewat `--ignore-filename-regex`) belum diukur.
3. **Branch coverage tidak ada** pada toolchain stable; tangga DAL-C menuntut **MC/DC**,
   jadi angka di file ini tidak boleh dikutip sebagai kemajuan menuju DAL-C.
4. Crate lain (termasuk `asl-kernel-core`, driver, HAL) tidak diinstrumentasi → tidak ada
   angka workspace-wide.
5. `ahci.rs`/`ata.rs` tetap 0%/4%: menutupnya butuh harness port-I/O/MMIO, yang merupakan
   keputusan fase tersendiri (dan menyentuh `.rs` — di luar kontrak PHASE-7).
6. Inventarisasi fungsi mana yang tidak tereksekusi di `block.rs`/`vfs.rs` tidak dilakukan
   di T2; itu langkah lanjutan yang sah tanpa mengubah sumber.

---

## 7. Kepatuhan gerbang

- **G1**: nol dependency crate workspace baru — `Cargo.toml`/`Cargo.lock` tidak berubah;
  `cargo-llvm-cov` adalah biner tooling host.
- **G2**: status gerbang tidak dinaikkan. Tidak ada klaim "certified"/"MC/DC"; coverage
  hanya angka operasional. `CHECKLIST_STATE.json` **tidak disentuh**.
- **G3**: `MajorFrameSchedule` tidak disentuh.
- **G4**: test workspace (30 suite, 172/0) dan release 0 warning / 0 error **direplikasi
  lokal pada tree commit ini** (§5b), bukan hanya dirujuk dari run lama. Run hosted atas
  commit T2 diukur dan diekstrak di T5; angka hosted terdahulu (§5) tidak digeser.
- **G5**: tidak ada pipeline merah. Semua run yang diamati sampai penulisan ini
  `conclusion=success`.
- **G7**: nol angka karangan — setiap digit punya baris sumber di transkrip §7 (blok
  mentah) atau di tabel ekstrak.
- **G8**: nol amend/rebase/reset/force. Rantai commit di atas `feature/phase7` tumbuh
  append-only; hash commit T2 dicatat di **entri berikutnya** (dokumen T5/closeout dan
  pesan commit susulan), bukan di dalam commitnya sendiri.
- **G9**: tidak berlaku di T2 (tidak ada run penerimaan berjendela pada fase ini). Run
  berjendela berikutnya = G9-lite pada **T4**, dan wajib disaksikan pemilik.
- **R1–R4**: log akuntabilitas append-only (entri baru untuk keputusan tooling +
  kejanggalan exit-code), laporan fase ini, ringkasan sepuluh baris di chat, dan
  `QODER_FINAL_REPORT.md` disinkronkan append-only.

Rantai commit PHASE-7 yang relevan: `60f621c` → `7e2ac24` → `0a49ba7` → `b9a4057` →
`eaea57c` [PHASE-7-1] → `7876749` [PHASE-7-1b] → `489d4ee` [PHASE-7-1c] →
**`[PHASE-7-2]` (commit ini, hash dicatat di entri berikutnya)**.

---

## 8. Reproduksi

```
# satu kali per mesin (tooling host)
cargo install cargo-llvm-cov --locked

# baseline (diukur 2026-10-21, rev 489d4ee)
cd asl-kernel
cargo llvm-cov -p asl-storage -p asl-bootloader \
  -p asl-kernel-core-secondary --lib --summary-only

# ambang advisory yang dipakai CI
cargo llvm-cov -p asl-storage -p asl-bootloader \
  -p asl-kernel-core-secondary --lib --summary-only --fail-under-lines 80
```

Jalankan di host Windows **tanpa** `--target` dan tanpa env `RUSTFLAGS`; artefak
instrumentasi masuk ke `asl-kernel/target/llvm-cov-target/` (ter-ignore git).
---

## 9. [PHASE-7-2b, 2026-10-21] Hasil hosted pertama — pasangan platform coverage TERUKUR

Dua run atas commit `[PHASE-7-2]` (`60ef58a`), keduanya `conclusion=success` dan keduanya
sudah punya **dua** job:

| Run | Event | `test-and-build` | Job `coverage` |
|---|---|---|---|
| `39498246825` | push | 30 suite **172 passed / 0 failed**, 13 baris `Compiling`, `Finished` release 2,25s, 0 `warning:` / 0 `error:`, ELF Linux **243.560 B**, 21 baris warning profil test | **success**, 13 baris tabel terukur |
| `39498253429` | pull_request (merge ref PR #4) | identik (release `Finished` 3,72s) | **success**, 13 baris tabel terukur |

**Job coverage benar-benar berjalan, bukan cache.** Langkah `[1/2]` (118 baris log) memuat
`Compiling cargo-llvm-cov v0.9.1` lalu `Installed package `cargo-llvm-cov v0.9.1` (executable
`cargo-llvm-cov`)`; langkah `[2/2]` (107 baris) memuat empat baris `Compiling` terinstrumentasi
(`asl-driver-pci`, `asl-bootloader`, `asl-kernel-core-secondary`, `asl-storage`) dan
`Finished `test` profile [unoptimized + debuginfo] target(s) in 2.49s` (push) / `0.70s`
(pull_request). Rentang stempel waktu langkah di log (21:12:18,46 -> 21:12:22,02 UTC pada run
push) konsisten dengan durasi yang dilaporkan cargo sendiri, dan build instrumentasi yang sama
di host memakan 2,50s — jadi tidak ada tanda langkah noop. Exit code 0 berarti ambang advisory
`--fail-under-lines 80` **terpenuhi di Linux**.

**Temuan utama: delta platform coverage TERUKUR = NOL pada pasangan ini.** Ketiga belas baris
ringkasan Linux (12 berkas + `TOTAL`) **sama baris-per-baris dan angka-per-angka** dengan
baseline host Windows, termasuk `TOTAL` regions 6.027/740 = **87,72%**, functions 225/43 =
**80,89%**, lines 3.718/490 = **86,82%**, branches `0 0 -`. Perbandingan ini sah karena
`git diff --numstat 489d4ee 60ef58a -- *.rs Cargo.toml Cargo.lock` **kosong** (baseline host
diukur pada rev `489d4ee`, run hosted pada rev `60ef58a`, tree sumber identik). Nama berkas
hanya dibedakan pemisah jalur (`asl-storage\src\lib.rs` di host vs `asl-storage/src/lib.rs`
di Linux).

**Batas yang harus dibaca bersama temuan itu.** (1) Toolchain **tidak** identik: host
`rustc 1.99.0` (`x86_64-pc-windows-msvc`) vs runner `rustc 1.99.1`
(`stable-x86_64-unknown-linux-gnu`; terukur dari baris `rustc 1.99.1 (58cf8f9c3 2026-10-13)`
pada log run). Jadi "nol delta" adalah hasil ukur **satu pasangan**, bukan hukum, dan tidak
boleh dikutip sebagai "coverage tidak bergantung platform". (2) Delta 8 B pada ELF
(243.568 B Windows vs 243.560 B Linux) tetap nyata dan tidak berubah oleh temuan ini —
coverage mengukur profil test, bukan artefak release. (3) Margin 6,82 poin pada ambang
**tidak** disetel ulang: ambang tetap advisory, dan penyetelan ulang adalah keputusan fase
tersendiri (utang §6 butir lain tetap berlaku). (4) Kategori bukti blok ini **VERIFIED-ON-CI**;
baseline resmi tetap §2 (VERIFIED-ON-HOST) sampai ada kebijakan lain.

**Koreksi append-only atas pernyataan sebelum ukur.** Transkrip baseline §6c dan butir
"Delta platform" README §6 menulis bahwa angka CI **mungkin** berbeda beberapa digit karena
belum ada pasangan pengukuran. Keduanya dibiarkan apa adanya sebagai keadaan pada saat
penulisan; hasil ukur bagian inilah yang menggantikannya, dan ratifikasinya tercatat sebagai
entri #19 di `docs/ROADMAP/ACCOUNTABILITY_FAULT_LOG.json`. Tidak ada riwayat commit yang
ditulis-ulang (G8).

**Bukti dan recount.** Dua ekstrak baru:
`Analysis/reports/ci_hosted_39498246825_2026-10-20.log` (94 baris) dan
`Analysis/reports/ci_hosted_39498253429_2026-10-20.log` (95 baris) — yang pertama dalam
keluarga ini yang memuat blok `job coverage`; keduanya ASCII murni (non-ASCII 0) dan LF.
Rantai diukur ulang dengan `gh run list --workflow asl-ci-minimal --limit 60`: **27 run**
`asl-ci-minimal` `conclusion=success` = 14 rantai `main` (`8b829d8` -> `6623dd8`) + **13**
`feature/phase7`; jumlah berkas `Analysis/reports/ci_hosted_*.log` terukur = **20**
(18 sebelum commit ini + 2 baru), sehingga tingkat bukti menjadi **20 / 5 / 2** dan
20 + 5 + 2 = **27**. Nol `.rs`, nol `Cargo.*`, `CHECKLIST_STATE.json` tidak disentuh pada
commit ini.
