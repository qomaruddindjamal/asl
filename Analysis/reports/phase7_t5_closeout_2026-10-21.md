# PHASE-7 T5 — CLOSEOUT TIP `feature/phase7` + MERGE PR #4

Tanggal: 2026-10-21. Branch kerja: `feature/phase7`. Basis pengukuran: tip **`d2f63b7`** (`[PHASE-7-4]`).
Kontrak commit ini: **docs/evidence-only** — nol berkas `.rs`, nol `Cargo.*`, nol berkas workflow, `CHECKLIST_STATE.json` tidak disentuh.

Otorisasi: dokumen misi pemilik 2026-10-21 Bagian 0.2 butir (a) — "P0: tuntaskan T5 PHASE-7 (closeout + merge PR #4 ke main) — DIOTORISASI" — dan Bagian 4 langkah P0. Metode merge ditetapkan Bagian 0.2 butir (b): **merge commit dua-parent, TANPA squash**, proteksi branch Opsi B tetap dihormati.

---

## 1. Yang dikerjakan pada T5

| Butir checklist T5 (`HANDOFF_STATE.json.next_immediate_step`) | Status | Tempat bukti |
|---|---|---|
| (1) Ekstrak dua run hosted atas commit T4 | **SELESAI** | `Analysis/reports/ci_hosted_39511692285_2026-10-20.log`, `Analysis/reports/ci_hosted_39511696696_2026-10-20.log` |
| (2) Ukur ulang SEMUA gerbang di tip | **SELESAI** | `Analysis/reports/t5_closeout_measurements_2026-10-21.log` bagian [1]–[2] |
| (3) Re-pin SHA256 ELF dan ISO untuk tree final, dua invokasi + `cmp` | **SELESAI** | berkas yang sama bagian [2]–[3] |
| (4) Laporan + sinkron dokumen (append-only terverifikasi mekanis) | **SELESAI pada commit ini** | dokumen ini + §5 di bawah |
| (5) Merge PR #4 dengan merge commit biasa | **DIEKSEKUSI SETELAH commit ini hijau** (lihat §6) | hash merge commit dicatat pada commit BERIKUTNYA (aturan G8) |
| (6) JANGAN hapus branch `feature/phase7` | **DIPATUHI** | §6 |
| (7) Verifikasi proteksi branch pasca-merge | **TERJADWAL** — hanya bisa diukur sesudah merge; direkam dan diekstrak pada commit berikutnya | §6 batas yang dinyatakan |
| (8) Ekstrak run pasca-merge atas tip `main` | **TERJADWAL** pada commit berikutnya (pola G8: bukti run atas commit N masuk lewat commit N+1) | §6 |

## 2. Gerbang yang diukur ulang di tip (bukan angka warisan)

Semua angka di bawah dihasilkan `logs/make_t5_evidence.py` dari log dan artefak pada tree ini; salinan mentahnya ter-commit di `Analysis/reports/t5_closeout_measurements_2026-10-21.log`.

| Gerbang | Cara ukur | Hasil terukur | Kelas bukti |
|---|---|---|---|
| Host test | `env -u RUSTFLAGS cargo test --workspace --lib`, exit 0 | **30 suite**, **172 passed / 0 failed**, 172 baris roster `test … ok`, 21 baris `warning:` profil test (non-blocking), 0 baris error | VERIFIED-ON-HOST |
| Build release dari-nol | direktori `target/x86_64-unknown-none/release` dihapus lalu `cargo build --package asl-kernel-bin --target x86_64-unknown-none --release`, exit 0 | **13 baris `Compiling`**, `Finished` 5,70s, **0 `warning:` / 0 `error:`** | VERIFIED-ON-HOST |
| ELF Windows tree ini | `wc -c` + `sha256` | **243.568 B**, SHA256 `3c9c4ee9c09123398835ad0779620327dee901a0a6f5de724eceb91b207c4248` | VERIFIED-ON-HOST |
| ISO tree ini, dua invokasi | `scripts/build_kernel.ps1 -Action iso` dijalankan dua kali lalu dibandingkan byte | **4.118.528 B**, keduanya SHA256 `da5b91f95cd10ed4844c4b189717c5bfd6e71c6c225fb02a83f600f5a7abce56`, **identik byte = `True`** | VERIFIED-ON-HOST |
| Pin reproduksibilitas | pembacaan struktur GPT ISO | header `EFI PART` di offset 512, **Disk GUID `e181e879-31ec-3d41-a200-b4997a320a0d`** (GUID ditentukan dari epoch, bukan acak — kriteria exit T1 tetap berlaku pada tree ini) | VERIFIED-ON-HOST |
| Konsistensi artefak | kepala 128 B ELF dicari di dalam ISO | hadir **tepat 1 kali** → ISO yang di-pin memuat ELF yang di-pin | VERIFIED-ON-HOST |
| Hosted wajib | dua run atas `d2f63b7` | `test-and-build` **success** pada keduanya: 30 suite / 172 / 0, 13 `Compiling`, 0 warning / 0 error, ELF Linux **243.568 B** | VERIFIED-ON-CI |
| Hosted advisory | job `coverage` pada kedua run | **success** (non-blocking), 13 baris ringkasan, TOTAL regions 6027/740 = 87,72%, functions 225/43 = 80,89%, lines 3718/490 = 86,82%, branch tidak diukur | VERIFIED-ON-CI |

**Baseline misi tidak boleh menurun:** 172 passed / 0 failed terpersis sama; tidak ada test yang hilang (roster `test … ok` = 172 baik di host maupun di runner).

## 3. Pasangan platform dan stabilitas pengukuran

- Delta **ukuran** ELF antar-platform pada tree ini = **0 B** (Linux runner 243.568 B vs Windows host 243.568 B). Sesuai aturan fault log entri #21, delta ini adalah **hasil ukur per-tree**, bukan konstanta; riwayat "delta 8 B" pada tree pra-T3 dibiarkan apa adanya di berkas ekstrak lama (append-only).
- Yang bergerak justru **isi** ELF host: `9609ab8b…` (tree `9d2ae57`, pra-banner) → `3c9c4ee9…` (tree `d2f63b7`, pasca-banner) dengan ukuran tetap — konsekuensi yang memang diharapkan dari perubahan string tampilan versi.
- Tabel llvm-cov **tidak** bergerak: blok 13 baris ringkasan pada keenam ekstrak (`39506682088`, `39506688219`, `39508817039`, `39508823480`, dan dua berkas baru commit ini) ber-SHA256 sama, `13db7b350e6f215847a8e284cf2cdc35dca64fd7a7090218597a2abe83174aa0`. Alasannya terukur, bukan diasumsikan: berkas yang berubah pada tree T4 (`asl-kernel-core/src/lib.rs`) bukan milik tiga crate yang diinstrumentasi job coverage, dan jumlah barisnya tetap 2693 di kedua rev (dihitung dengan `git show … | wc -l`).
- Roster rename T3 pada kedua run baru: tepat **8 nama BARU hadir, 0 nama LAMA** (pencocokan batas-identifier dari teks log, bukan pembacaan mata).

## 4. Rantai hosted terukur saat commit ini dibuat

`gh run list --workflow asl-ci-minimal --limit 100 --json databaseId,headSha,conclusion,event,headBranch,createdAt` menghasilkan **35 baris, 35 `success`, 0 bukan-success** = 14 pada rantai yang menjadi `main` (9 `main` + 5 tercatat sebagai `security/asl-v1.0.2-audit`) + **21** pada `feature/phase7`. Berkas ekstrak ter-commit menjadi **28** (26 sebelum commit ini + 2 pada commit ini). Yang **tidak** dihitung sebagai `asl-ci-minimal`: `39417510019 @ cc3fe28` dan sembilan run 2026-10-17 ber-event `dynamic`.

Backlog yang dinyatakan terang: **tujuh** run `asl-ci-minimal` berstatus success **tidak** punya ekstrak ter-commit — empat pada rantai `main` (`45a33c7`, `5829e28`, `bb4a9df`, `08f706a`) dan tiga pada branch audit (`39400212750 @ 5a180f6`, dua run `cc3fe28` = push + pull_request). Angka "28" di atas adalah jumlah berkas ekstrak, bukan klaim bahwa setiap run pernah diekstrak.

## 5. Sinkron dokumen dan verifikasi append-only

Dokumen yang dianotasi pada commit ini: `README.md`, `RELEASE_NOTES.md`, `Analysis/reports/QODER_FINAL_REPORT.md`, `docs/ROADMAP/HANDOFF_STATE.json`, plus laporan ini dan tiga berkas bukti (`t5_closeout_measurements_2026-10-21.log`, dua ekstrak §4).

Setiap suntingan diverifikasi **append-only secara mekanis** (skrip scratch `logs/sync_t5_docs.py` + pemeriksaan lanjutan di atas tree yang sama): `difflib` tidak boleh melaporkan operasi selain `equal`/`insert`/`replace`, dan untuk setiap berkas `git diff --unified=0` memecah seluruh baris yang ditandai hilang lalu menuntut tiap baris itu tetap utuh **sebagai prefiks** sebuah baris baru. Hasil terukur: `README.md` `+3/-2` — kedua baris `-` adalah baris anotasi rantai (173) dan baris delta platform (174) yang **diperpanjang**, masing-masing 4.784 B -> 6.007 B dan 4.918 B -> 6.013 B dengan prefiks lama utuh; `RELEASE_NOTES.md` `+1/-0`; `QODER_FINAL_REPORT.md` `+1/-0`. Berkas keluaran wajib bebas karakter CJK (terukur 0 pada keempat dokumen) dan memakai gaya akhir baris yang sama dengan sebelumnya (README/RN LF, FINAL REPORT CRLF) — G10: konten baru bahasa Indonesia; identifier tetap Inggris.

`ACCOUNTABILITY_FAULT_LOG.json` **tidak** ditambah entri pada commit ini: tidak ada klaim lama yang dibatalkan oleh pengukuran closeout ini — semua angka T5 mengonfirmasi angka T4. (Pembandingan: entri #20/#21/#22 lahir justru karena pengukuran bergerak melawan asumsi.)

## 6. Merge PR #4 dan batas yang dinyatakan

Urutan yang dijalankan setelah commit ini di-push: tunggu `asl-ci-minimal` hijau pada tip → `gh pr merge 4 --merge` (**merge commit dua-parent; `--squash` dan `--rebase` DILARANG**) → branch `feature/phase7` **tidak** dihapus → GET ulang proteksi `main` dan ekstrak run atas tip baru `main`.

Tiga hal yang **tidak** bisa dibuktikan di dalam commit ini dan karena itu dinyatakan sebagai utang, bukan sebagai selesai:
1. **Hash merge commit** — ia belum ada saat dokumen ini ditulis; dicatat pada commit berikutnya (G8 melarang hash sebuah commit dikutip di dalamnya sendiri).
2. **Verifikasi proteksi pasca-merge** — keadaannya hanya bisa dibaca sesudah merge.
3. **Ekstrak run hosted atas tip `main` pasca-merge** — mengikuti pola G8, bukti run atas commit N masuk repo lewat commit N+1.

Kelas bukti dokumen ini: **VERIFIED-ON-HOST** (§2 baris 1–6, §3 perhitungan blok, §5), **VERIFIED-ON-CI** (§2 baris 7–8, §3 roster, §4), dan **TERJADWAL** (§6). Tidak ada klaim **VERIFIED-ON-HARDWARE** (konter perangkat keras tetap 0) dan tidak ada klaim tangkapan layar. Run penerimaan berjendela terakhir yang disaksikan pemilik tetap yang T4 (`Analysis/reports/g9_lite_t4_*_2026-10-21_05-43-47.log`); jendela G9 berikutnya dijadwalkan pada misi v1.0.2 langkah P8, bukan di sini.

## 7. Kepatuhan gerbang governance

| Gerbang | Penerapan pada commit ini |
|---|---|
| G1 (tanpa dependensi baru) | Nol perubahan `Cargo.toml`/`Cargo.lock`; tidak ada tool yang di-install |
| G4 (gerbang direplikasi di tree yang berubah) | Commit ini docs/evidence-only, tetapi gerbang tetap **diukur ulang** di tip karena ia adalah dasar klaim closeout (§2) |
| G5 (CI merah = STOP) | Tidak ada gerbang yang merah; dua run atas tip akan diekstrak sebelum merge |
| G8 (append-only) | Nol amend/rebase/reset/force-push; 26 ekstrak lama tidak ditulis ulang; empat dokumen governance disunting dengan verifikasi subsekuen (§5); hash commit ini TIDAK dikutip di dalamnya |
| G10 (bahasa Indonesia) | Seluruh konten baru dokumen ini, berkas bukti, dan anotasi dokumen dalam bahasa Indonesia; identifier, nama berkas, dan kutipan log mentah tetap aslinya |
| Proteksi branch Opsi B | Tidak dilonggarkan sedikit pun; commit ini masuk `main` hanya lewat PR #4 |

## 8. UTANG §6 DITUTUP (commit `[PHASE-7-5b]`, langkah P0-sisa misi v1.0.2, 2026-10-21)

Bagian ini adalah operasi INSERT murni pada laporan yang sudah ter-commit; §1-§7 tidak ditulis ulang (G8 append-only).

Ketiga hal yang di §6 dinyatakan sebagai utang - bukan sebagai selesai - sekarang terukur:

1. **Hash merge commit.** `gh pr merge 4 --merge` menghasilkan `c35d09adcfe757f64a4c0016cc9f2d25cc16afff` dengan orang tua `6623dd83...` + `2906e414...` (terukur `git log --format=%H %P -1 c35d09a`). Dua-parent => bukan squash, bukan fast-forward; `feature/phase7` tidak dihapus. Bonus terukur: `git diff --stat 2906e41 c35d09a` keluar KOSONG, jadi tree tip main = tree tip branch yang menjadi dasar §2.
2. **Verifikasi proteksi pasca-merge.** GET `/repos/qomaruddindjamal/asl/branches/main/protection` sesudah merge (2026-10-20T23:40Z) identik dengan GET sebelum merge (2026-10-20T23:31Z) pada enam kunci yang ditegakkan: `strict=True`, `contexts=[test-and-build]`, `required_approving_review_count=0`, `enforce_admins=True`, `allow_force_pushes=False`, `allow_deletions=False` (`required_linear_history=False`). Proteksi Opsi B tidak dilonggarkan oleh merge, dan karena direct push ke `main` tertutup, bukti closeout ini masuk repo lewat PR dari branch `feature/v1.0.2-shell`.
3. **Ekstrak run hosted atas tip `main`.** Run `39515542430` (push @ `c35d09a`) hijau; dua run atas commit closeout `2906e41` (`39515248386` push, `39515252055` pull_request) ikut diekstrak supaya utang bukti tidak diwariskan ke commit berikutnya. Ketiganya terukur dari log mentah runner: 30 suite / 172 passed / 0 failed, 172 baris roster `test ... ok`, 21 baris warning profil test pada langkah [1/2], 13 baris `Compiling` + `Finished` pada langkah [2/2] dengan 0 warning / 0 error, ELF Linux 243.568 B, dan blok 13 baris llvm-cov yang hash-nya sama dengan blok pada sepuluh ekstrak ter-commit sebelumnya (konvensi hash dinyatakan di berkas bukti).

Rantai hosted kini **38 run success / 0 bukan-success** (23 `feature/phase7` + 10 `main` + 5 `security/asl-v1.0.2-audit`), berkas ekstrak ter-commit **28 -> 31**. Backlog yang masih tanpa ekstrak = **7 run** dan dihitung sebagai selisih himpunan, bukan diingat: `39400212750 @ 5a180f6`, `39417162899 @ cc3fe28`, `39417504070 @ cc3fe28` (branch audit) serta `39418646354 @ 45a33c7`, `39422495702 @ 5829e28`, `39423423307 @ bb4a9df`, `39436250243 @ 08f706a` (rantai main) - semuanya commit docs/evidence-only PHASE-6B. Dengan tertutupnya tiga butir di atas, **tidak ada lagi klaim T5 yang bergantung pada bukti yang belum ada**; yang tersisa hanyalah backlog yang dinyatakan terang.

Angka anti-drift untuk misi v1.0.2 diukur pada tree yang sama dan dicatat di `Analysis/reports/t5b_postmerge_measurements_2026-10-21.log` bagian [6]: occurrence `ASL-OS` pada `*.rs` = 6 (bukan 4 seperti catatan lisan - dua sisanya ada di `asl-cockpit-shell/src/lib.rs` baris 5 dan 19, yang kedua adalah konstanta `PROMPT_STRING` yang TIDAK dirujuk berkas mana pun), occurrence `v1.1.0` pada `*.rs` = 5 (baris 1126/1145/1150/1233/1822), tiga skrip `scripts/*.ps1` masih menunggu prompt lama. Angka inilah yang dipakai sebagai dasar langkah P2, bukan angka warisan.

Kelas bukti bagian ini: **VERIFIED-ON-CI** (butir 3 dan seluruh angka runner) + **VERIFIED-ON-HOST** (butir 1-2, `git diff`/`git log`, GET API). Tidak ada klaim VERIFIED-ON-HARDWARE (konter tetap 0) dan tidak ada klaim tangkapan layar. Jendela penerimaan G9 berikutnya tetap dijadwalkan pada misi v1.0.2 langkah P8 - commit ini tidak menjalankannya.
