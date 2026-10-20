# PHASE-7 T4 — Banner versi v1.0.2 -> v1.1.0 (lima string tampil) + run penerimaan G9-lite berjendela

Tanggal: 2026-10-21 (+0700). Branch `feature/phase7` (tip sebelum commit ini: `9d2ae57`), PR #4 terbuka.
Status: **SELESAI dengan kriteria exit terukur** — suntingan string diterapkan dan diverifikasi di level byte,
gerbang G4 direplikasi pada tree T4 (30 suite 172/0; release dari-nol 13 `Compiling`, 0 warning / 0 error),
sidik jari ELF **dan** ISO diukur ulang pada tree ini, dan run penerimaan **berjendela** mencapai M6 +
prompt `ASL-OS>` dengan string versi baru terukur tampil di serial. Hosted CI atas commit T4 belum ada
pada saat penulisan — bukti run atas commit ini masuk lewat commit berikutnya, bukan diklaim sekarang.

Klasifikasi bukti keseluruhan: **VERIFIED-ON-HOST** (§2-§6) dan **VERIFIED-ON-VM-VISIBLE** (§5, cold boot
berjendela di QEMU disaksikan pemilik). Tidak ada klaim perangkat keras; jumlah `VERIFIED-ON-HARDWARE`
tetap **0**. Tidak ada klaim screenshot; bukti visual = serial log + jendela yang disaksikan.

---

## 0. Otorisasi, dan mengapa scope-nya harus ditanyakan lebih dulu

Kontrak `.rs` PHASE-7 dari prompt induk pemilik mengizinkan **hanya dua** jenis perubahan kode:
(a) rename fungsi test — sudah selesai di T3, dan (b) **satu baris banner versi** pada T4, dengan
catatan "otorisasi eksplisit pemilik". Tidak ada yang lain.

Pengukuran di tree menunjukkan butir (b) tidak sesempit bunyinya. String `v1.0.2` berada pada
**enam baris** di `asl-kernel/asl-kernel-core/src/lib.rs`, dan **lima** di antaranya adalah string yang
benar-benar dicetak keluar oleh artefak yang di-boot:

| Baris | Bentuk | Dampak bila tidak diubah |
|---|---|---|
| 1126 | banner/logo `ASL Kernel v1.0.2` | — (ini "satu baris banner" yang dimaksud kontrak) |
| 1145 | `(*vga).write_str("Kernel initialized: ASL Core Kernel v1.0.2\n")` | layar VGA menyebut versi lama |
| 1150 | `(*fb).write_str("Kernel initialized: ASL Core Kernel v1.0.2\n")` | framebuffer menyebut versi lama |
| 1233 | `early_serial_write_str(" [ OK ] Kernel initialized: ASL Core Kernel v1.0.2\n")` | **serial log** menyebut versi lama |
| 1287 | `// MILESTONE 1 (v1.0.2): ...` | tidak berdampak — komentar sejarah |
| 1822 | `let uname_msg = "ASL Core Kernel v1.0.2 (x86_64-unknown-none) ..."` | keluaran `uname` menyebut versi lama |

Dua pilihan yang jujur: menyunting **hanya 1126** menghasilkan artefak boot yang menyebut dua versi
berbeda pada layar dan serial yang sama; menyunting **seluruhnya** melampaui kata "satu baris" dan
karenanya butuh perluasan otorisasi tertulis. Keputusan ini **tidak** diambil oleh agen. Pemilik
ditanya lewat pertanyaan pilihan dan menjawab:

> **Keputusan pemilik (T4 scope): "5 string tampil, komentar tetap"** — ubah `v1.0.2` -> `v1.1.0` pada
> baris 1126, 1145, 1150, 1233, 1822; **pertahankan** baris 1287 (`// MILESTONE 1 (v1.0.2): ...`)
> sebagai label sejarah.

Jawaban ini dicatat sebagai perluasan otorisasi tertulis atas butir (b) dan menjadi isi utama
`ACCOUNTABILITY_FAULT_LOG.json` entri #22.

## 1. Koreksi atas dokumen status saya sendiri (enumerasi tidak lengkap)

Instruksi T4 yang saya tulis di `docs/ROADMAP/HANDOFF_STATE.json` menyebut jumlahnya dengan benar
("muncul ENAM kali") tetapi hanya mendaftar **lima** lokasi — baris 1822 (`uname_msg`) tidak terdaftar.
Dokumen status dengan angka benar dan daftar cacat adalah umpan keputusan yang buruk: pengikut daftar,
bukan angka, akan meninggalkan satu string versi lama di keluaran `uname`. Enumerasi kini dilengkapi
menjadi enam baris. Aturan yang diturunkan: **daftar lokasi wajib dihasilkan dari pengukuran
(`grep -n` + nomor baris), dan jumlah yang dinyatakan wajib dicocokkan dengan panjang daftar pada saat
penulisan.** Ini cacat kategori sama dengan entri #19/#20/#21 (harapan atau angka yang tidak diuji), hanya
saja kini pada dokumen status, bukan pada bukti.

## 2. Suntingan, diverifikasi di level byte

Berkas `asl-kernel/asl-kernel-core/src/lib.rs` ber-CRLF; suntingan dikerjakan di level byte lewat
`logs/apply_t4_banner.py` dengan asersi per-baris target (tepat satu occurrence `v1.0.2`) dan asersi
pasca-tulis, bukan pencarian teks global.

| Ukuran | Pra-edit | Pasca-edit |
|---|---|---|
| occurrence `v1.0.2` | 6 | **1** (baris 1287, sesuai perintah owner) |
| occurrence `v1.1.0` | 0 | **5** |
| panjang berkas | 119.159 B | 119.159 B (nol delta — `v1.0.2` dan `v1.1.0` sama panjang) |
| jumlah baris / CRLF | 2.694 / 2.693 | 2.694 / 2.693 (akhir baris utuh) |
| SHA256 berkas | `fe151dc8feee68a6...d3e5cf01` | `6fa894836c05dc5d...86c00405` |
| `git diff --numstat` | — | 5 tambah / 5 kurang, **satu berkas** |

NOL identifier, NOL signature fungsi, NOL logika, NOL `Cargo.*`, NOL berkas workflow;
`CHECKLIST_STATE.json` tidak disentuh. Kontrak `.rs` PHASE-7 tetap berlaku: ini butir (b) dengan
perluasan tertulis dari owner (§0).

## 3. Gerbang G4 diulang pada tree T4 (VERIFIED-ON-HOST)

| Gerbang | Perintah | Hasil terukur |
|---|---|---|
| G4 [1] | `cargo test --workspace --lib` (log scratch `logs/t4_gate_test.log`) | exit **0**; 30 baris `Running unittests`; **172 passed / 0 failed / 0 ignored**; 172 baris `test <nama> ... ok` (172 unik); 21 baris `warning:` profil test (non-blocking); 0 `error`. 27 baris `Compiling`, termasuk `asl-kernel-core` pada baris 83 — jadi test sungguh dibangun dari sumber yang baru disunting |
| G4 [2] | hapus `target/x86_64-unknown-none/release`, lalu `cargo build -p asl-kernel-bin --target x86_64-unknown-none --release` tanpa env `RUSTFLAGS` (log scratch `logs/t4_gate_release.log`) | exit **0**; **13 baris `Compiling`** + `Finished` dalam **4,30s**; **0 baris `warning:`**, **0 baris `error:`** |

Roster T3 tetap utuh pada tree ini: 8 nama test baru hadir, 8 nama lama absen — perubahan string banner
tidak menggerakkan satu pun nama test.

## 4. Sidik jari PER-TREE: ELF dan ISO diukur ulang (aturan entri #20/#21)

Tidak ada angka yang diwarisi dari T2/T3.

| Artefak | Tree T3 (`62c1c31`) | Tree T4 (commit ini) | Bergerak? |
|---|---|---|---|
| ELF Windows, ukuran | 243.568 B | **243.568 B** | tidak |
| ELF Windows, SHA256 | `9609ab8b1566bda2...ba88e890` | **`3c9c4ee9c0912339...207c4248`** | **ya** |
| ISO, ukuran | 4.118.528 B | **4.118.528 B** | tidak |
| ISO, SHA256 | `f7dd6ecace8d2995...a9d05800` | **`da5b91f95cd10ed4...a7abce56`** | **ya** |
| ELF di dalam ISO | — | `3c9c4ee9...` = ELF hasil build | cocok |

Cara ukur ISO: `build_kernel.ps1 -Action iso` dijalankan **dua invokasi terpisah**; keluarannya
`cmp` = **byte-identik** (SHA256 kedua berkas sama), jadi klaim reproduksibilitas per-platform (hasil
PHASE-7 T1) tetap berlaku pada tree T4. GPT Disk GUID ter-pin `e181e879-31ec-3d41-a200-b4997a320a0d`
dari epoch tetap 1767225600. Biner yang tertanam di `target/iso_root/boot/asl-kernel.elf` punya SHA256
sama dengan ELF hasil build dari-nol, membuktikan ISO membawa muatan T4, bukan sisa build sebelumnya.

Pola yang kini konsisten: **ukuran** biner tidak bergerak (penggantian lima karakter sepadan), **isi**
biner bergerak. Untuk banner yang berada di kode non-test, pergeseran ini terjadi di muatan yang
di-load (`.rodata`), berbeda dari rename `#[cfg(test)]` di T3 yang hanya menggerakkan metadata — karena
itu §5 menuntut bukti boot, bukan hanya hash. Pin ISO historis T1 (`9cd49028...`) dan T3 (`f7dd6eca...`)
tidak lagi menjelaskan tree ini.

## 5. Run penerimaan G9-lite T4 — berjendela, disaksikan pemilik (VERIFIED-ON-VM-VISIBLE)

Skrip: `scripts/g9_lite_t4_run.ps1` (varian baru; `scripts/g9_acceptance_run.ps1` yang sudah ter-commit
tidak disunting). Tiga alasan varian, semuanya terukur pada batasnya:
1. nama keluaran berbasis tanggal -> tabrakan antar-invokasi pada hari yang sama; varian memakai
   cap-waktu per-invokasi sehingga tidak dapat menimpa artefak G9 2026-10-20 yang ter-commit,
2. catatan pada skrip lama masih menyatakan "ISO tidak byte-reproducible" — pernyataan yang dibatalkan
   PHASE-7 T1 dan tidak boleh diwariskan ke bukti baru,
3. gerbang versi T4 **ditambahkan** agar kegagalan banner menjadi FAILURE, bukan harapan.

Perintah terdokumentasi di summary: `qemu-system-x86_64 -machine q35 -vga std -m 256M -cdrom <ISO>
-serial file:<log> -no-reboot -no-shutdown` — **tanpa `-display none`** (aturan G9: mata manusia wajib
menyaksikan jendela sampai M6 + prompt shell). QEMU 10.2.50 (v10.2.0-14086-g840320663e).

Hasil (`Analysis/reports/g9_lite_t4_summary_2026-10-21_05-43-47.log`):

| Kriteria | Terukur |
|---|---|
| M6 `Kernel Cold Boot Complete` | **True** |
| prompt `ASL-OS>` | **True** |
| crash (`ASL EXCEPTION` / `ASL-CORE-PANIC` / `FATAL`) | **False** |
| occurrence `v1.1.0` pada serial | **2** (gerbang: >= 1) |
| occurrence `v1.0.2` pada serial | **0** (gerbang: == 0) |
| stderr QEMU | 0 baris |
| jendela | **dibiarkan TERBUKA** (PID 19912) agar disaksikan; ditutup dengan `-StopQemu` |
| exit code runner | **0 = PASS** |

## 6. Perbandingan terukur terhadap bukti G9 2026-10-20

Klaim "banner berubah, sisanya sama" diuji, bukan dinyatakan. `diff` antara serial G9 lama
(`Analysis/reports/g9_acceptance_serial_2026-10-20.log`) dan serial T4 ini menghasilkan **tepat dua**
perubahan baris pada berkas yang sama-sama 55 item `Get-Content` / 54 baris ber-newline (`wc -l`; baris
terakhir `ASL-OS> ` tidak ditutup newline):

```
39c39
<    ASL Kernel v1.0.2
>    ASL Kernel v1.1.0
42c42
<    [ OK ] Kernel initialized: ASL Core Kernel v1.0.2
>    [ OK ] Kernel initialized: ASL Core Kernel v1.1.0
```

Tidak ada baris keluaran boot lain yang berbeda — termasuk tidak ada perubahan pada angka milestone,
angka memori, maupun urutan inisialisasi. Inilah bukti dampak yang paling sempit yang bisa diberikan
sebuah perubahan string: muatan yang di-boot menghasilkan log boot identik kecuali pada versi yang ia
cetak.

## 7. Bukti hosted yang ikut menutup utang pada commit ini

Dua ekstrak CI yang terhutang dari `[PHASE-7-3b]` (run atas `9d2ae57`) masuk repo lewat commit ini,
mengikuti pola G8 "bukti run atas commit N masuk lewat commit N+1":

| Berkas | Run | Job | Angka gerbang | ELF Linux | llvm-cov |
|---|---|---|---|---|---|
| `ci_hosted_39508817039_2026-10-20.log` | push @ `9d2ae57`, 1.379 baris mentah | 2 (test-and-build + coverage) | 30 suite / 172 passed / 0 failed; 13 `Compiling`; 0 warning / 0 error; `Finished` 2,14s | 243.568 B | 13 baris, identik |
| `ci_hosted_39508823480_2026-10-20.log` | pull_request @ `9d2ae57`, 1.401 baris mentah | 2 | sama; `Finished` 1,81s | 243.568 B | 13 baris, identik |

Keduanya mengonfirmasi `git diff --numstat 62c1c31 9d2ae57 -- *.rs Cargo.toml Cargo.lock` **kosong**
(commit sebelumnya docs/evidence-only) lewat roster 8 nama baru / 0 nama lama dan tabel ringkasan yang
sama baris-per-baris dengan baseline host. Rantai terukur `gh run list --workflow asl-ci-minimal
--limit 100`: **33 baris, 33 success** = 14 rantai main + 19 `feature/phase7`; berkas ekstrak ter-commit
naik 24 -> **26**.

## 8. Batas yang dinyatakan

- Hosted tidak menjalankan langkah ISO maupun QEMU, jadi tidak ada angka ISO yang diklaim dari runner;
  dua run atas commit T4 ini sendiri **belum** diekstrak (menyusul di commit berikutnya).
- Run §5 adalah cold boot pada **QEMU** (virtual) dan menjadi bukti G9/G9-lite, bukan
  `VERIFIED-ON-HARDWARE`; konter itu tetap 0. Tidak ada klaim screenshot.
- Angka ELF/ISO di §4 sah **untuk tree T4 ini saja**; fase berikutnya wajib mengukur ulang.
- Delta ukuran antar-platform tidak lagi ditulis sebagai konstanta 8 B (entri #21); yang dilaporkan
  adalah hasil ukur per tree per platform.

## 9. Berkas yang dihasilkan commit ini

Masuk repo: `asl-kernel/asl-kernel-core/src/lib.rs` (5 baris), `scripts/g9_lite_t4_run.ps1` (baru),
tiga artefak `Analysis/reports/g9_lite_t4_{serial,qemu_stderr,summary}_2026-10-21_05-43-47.log` (baru),
dua ekstrak CI §7 (baru), laporan ini, `docs/ROADMAP/ACCOUNTABILITY_FAULT_LOG.json` (entri #22,
append-only), `docs/ROADMAP/HANDOFF_STATE.json` (status T4 + koreksi enumerasi §1), serta baris status
T4 di `README.md`, `RELEASE_NOTES.md`, dan `Analysis/reports/QODER_FINAL_REPORT.md`.

Scratch ter-ignore: `logs/apply_t4_banner.py`, `logs/add_fault22.py`, `logs/t4_gate_test.log`,
`logs/t4_gate_release.log`, `logs/t4_iso_run1.log`, `logs/t4_iso_run2.log`, `logs/t4_iso_run1.iso`
(salinan ISO invokasi pertama untuk `cmp`), `logs/extract_ci.py` + `logs/measure_9d2_ci.py` +
`logs/covrows_*.txt` (generator/pengukur ekstrak §7), `logs/ci_39508*_raw.log`, `logs/ci_39508*_meta.json`.

## 10. Langkah berikutnya (T5, CLOSEOUT)

Ukur ulang seluruh gerbang di tip; tulis laporan sepuluh baris R2/R3; sinkronkan dokumen; re-pin hash
ELF/ISO tree final; ekstrak dua run atas commit T4 ini bersama run tip; lalu **merge PR #4 dengan
merge commit biasa** (`--squash` / `--rebase` DILARANG), pertahankan branch `feature/phase7`, dan
verifikasi proteksi branch pasca-merge.
