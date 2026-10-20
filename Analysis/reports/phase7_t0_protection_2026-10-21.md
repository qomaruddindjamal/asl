# PHASE-7 T0 — proteksi "Opsi B" + uji pipeline PR + rekonsiliasi hitungan run CI

Tanggal: 2026-10-21 (02:3x +0700) — nama berkas memakai tanggal **lokal**; stempel di `ACCOUNTABILITY_FAULT_LOG.json` dan `HANDOFF_STATE.json` memakai **UTC** sehingga masih terbaca `2026-10-20T19:3xZ`. Branch: `feature/phase7` (base `6623dd8`). Otorisasi: prompt misi PHASE-7 dari pemilik repo — "Owner menyetujui Opsi B → eksekusi", "SEMUA perubahan main WAJIB lewat PR (proteksi aktif)", "ALUR: branch feature/phase7, commit [PHASE-7-0..5], SATU PR (#4), CI hijau, merge".

## 1. Keputusan governance: konfigurasi proteksi `main` diubah dari CLOSEOUT T6 ke Opsi B

Konfigurasi pasca-CLOSEOUT T6 (dipasang 2026-10-20) memakai `requires/required_approving_review_count=1`. Karena repo hanya punya **satu** akun GitHub (`qomaruddindjamal`, sekaligus penulis commit), aturan itu secara struktural memblokir seluruh perubahan `main` berikutnya: penulis tidak dapat menyetujui PR sendiri, dan `enforce_admins=true` menutup jalur pintas admin. Deadlock ini sudah dinyatakan di `Analysis/reports/branch_protection_main_2026-10-20.log` §6 dan §7 butir 8 `QODER_FINAL_REPORT.md`, dan pemilik meresponsnya dengan Opsi B.

Yang **tetap** ditegakkan: PR wajib, status check `test-and-build` wajib hijau, branch harus up-to-date, anti force-push, anti penghapusan branch, aturan berlaku untuk admin. Yang **berubah**: jumlah approval minimum menjadi 0 dan review stale tidak auto-dismiss. Tidak ada perubahan `.rs` dan tidak ada gerbang governance yang di-lock/unlock (G2 tidak terbentuk; `CHECKLIST_STATE.json` tidak disentuh).

## 2. Payload PUT yang dikirim (persis)

```json
{
  "required_status_checks": {"strict": true, "contexts": ["test-and-build"]},
  "required_pull_request_reviews": {"dismiss_stale_reviews": false, "require_codeowners_reviews": false,
                                     "required_approving_review_count": 0, "restricts_reviews": false},
  "enforce_admins": true,
  "allow_force_pushes": false,
  "allow_deletions": false,
  "required_linear_history": false,
  "required_signatures": false,
  "block_creations": false,
  "required_conversation_resolution": false,
  "restrictions": null
}
```

Dikirim dengan `gh api -X PUT repos/qomaruddindjamal/asl/branches/main/protection --input logs/protection_payload_opsib.json` (keluaran PUT disimpan di `logs/protection_opsib_resp.json`, respons GET di `logs/protection_get_opsib.json`).

Fakta API yang tercatat supaya tidak ditebak lagi di fase berikutnya: `PUT protection` **menolak** tanpa field `restrictions` (HTTP 422 `"restrictions" wasn't supplied`), jadi `restrictions: null` wajib dikirim; kunci karangan (`admin_enforcement`, `restrict_push`) diterima lalu diabaikan diam-diam; nama field respons yang kanonik `required_approving_review_count` dan `require_code_owner_reviews` (bukan `require_codeowners_reviews` seperti pada sisi PUT).

## 3. Verifikasi GET balik — field per field (bukti struktural, bukan tampilan UI)

| field | GET | PUT yang dimaksudkan | status |
|---|---|---|---|
| `required_status_checks.strict` | `true` | `true` | cocok |
| `required_status_checks.contexts` | `["test-and-build"]` | `["test-and-build"]` | cocok |
| `required_status_checks.checks[].app_id` | `15368` (github-actions) | — (dinormalisasi GitHub) | cocok |
| `required_pull_request_reviews.required_approving_review_count` | `0` | `0` | cocok |
| `required_pull_request_reviews.dismiss_stale_reviews` | `false` | `false` | cocok |
| `required_pull_request_reviews.require_code_owner_reviews` | `false` | `false` | cocok |
| `required_pull_request_reviews.require_last_push_approval` | `false` | default | cocok |
| `enforce_admins.enabled` | `true` | `true` | cocok |
| `allow_force_pushes.enabled` | `false` | `false` | cocok |
| `allow_deletions.enabled` | `false` | `false` | cocok |
| `required_linear_history.enabled` | `false` | `false` | cocok |
| `required_signatures.enabled` | `false` | `false` | cocok |
| `block_creations.enabled` | `false` | `false` | cocok |
| `required_conversation_resolution.enabled` | `false` | `false` | cocok |
| `restrictions` | tidak dikembalikan (null) | `null` | cocok |

Kesimpulan: **direct push ke `main` masih ditolak GitHub** (PR tetap wajib + check tetap wajib), tetapi merge PR tidak lagi bergantung pada approval akun kedua.

## 4. Uji pipeline baru (PR pertama fase ini = PR #4)

`feature/phase7` dibuat dari `6623dd8` (bukan branch misi lama `security/asl-v1.0.2-audit`, yang dipertahankan sebagai rantai bukti PHASE-1..6B). Workflow `asl-ci-minimal` punya pemicu `push: branches ["**"]` + `pull_request`, jadi branch fitur ini ikut ter-CI sebelum PR dibuka. Tidak ada langkah yang mengklaim [3]–[4] di hosted; gerbang lokal `scripts/ci_local.ps1` tetap dijalankan terpisah dan **tidak** di-overwrite ke artefak yang sudah ter-commit (nama output baru).

## 5. Rekonsiliasi hitungan run CI: satu angka final = 14 di `main` (+1 run branch fitur = 15 total)

Dokumen pra-PHASE-7 menyebut angka berbeda karena diukur pada waktu berbeda: "enam run" (PHASE-6B), "sebelas run" (RELEASE_NOTES §0.1 pasca-T4), "13 run" (laporan CLOSEOUT). Pengukuran hari ini dengan `gh run list --workflow asl-ci-minimal --limit 40 --json databaseId,headSha,conclusion,event`: **14 run `asl-ci-minimal` dengan `conclusion=success`** pada rantai `8b829d8` → `6623dd8`, daftar lengkap ada di `Analysis/reports/ci_hosted_39444157361_2026-10-20.log` baris rantai. Entri `39417510019` @ `cc3fe28` dan sembilan run 2026-10-17 **dikecualikan** karena bukan workflow `asl-ci-minimal` (event `dynamic`/suite lain) — sebelumnya sempat dihitung sebagai "run hosted", itu sumber selisih angka.

### 5.1 Tingkat bukti: 15 run = 8 + 5 + 2 (audit ulang hari ini, bukan salin kalimat lama)

**Tier 1 — log mentah ditarik DAN ekstrak ter-commit di repo (8 run: 7 pada rantai `main` + 1 pada branch fitur).** Inilah tingkat bukti terkuat karena angkanya bisa dibaca ulang dari artefak yang ter-track:
`39399083819` @ `8b829d8`, `39400350619` @ `fff0975`, `39417746328` @ `c56c69a`, `39434189599` @ `e78b186`, `39435093922` @ `7104814` (run atas commit bertag), `39437444616` @ `3c716d4`, `39444157361` @ `6623dd8` (ekstrak yang commit PHASE-7-0 masukkan), `39486017207` @ `7e2ac24` (run atas branch `feature/phase7` — bukti pertama yang masuk repo lewat jalur PR, jadi ia sekaligus uji fungsional pipeline baru).

**Tier 2 — log mentah ditarik dan diukur pada sesinya, tetapi ekstrak TIDAK pernah ter-commit (5 run).** `39417162899` @ `cc3fe28` (push), `39417504070` @ `cc3fe28` (`pull_request` PR #3), `39418646354` @ `45a33c7`, `39422495702` @ `5829e28`, `39423423307` @ `bb4a9df`. Berkas mentahnya masih ada di `logs/` — direktori yang di-ignore `.gitignore:159 **/logs/`, jadi **tidak dapat diaudit dari repo**. Pemetaan berkas→run dilakukan struktural (kolom langkah `[2/2]`/`Bukti footprint ELF hasil build` + stempel waktu UTC di dalam berkas vs `createdAt` API), lalu angka diukur ulang hari ini dengan parser yang dipakai untuk ekstrak:

| berkas mentah lokal | run | commit | sha256 berkas | baris | suite | passed/failed | `Compiling` di `[2/2]` | warn/err di `[2/2]` | ELF |
|---|---|---|---|---|---|---|---|---|---|
| `logs/ci_hosted_raw3.log` (tier 1) | 39400350619 | `fff0975` | `9b41d77420262a8a38accb0b132157957fa52a31ebb28d746ea617658e6fcc7d` | 841 | 30 | 172 / 0 | kolom langkah = `UNKNOWN STEP`, attribusi per langkah tidak tersedia | — | 243560 |
| `logs/ci_hosted_raw4.log` | 39417162899 | `cc3fe28` push | `208a938608ee1200cfba6aa87c863b667166e478c4903c2080bc6cb799f9631a` | 841 | 30 | 172 / 0 | 13 | 0 / 0 | 243560 |
| `logs/ci_hosted_raw_pr.log` | 39417504070 | `cc3fe28` pull_request | `3dc8faa22002d8db31d19589ce75033c62d4756d52ff1f64a48a888443ce9170` | 852 | 30 | 172 / 0 | 13 | 0 / 0 | 243560 |
| `logs/ci_hosted_raw_main.log` (tier 1) | 39417746328 | `c56c69a` | `698686cdfca6b3547ef45abc215c5193a2b2d1014d090edfc1de7f7fb5a117c9` | 841 | 30 | 172 / 0 | 13 | 0 / 0 | 243560 |
| `logs/ci_hosted_raw7.log` | 39418646354 | `45a33c7` | `69162a233ef9347b6184cc640f84f811724792e74b94a04a2fe0089af7893cb4` | 841 | 30 | 172 / 0 | 13 | 0 / 0 | 243560 |
| `logs/raw_t0.log` | 39422495702 | `5829e28` | `f4f0eb52acddbffd7115db90ca9c09cb1dd154244a7867d97053fb77b5669582` | 841 | 30 | 172 / 0 | 13 | 0 / 0 | 243560 |
| `logs/raw_c1.log` | 39423423307 | `bb4a9df` | `23f2d173b2a62e7371d583c88e3f5e042e798076b7e64577e57e3e6224d1a218` | 841 | 30 | 172 / 0 | 13 | 0 / 0 | 243560 |

Waktu mtime `logs/raw_t0.log` = 2026-10-20 19:29 dan `logs/raw_c1.log` = 19:36 (+0700), yaitu 19 dan 60 detik setelah `createdAt` run masing-masing (12:28:17Z dan 12:35:18Z) — jadi tarikan log mentah terjadi pada sesi yang sama dengan commit yang mengklaimnya, bukan rekonstruksi setelahnya.

**Tier 3 — hanya metadata API `conclusion=success` + `headSha`, log mentah tidak pernah ditarik (2 run).** `39400212750` @ `5a180f6`, `39436250243` @ `08f706a`.

**Rekonsiliasi setelah run branch fitur masuk (`gh run list --workflow asl-ci-minimal --limit 40 --json databaseId,headBranch,headSha,createdAt,conclusion` diukur ulang 2026-10-21 02:47 +0700):** total run `asl-ci-minimal` `conclusion=success` yang pernah ada = **15** = 14 pada rantai `main` `8b829d8` -> `6623dd8` + 1 pada `feature/phase7` (`39486017207` @ `7e2ac24`, `createdAt 2026-10-20T19:44:28Z`, `updatedAt 19:44:42Z`). Angka **14 tetap dipakai sebagai hitungan rantai `main`** — angka yang selama ini dikutip dokumen — dan selisihnya selalu disebut sebagai run branch fitur, supaya rekonsiliasi hari ini tidak mengulang sumber kebingungan yang sama (campur-campur cakupan, bukan beda waktu pengukuran).

Angka yang dihitung ulang dari teks log pada tier 1 dan tier 2 identik: 30 suite **172 passed / 0 failed**, langkah release 13 baris `Compiling` + `Finished`, **0 `warning:` / 0 `error:`** pada langkah release (21 baris warning milik profil test, non-blocking), ELF Linux **243.560 B**.

### 5.2 Koreksi append-only dua arah atas pernyataan tingkat bukti

* `6623dd8` (dan dokumen yang disinkronkannya) menggolongkan **7** run sebagai "hanya dari metadata API". Terukur hari ini: **5** di antaranya punya log mentah yang benar-benar ditarik dan diukur (tier 2), sehingga penyataan itu **terlalu lemah** untuk kelima run tersebut. Yang benar-benar tanpa tarikan log tinggal **2** (tier 3).
* Arah sebaliknya tetap berlaku: klaim `bb4a9df`/`45a33c7` bahwa run "diverifikasi dari log mentah" tidak disertai ekstrak ter-commit, sehingga pihak ketiga tidak bisa mengauditnya dari repo — kelemahan prosedural yang kini tertutup untuk run `39444157361` lewat §6, dan yang tercatat sebagai alasan PHASE-7 mensyaratkan ekstrak ter-commit untuk run baru.
* Yang berubah akibat koreksi ini **hanya penempatan run pada tingkat bukti**, bukan angka test/build/footprint (172/0, release 0/0, ELF 243.560 B — sama di semua run yang teksnya terukur). Tidak ada amend/rebase/reset; koreksi dilakukan di commit baru yang menyebut hash `6623dd8` (G8). Ratifikasi tercatat di fault log entri #13.

## 6. Ekstrak CI baru yang masuk repo lewat PR ini

`Analysis/reports/ci_hosted_39444157361_2026-10-20.log` (67 baris ASCII) — run `push` atas commit `6623dd8` (tip `main` pasca-CLOSEOUT T6), `createdAt 2026-10-20T14:58:41Z`, `updatedAt 14:59:05Z`, job `test-and-build`. Berkas ini tadinya hanya bisa hidup di `logs/` (ter-ignore) karena direct push sudah tertutup; Opsi B + pipeline PR menutup celah penelusuran itu.

**Susulan (bagian dari commit PHASE-7-0c):** `Analysis/reports/ci_hosted_39486017207_2026-10-20.log` (67 baris ASCII) = ekstrak log mentah run `39486017207` atas tip branch `7e2ac24`, diukur dari teks: 30 suite **172 passed / 0 failed**, 13 baris `Compiling` + `Finished release profile [optimized] target(s) in 2.45s`, **0 `warning:` / 0 `error:`** pada langkah release (21 baris warning milik profil test, non-blocking), ELF Linux **243.560 B**. Ini pola yang dinyatakan di §4: ekstrak run atas commit N masuk repo lewat commit N+1, bukan dengan amend commit N (G8); run atas tip terakhir PR akan diekstrak-ter-commit pada closeout T5.

## 7. Higiene scratch `Analysis/reports/`

Lihat `Analysis/reports/phase7_t0_hygiene_2026-10-21.md`: 13 berkas `.log` tak-ter-track diklasifikasi jadi 4 kelas dengan bukti (`cmp` byte-identik untuk 7 berkas, containment baris-per-baris terhadap korpus ter-commit, manifest SHA256 sebelum tindakan). 9 dihapus, 4 disahkan sebagai baseline PHASE-7 (termasuk inventaris nama test pra-T3 dan transkrip xorriso pra-T1). Nol berkas ter-track dihapus atau diubah isinya.

## 8. Sinkron dokumen yang dilakukan commit PHASE-7-0 ini

Empat pernyataan yang kini **kedaluwarsa secara faktual** dikoreksi append-only (teks lama tetap terbaca sebagai yang dikoreksi, tidak dihapus diam-diam):

| berkas | butir | dari | menjadi |
|---|---|---|---|
| `README.md` §6 | bukti hosted | tigabelas run hijau, dua tingkat (6 ekstrak / 7 metadata) | empat belas run, **tiga** tingkat (7 ekstrak ter-commit / 5 log mentah tanpa ekstrak ter-commit / 2 hanya metadata) + kalimat koreksi atas penggolongan `6623dd8` |
| `README.md` §7 (baris "Status CLOSEOUT") | konfigurasi proteksi | "PR + 1 approval" | catatan bahwa keputusan pemilik 2026-10-21 menggantikannya dengan Opsi B (`required_approving_review_count=0`), dengan alasan single-account |
| `RELEASE_NOTES.md` §0.1 + §0.4 + baris tabel build | jumlah run + tingkat bukti + approvals | 13 run / 6 ekstrak / 7 API; "1 approval" | 14 run / 7 / 5 / 2; annotasi penggantian ke Opsi B + rujukan laporan T0 |
| `QODER_FINAL_REPORT.md` §4 (blockquote), §2, §7 butir 5 & 8, §8 baris T6 + verdict | jumlah run, tingkat bukti, syarat approval, status deadlock | seperti di atas | seperti di atas + entri fault log #13/#14 |
| `docs/ROADMAP/HANDOFF_STATE.json` | `branch_protection`, `phase6b_closeout_status`, `phase7_entry_gate` | 1 approval, 13 run, "1 approval" sebagai syarat masuk PHASE-7 | Opsi B terukur, 14 run, syarat PR + check hijau tanpa approval akun kedua + kunci baru `phase7_t0_state` |
| `docs/ROADMAP/ACCOUNTABILITY_FAULT_LOG.json` | `fault_history` | 12 entri | 14 entri (+#13 governance Opsi B CLOSED, +#14 koreksi tingkat bukti ACCEPTED_WITH_WRITTEN_JUSTIFICATION); `violations_recorded` tetap **0** |

`CHECKLIST_STATE.json` **tidak disentuh** (nol gerbang berubah status, G2 tidak terbentuk). `QODER_EXECUTION_LOG.md` tetap CLOSED untuk misi PHASE-1..6B — eksekusi PHASE-7 tercatat di laporan fase + fault log, bukan dengan membuka kembali log yang sudah ditutup resmi.
