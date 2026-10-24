# ARSIP TAG v1.1.0 - SEBELUM DIHAPUS (BAGIAN 7 MASTER PROMPT R3)

Tanggal pengukuran: 2026-10-24. Otorisasi: MASTER PROMPT R3 BAGIAN 7 butir
"Rilis lama v1.0.2 (dipotong prematur) dan v1.1.0 sesuai definisi P9 dokumen misi terbaru:
verifikasi keberadaannya terukur; arsip-dulu-hapus dengan pola T4R (manifes ter-commit:
sidik jari byte + salinan teks) bila arsip serupa belum ada; hapus; catat." Pemilik
bepergian (otorisasi tertulis 2026-10-24). Pola yang diikuti: dokumen arsip sebelumnya,
`Analysis/reports/T4R-ARSIP-RILIS-V1.0.2-2026-10-21.md`.

Nol berkas `.rs`, nol `Cargo.*`, nol berkas workflow pada komit ini.

## 1. Verifikasi keberadaan terukur (sebelum satu pun penghapusan)

| Objek | Lokal | Origin | Cara mengukur |
|---|---|---|---|
| tag `v1.1.0` | ADA (`git tag -l` memuat 4 tag; `v1.1.0` ada di daftar) | TIDAK ADA | `git tag -l` = v1.0.0, v1.0.1, v1.0.1-alfa, v1.1.0; `git ls-remote --tags origin` = v1.0.0, v1.0.0^{}, v1.0.1 |
| rilis GitHub `v1.1.0` | - | TIDAK ADA (`gh release view v1.1.0` -> `release not found`) | API hosted |
| tag `v1.0.2` | TIDAK ADA | TIDAK ADA | `git rev-parse -q --verify refs/tags/v1.0.2` (lokal) dan `git ls-remote --tags origin refs/tags/v1.0.2` (luaran 0 baris) |
| rilis GitHub `v1.0.2` | - | TIDAK ADA | `gh release list` mengembalikan 2 rilis: v1.0.1 (pre-release); v1.0.0 (pre-release) |

Akibat yang terukur: yang benar-benar ada untuk diarsipkan dan dihapus hanyalah TAG LOKAL
`v1.1.0`. Tidak ada rilis GitHub `v1.1.0`, tidak ada tag `v1.1.0` di origin, dan tidak ada
lagi artefak `v1.0.2` mana pun (rilis prematur `v1.0.2` sudah dihapus pada rantai R1 BAGIAN 0.2,
bukti arsipnya di `T4R-ARSIP-RILIS-V1.0.2-2026-10-21.md`). Angka 0 dan kata TIDAK ADA di atas
diukur, bukan disalin dari dokumen lama.

Yang TIDAK disentuh (di luar daftar BAGIAN 7; larangan butir e melarang menghapus tag di luar
daftar): `v1.0.0`, `v1.0.1` di origin beserta dua rilis pre-released-nya, dan tag lokal
`v1.0.1-alfa`. Terukur `git tag -l` tetap memuat tiga nama itu sesudah operasi ini (lihat bagian 6).

## 2. Sidik jari byte objek tag `v1.1.0`

| Bidang | Nilai terukur |
|---|---|
| Nama ref | `refs/tags/v1.1.0` |
| Tipe objek | `tag` (annotated tag object) |
| SHA objek tag | `90954ce008eb1e4774e792357ddcf528bc7f41a3` |
| Ukuran objek tag | 1595 B |
| SHA256 isi objek tag (mentah, `git cat-file tag`) | 2ecabbb2aeaf0eda287931de151801bdc7f459833599d6dd3b00fd2930c18018 |
| Komit yang dipeel (`^{}`) | `7104814e2ea1b3c45ff4b69c0fe70ed6d8c0423e` |
| Tree komit | `5335775eb042dba36de2f467ba6125af8471b635` |
| Tagger | `tagger qomaruddindjamal <jamalpancur@gmail.com> 1792504805 +0700` |
| Autor komit | qomaruddindjamal <jamalpancur@gmail.com>, Tue Oct 20 20:58:26 2026 +0700 |
| Subjek komit | `[PHASE-6B-C2][verify+koreksi] bukti hosted CI run kesepuluh + koreksi presisi waktu klaim stderr G9 (docs/evidence-only, nol .rs)` |
| Ukuran pesan tag (blok setelah header, CR diluruhkan) | 1458 B |
| SHA256 pesan tag | `040d6c79b1bcbde1f6d57af3e17c063f1b4e4c710b37c14a8d56e0066e36799c` |
| non-ASCII dalam pesan tag | 0 byte |

## 3. Salinan penuh pesan tag `v1.1.0` (verbatum, 1458 B SHA256 040d6c79b1bcbde1f6d57af3e17c063f1b4e4c710b37c14a8d56e0066e36799c)

```
ASL Core Kernel v1.1.0 (2026-10-20)

Riset misi remediasi PHASE-1..6B + PHASE-6B-CLOSEOUT T0..T3.
Provenance: merge commit c56c69a (PR #3, parent 14bc022 + cc3fe28) sebagai basis
kode; rantai docs-only setelahnya: 45a33c7 -> 5829e28 -> bb4a9df -> e78b186 ->
7104814 (tip tag ini). `git diff --name-only cc3fe28 7104814` memuat NOL berkas
.rs (15 file, +631/-16, semuanya dokumen/ekstrak log/harness PowerShell), jadi
input kernel pada tag ini identik dengan tree yang diuji di GERBANG AKHIR.

Angka teruji (artefak ter-commit, bukan asumsi): 30 suite host test
172 passed / 0 failed; build x86_64-unknown-none release 0 error / 0 warning;
ELF 243.568 B (Windows, biner yang di-boot) / 243.560 B (Linux runner);
ISO 4.118.528 B SHA256 faca92f4e1e92b2ee5eb8d2e931c0b1a64df9c2035a308529e69d4e5128c8355;
49 gerbang governance (24 Phase_01..06 PASS locked, 25 Milestone_03 unlocked).

Kategori bukti: VERIFIED (host test), VERIFIED-ON-VM (QEMU q35 serial + run
penerimaan G9 BERJENDELA 2026-10-20 20:48 capai M6 + prompt ASL-OS>),
VERIFIED-ON-CI untuk langkah [1]-[2] pipeline saja (10 run hijau, angka dari log
mentah). VERIFIED-ON-HARDWARE = 0 item.

Keterbatasan: ISO tidak byte-reproducible (volume timestamp xorriso); banner
console kernel masih tertulis v1.0.2 (string hardcoded di lib.rs, perubahan
menuntut .rs); hosted CI tidak menjalankan langkah QEMU; tangga DO-178C DAL-C
belum terpenuhi (MC/DC, tim verifikasi independen, Tool Qualification DO-330).
```

## 4. Artefak ter-commit yang dirujuk pesan tag (tetap di repo, jadi tidak ada yang hilang)

| Berkas | B | baris | SHA256 | terlacak git | blob | komit terakhir yang mengubah | non-ASCII |
|---|---|---|---|---|---|---|---|
| `Analysis/reports/iso_build_v1.1.0_2026-10-20.log` | 3500 | 55 | `ac383cfa3cdaf10d82340130887e5cfb7f111ab10c5ef5b021da9f83931ad5f4` | YA | `b507c5ea7c43df88620e5452b80e51161573fa60` | `e78b186` | 0 |
| `Analysis/reports/release_v1.1.0_SHA256SUMS_2026-10-20.txt` | 1392 | 14 | `df13c3b8e414e9e4e83704ebe2eb8bafcd9708c612e526f6c80c0236bc9f78c8` | YA | `e286ce6f5b33a2780d9ba4d0856fb3f0f40da0ae` | `08f706a` | 0 |
| `Analysis/reports/release_v1.1.0_evidence_2026-10-20.log` | 3099 | 44 | `5933d231110d1172160383c93db9cae671d5b5e617942ad9bc1c3f353f3cb449` | YA | `167fd42e5e49ec6c4f3f51956c48444117414bfe` | `08f706a` | 0 |

Ketiga berkas itu ada di `Analysis/reports/` dan terlacak git, jadi menghapus tag tidak
menghilangkan buktinya. Yang TIDAK bisa dipulihkan dari repo dijelaskan di bagian 5.

## 5. Batasan jujur: apa yang benar-benar berhenti ada

- Pesan tag mencantumkan ISO v1.1.0 `faca92f4e1e92b2ee5eb8d2e931c0b1a64df9c2035a308529e69d4e5128c8355` berukuran 4.118.528 B. Biner ISO itu TIDAK pernah di-commit
  (direktori tempatnya diarsipkan, `logs/`, diabaikan `.gitignore` baris 159 `**/logs/`),
  dan rilis GitHub-nya sudah dihapus lebih dulu, jadi **sidik jarinya tidak dapat diverifikasi
  ulang dengan membuka ulang apa pun**.
- Terukur: dari 29 berkas `.iso` yang tersisa di `logs/`, 0 di antaranya ber-SHA256 sama dengan
  `faca92f4e1e92b2ee5eb8d2e931c0b1a64df9c2035a308529e69d4e5128c8355`. Jadi biner ISO v1.1.0 memang tidak ada lagi di disk host ini. Yang tersisa hanyalah
  berkas ISO lain dengan fingerprint berbeda (di antaranya `t3_iso_build1.iso` 4118528 B `f7dd6ecace8d...`, `t5_iso_a.iso` 4118528 B `da5b91f95cd1...`, `t5_iso_b.iso` 4118528 B `da5b91f95cd1...`), dan itu TIDAK diklaim sebagai
  ISO v1.1.0.
- Yang TIDAK hilang tidak ada kaitannya dengan kode: seluruh tree komit `7104814e2ea1b3c45ff4b69c0fe70ed6d8c0423e` dan riwayatnya
  tetap ada (`git log` tidak dihapus), jadi kernel/banner/dokumen pada titik itu dapat diperiksa
  dan dibangun ulang. Build ulang hari ini menghasilkan ISO master v1.0.2 `1569c17157b98c8aa29c9e9d18ccb6aeb931c883d1e1a25983864df4206c668c` (4624384 B, BAGIAN 5),
  bukan byte v1.1.0, dan itu dinyatakan apa adanya.
- Alasan penghapusan sesuai mandat: v1.1.0 adalah nomor versi yang dipotong prematur pada rantai
  lama dan digantikan oleh definisi rilis v1.0.2 pada dokumen misi terbaru (P9). Riwayatnya
  tidak dibatalkan, hanya tidak lagi disoroti sebagai rilis.

## 6. Operasi yang dijalankan sesudah manifes ini ditulis (urut dan terukur)

1. Manifes ini ditulis dan diverifikasi ada di disk SEBELUM perintah hapus apa pun dijalankan.
2. `git tag -d v1.1.0` (hapus tag LOKAL saja; tidak ada `git push --delete` karena origin TIDAK
   punya tag ini - lihat tabel bagian 1; nol force-push, nol amend).
3. Verifikasi ulang: `git tag -l` harus tetap memuat `v1.0.0`, `v1.0.1`, `v1.0.1-alfa` dan tidak
   lagi memuat `v1.1.0`; `git ls-remote --tags origin` harus tetap mengembalikan hasil yang sama
   seperti tabel bagian 1 (nol perubahan di origin oleh operasi ini).
4. Tag ANNOTATED `v1.0.2` dipotong di tip yang lulus BAGIAN 6, didorong, lalu rilis GitHub dibuat
   dengan aset ISO + SHA256SUMS.txt dan diverifikasi lewat API - semuanya di berkas bukti
   `Analysis/reports/r3_bagian7-merge-tag-rilis_2026-10-24.txt` dan dicatat di
   `docs/ROADMAP/ACCOUNTABILITY_FAULT_LOG.json` serta `docs/ROADMAP/HANDOFF_STATE.json`.

## 7. Higiene dokumen ini

Ditulis oleh `logs/r3_b7_arsip.py`; setiap byte dan SHA256 di atas diambil dari `git`, `gh`,
atau `hashlib` pada saat yang sama, bukan disalin dari dokumen lama.
