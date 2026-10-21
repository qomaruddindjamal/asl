# ARSIP ASET RILIS v1.0.2 — SEBELUM PENGGANTIAN (BAGIAN 0.2)

Dokumen ini adalah bukti langkah pertama BAGIAN 0.2 pada MASTER PROMPT R1-REVISI
(keputusan pemilik 2026-10-21): rilis `v1.0.2` dan tag remote-nya akan dihapus
karena dipotong terlalu dini, sebelum perbaikan E8 (kernel panic) dan F1-F7
(perbaikan shell) selesai. Penghapusan artefak publik tidak boleh berjalan tanpa
jejak, jadi seluruh isi artefak diarsipkan lebih dulu dan isi yang perlu
direkonstruksi ditulis utuh di sini.

Nol berkas `.rs`, nol `Cargo.*`, nol berkas workflow pada commit ini.

## 1. Lokasi arsip dan mengapa manifes ini yang di-commit

Direktori arsip: `logs/t4r_arsip/` (pola yang sama dengan arsip rilis
sebelumnya). Direktori itu DIABAIKAN oleh git — terukur `git check-ignore -v`
menunjuk aturan baris 159 `.gitignore`: `**/logs/`. Jadi berkas biner 4,2 MB
tidak bisa masuk riwayat repo dari lokasi itu, dan saya tidak mengubah
`.gitignore` (di luar cakupan yang diizinkan perintah ini).

Konsekuensinya dinyatakan terang: yang di-commit adalah manifes ini, berisi
sidik jari byte + salinan penuh setiap berkas teks artefak. ISO-nya sendiri
tidak di-commit; ia dapat dibangun ulang byte-identik dari tip `main`
(bukti reproduksinya ada di bagian 4), jadi tidak ada informasi yang hilang
untuk selamanya.

## 2. Isi arsip yang terukur (sebelum penghapusan)

| Berkas | Ukuran byte | SHA256 terukur lokal | SHA256 dihitung ulang oleh pihak hosted | Cocok |
|---|---|---|---|---|
| `asl-core-kernel-v1.0.2.iso` | 4.200.448 | `aa77806018d7a9009bb705e29e71d33724a6b50fc3a5cf62de24133359c74de9` | `aa778060…9c74de9` | ya |
| `SHA256SUMS.txt` | 1.287 | `d679e75ce4d3560224c463bdb7db565d37ee9fd36532ad41ba0e39fbc48be40a` | `d679e75c…be40a` | ya |
| `release_meta_v1.0.2.json` (salinan API) | 7.494 | `c546a6cee184d7f5900939c65e978d3d73f07dae9d0369c4856feabcedc705d5` | — (bukan aset rilis) | — |
| `tag_ref_v1.0.2.json` (salinan API) | 335 | `78578f74f782c881a6788f2c2afa162e75b45de3edb20373dc7800ff8ebeb321` | — (bukan aset rilis) | — |

Kolom "dihitung ulang oleh pihak hosted" diambil dari `digest` aset pada API
sistem hosted, yaitu hash yang dikompute ulang di sisi server setelah unggahan —
konfirmasi independen atas angka di host build.

**Koreksi angka terhadap teks perintah.** Perintah BAGIAN 0.2 menulis
"arsipkan 5 aset rilis v1.0.2". Terukur: rilis `v1.0.2` punya **2** aset
(`gh release view v1.0.2 --json assets` dan `gh api …/releases/tags/v1.0.2`
sama-sama mengembalikan dua entri). Angka 5 adalah jumlah aset pada rilis
sebelumnya yang saya arsipkan pada tahap T4 rantai lama. Yang diarsipkan di
sini adalah semua yang benar-benar ada pada rilis ini, tidak lebih dan tidak
kurang.

## 3. Keadaan rilis dan tag yang akan dihapus (terukur)

- Nama tag: `v1.0.2`; URL https://github.com/qomaruddindjamal/asl/releases/tag/v1.0.2
- Judul rilis: `ASL Core Kernel v1.0.2: shell interaktif Cockpit + identitas rilis + dokumentasi PANDUAN`
- Id rilis (sistem hosted): `419458288`, node `RE_kwDOVGvOuM4ZAGzw`
- `draft=false`, `prerelease=false`, dibuat 2026-10-21T11:07:15Z, diterbitkan 2026-10-21T11:08:01Z
- `target_commitish` = `b47c24c1e2ac73c7e495c317d32cd01892e7d247` (merge commit dua-parent PR #6)
- Id aset: ISO `662228852`, `SHA256SUMS.txt` `662228855`; keduanya `state=uploaded`
- Tag annotated: objek `fcdddfd216aebf81a02229e989010c69b4aa9031` (tipe `tag`) → peel commit `b47c24c1…`
- Yang **TIDAK** disentuh: merge commit `b47c24c` dan seluruh riwayatnya; tag `v1.0.0` dan `v1.0.1-alfa` beserta rilis pre-release-nya; berkas SHA256SUMS rilis lama yang sudah ter-commit di repo.

## 4. Rekonstrusi byte-identik (bukti bahwa menghapus aset tidak menghilangkan karya)

ISO ini sudah terbukti dua kali byte-identik bila dibangun dari tree yang sama
pada host build, dengan `RUSTFLAGS` tidak ter-set:

| Tree sumber | Ukuran | SHA256 |
|---|---|---|
| tip cabang saat jendela P8 (`75aaf93`) | 4.200.448 | `aa778060…9c74de9` |
| tip `main` pasca-merge (`b47c24c`) | 4.200.448 | `aa778060…9c74de9` |

Tree `.rs` kedua tip itu identik (delta `git diff --name-only` memuat nol
berkas `.rs`), dan reproduksibilitas ISO berlaku per-lingkungan kompilasi sejak
PHASE-7 T1 (tanggal tetap perkakas pengemas + GUID cakram GPT diturunkan dari
epoch, bukan acak). ELF di dalam ISO: `boot/asl-kernel.elf` 325.528 byte,
SHA256 `9203864ea6feae5bd3d51abd0cd3dc69ddcec0b8e91cd5401b1c63074a27b216`.

## 5. Salinan penuh isi teks artefak

### 5.1 `SHA256SUMS.txt` (1.287 byte, SHA256 `d679e75c…be40a`)

```
# ASL Core Kernel v1.0.2 - checksum aset rilis (2026-10-21)
# Diukur pada host build; diverifikasi ulang dengan `sha256sum -c SHA256SUMS.txt` di direktori ini.
# Tag annotated v1.0.2 -> commit b47c24c1e2ac73c7e495c317d32cd01892e7d247 (tip main pasca-merge PR #6;
#   merge commit dua-parent, tanpa squash dan tanpa rebase).
# Prinsip yang ditegakkan: ISO yang dirilis = ISO yang di-boot pada jendela penerimaan P8. Build ulang
#   ISO pada tip main menghasilkan SHA256 yang IDENTIK dengan ISO_P8 (terukur pada dua invokasi terpisah).
# ISO ini byte-reproducible per-platform sejak PHASE-7 T1 (pin tanggal xorriso + GPT Disk GUID dari epoch).
aa77806018d7a9009bb705e29e71d33724a6b50fc3a5cf62de24133359c74de9 *asl-core-kernel-v1.0.2.iso
# Ukuran: ISO 4200448 B.
# ELF di dalam ISO (boot/asl-kernel.elf): 325528 B,
#   SHA256 9203864ea6feae5bd3d51abd0cd3dc69ddcec0b8e91cd5401b1c63074a27b216.
# Bukti boot (VERIFIED-ON-VM-VISIBLE, bukan perangkat keras): Analysis/reports/P8-BUKTI-JENDELA-2026-10-21.md
#   plus artefak bercap-waktu 17-51-59 di Analysis/reports/; kode keluar jendela penerimaan terukur 0.
# Deviasi yang diterima pemilik secara sadar: janji lantai memori boot `-m 3M` masih MERAH
#   (ACCOUNTABILITY_FAULT_LOG entri ke-23); rilis ini diterbitkan di bawah keputusan pemilik.
```

### 5.2 Pesan tag annotated `v1.0.2` (675 byte, SHA256 `09324a4ca8572fa2ce8e13c10d6796b08e96e8b3fdcb688631a5ee518cea71c5`)

```
ASL Core Kernel v1.0.2

Shell interaktif Cockpit + identitas rilis v1.0.2 + dokumentasi PANDUAN.
Basis: merge commit b47c24c1e2ac73c7e495c317d32cd01892e7d247 pada branch main
(PR #6, merge commit dua-parent, tanpa squash dan tanpa rebase).

Artefak rilis: asl-core-kernel-v1.0.2.iso, 4.200.448 byte, SHA256
aa77806018d7a9009bb705e29e71d33724a6b50fc3a5cf62de24133359c74de9 - identik dengan
ISO yang di-boot pada jendela penerimaan P8.

Status bukti: boot terverifikasi pada mesin virtual q35 (VERIFIED-ON-VM-VISIBLE),
bukan pada perangkat keras. Janji lantai memori boot `-m 3M` masih merah dan
deviasinya diterima pemilik secara sadar (ACCOUNTABILITY_FAULT_LOG entri ke-23).
```

Catatan untuk pemotongan tag ulang di R2: berkas sumber pesan tag dan berkas
catatan rilis masih ada di direktori kerja `logs/t4_release/` (SHA256 catatan
rilis `74661279…ddb2c`, 2.345 byte — identik dengan badan rilis yang
terukur lewat API). Keduanya akan ditulis ulang isinya setelah fitur shell
beku, bukan disalin apa adanya, karena nomor versi tampilan dan daftar
perintah akan berubah oleh F1-F7.

## 6. Yang terjadi setelah dokumen ini

Urutan berikutnya, atas perintah yang sama: hapus rilis `v1.0.2`, hapus tag
remote `v1.0.2`, hapus tag lokal `v1.0.2`, lalu daftarkan seluruh operasi
tersebut sebagai ACCOUNTABILITY_FAULT_LOG entri ke-30 pada commit
`[V1.0.2-CO-T4R] revert tag v1.0.2`. Tag dan rilis akan dipotong ulang pada
rantai R2 di tip final, dengan jendela penerimaan yang disaksikan pemilik.
Menahannya bukan membatalkannya: riwayat kerja rantai lama tetap sah dan
tercatat.
