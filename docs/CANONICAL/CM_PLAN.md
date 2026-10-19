# CM PLAN: CONFIGURATION & EVIDENCE MANAGEMENT (ASL CORE KERNEL)
**Status:** RATIFIED 2026-10-20 | **Standar:** DO-178C configuration management & traceability

## 1. Ruang Lingkup
Semua artefak **RUNTIME VERIFIED** (log serial, screenshot, hasil test, metadata build) dan dokumen konfigurasi kanonik diperlakukan sebagai *baselined configuration items* yang wajib ter-track di git.

## 2. Kebijakan Artefak Bukti
| Kelas Artefak | Lokasi Kanonik | Tracking Git |
| :--- | :--- | :--- |
| Log serial boot/telemetri (M0..M6) | `Analysis/reports/` | **WAJIB ter-commit** (`.text`, `.log`) |
| Screenshot bukti tampilan (mis. `asl_vm_screen.png`) | `Analysis/reports/` | **WAJIB ter-commit** |
| Hasil test/audit (mis. `DEEP_AUDIT_REPORT.md`, `QODER_EXECUTION_LOG.md`) | `Analysis/reports/`, `audit/`, `docs/ROADMAP/` | **WAJIB ter-commit** |
| Dump mentah debugging (`*.dump`, `*.raw`, `*.bin`) | `Analysis/reports/` | TETAP di-ignore (aturan `.gitignore` sudah ada) |
| Log ad-hoc root (`boot_*.log`, `test_de_output*.log`, `trace_gt.log`) | root repo | **Deprecated-location** — salin/sahkan ke `Analysis/reports/` sebelum dirujuk dokumen; root log hanya kerja sementara |
| Artefak build (`target/`, ISO, ELF) | `asl-kernel/target/` | Tidak ter-track; identitas = SHA256 yang dicatat di laporan |

**Status `.gitignore` (verifikasi 2026-10-20):** sudah benar — `Analysis/reports/` ter-track kecuali `*.dump/*.raw/*.bin`. Tidak ada perubahan yang diperlukan; CM_PLAN ini menegakkan kebijakannya.

## 3. Identitas Konfigurasi
- Setiap klaim `RUNTIME VERIFIED` di README §2 wajib menunjuk satu file bukti konkret di `Analysis/reports/` (atau `audit/`), bukan sekadar deskripsi.
- Versi produk kanonik tunggal: `RELEASE_NOTES.md` (header versi) ← dicatat di `CHECKLIST_STATE.json.version` dan `HANDOFF_STATE.json` pada tiap rilis; ketiganya wajib identik saat tagging.
- Perintah verifikasi resmi kanonik: `cargo test --workspace --lib` (host) dan `cargo build --package asl-kernel-bin --target x86_64-unknown-none --release` (bare-metal); lihat `DEVELOPER_ONBOARDING.md` §3 + Workspace Cargo Configuration Map.

## 4. Prosedur Perubahan
1. Perubahan dokumen kanonik / register temuan → commit terpisah dengan justifikasi di pesan commit + append `docs/ROADMAP/ACCOUNTABILITY_FAULT_LOG.json` bila menyentuh status gate/register.
2. Bukti baru (log/screenshot) → simpan di `Analysis/reports/`, commit, lalu rujuk dari dokumen klaim.
3. Re-verifikasi gate terkunci → ikuti `docs/ROADMAP/GATE_REGRESSION_POLICY.md`.
