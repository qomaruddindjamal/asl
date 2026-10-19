# GATE REGRESSION POLICY
**Status:** RATIFIED 2026-10-20 | **Dokumen Induk:** `docs/CANONICAL/CM_PLAN.md`, `docs/ROADMAP/CHECKLIST_STATE.json`

## 1. Prinsip
- Gate berstatus `locked: true` **tidak boleh** diam-diam dibuka atau diubah statusnya.
- Gate terkunci **BOLEH dan WAJIB di-re-verify** ketika: (a) kode yang dibuktikannya berubah; (b) ada laporan regresi; (c) audit independen meminta pembuktian ulang; (d) siklus review berkala 90 hari.

## 2. Prosedur Re-Verifikasi
1. Jalankan gerbang standar:
   - `cargo test --workspace --lib` (host; saat baseline 2026-10-20: 128 PASS)
   - `cargo build --package asl-kernel-bin --target x86_64-unknown-none --release`
   - Bukti runtime terkait gate (log QEMU/VirtualBox/serial) bila gate bersifat RUNTIME VERIFIED.
2. Simpan artefak bukti di `Analysis/reports/` sesuai CM_PLAN.
3. Catat hasil pada `docs/ROADMAP/ACCOUNTABILITY_FAULT_LOG.json` dengan skema entri:
   `{ "date": "...", "actor": "...", "type": "GATE_REVERIFICATION", "gate_id": "...", "result": "PASS|FAIL|PARTIAL", "evidence": ["path..."], "notes": "..." }`
4. Jika hasil re-check **tetap valid** → tidak ada perubahan `CHECKLIST_STATE.json`.
5. Jika hasil re-check **menggugurkan gate** → status gate diturunkan di `CHECKLIST_STATE.json` (dengan field `last_reverified`), entri `GATE_DOWNGRADE` dibuat di fault log, dan seluruh klaim README/dokumen yang mengutip gate tersebut disinkronkan pada commit yang sama.

## 3. Aturan Larangan (G2 global)
- Dilarang mengubah status gate tanpa justifikasi tertulis di fault log.
- Dilarang menutup gate dengan klaim tanpa path artefak bukti konkret.
- `Milestone_03` dan gate `PARTIAL/GAP_IDENTIFIED/NOT TESTED` tidak boleh ditulis sebagai "PASS & LOCKED" di dokumen mana pun.

## 4. Rekonsiliasi Baseline 2026-10-20 (contoh penerapan)
- Temuan: README §2 lama mengklaim "25/25 gates PASS & LOCKED".
- Fakta `CHECKLIST_STATE.json` (parsed): 24 gate Phase 1–6 PASS & LOCKED; `Milestone_03` 25 gate berstatus PARTIAL/unlocked.
- Aksi: README §2 dikoreksi ke angka faktual di atas; `CHECKLIST_STATE.json` TIDAK diubah (isinya sudah benar); justifikasi dicatat di `ACCOUNTABILITY_FAULT_LOG.json` (entri `GOVERNANCE_RECONCILIATION`).
- Bukti parse deterministik (24 gate Phase 1–6 PASS/locked; Milestone_03 = 25 gate: 20 PASS murni + 1 PASS-kualifed-3.19 + 1 PASS_READONLY + 1 PARTIAL + 1 GAP_IDENTIFIED + 1 NOT TESTED; total 49 gate): `Analysis/reports/2026-10-20_gate_parse_evidence.txt`. Verdict final: **24** (bukan 25) untuk gate terkunci.
