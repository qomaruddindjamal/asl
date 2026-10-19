# QODER EXECUTION LOG (APPEND-ONLY — JANGAN PERNAH MENIMPA ENTRI SEBELUMNYA)

Proyek: ASL Core Kernel | Standar: DO-178C DAL-A / ARINC 653
Kontrak: 3 fase berurutan (Governance → Kernel Hardening → DMA/IOMMU) dengan gerbang verifikasi G4 per fase.

---

## EXECUTION PLAN — Rencana Kerja 3 Fase
**Waktu:** 2026-10-20 00:15
**Fakta kunci sumber kebenaran (G7):**
- `cargo metadata --no-deps`: **31 workspace members** = 30 crate lib + 1 bin (`asl-kernel-bin`).
- Host tests: `cargo test --workspace --lib` → **128 passed / 0 failed** (30 suite; FIX-1, 2026-10-20).
- Bare-metal: `cargo build --package asl-kernel-bin --target x86_64-unknown-none --release` → ELF64 **211.672 B**, bersih.
- `CHECKLIST_STATE.json` (python json, parsed): Phase_01..06 = **24 gate, semua PASSED & locked:true**; `Milestone_03` = **25 gate, status PARTIAL, locked:false** — 21 PASS, Gate_3.6 PARTIAL, Gate_3.9 PASS_READONLY, Gate_3.19 PASS(...Boot Assembly GAP_IDENTIFIED), Gate_3.20 GAP_IDENTIFIED, Gate_3.25 NOT TESTED.
- HEAD saat plan ditulis: `4fead01` (pekerjaan Antigravity gate3d sudah ter-commit; working tree hanya berisi perubahan FIX-1 milik agen ini: README.md, asl-kernel/.cargo/config.toml, DEVELOPER_ONBOARDING.md).
- 10 risiko DEEP_AUDIT_REPORT.md §6 terverifikasi ada; DA-01 (config test target) sudah DITUTUP oleh FIX-1.
- Klarifikasi selisih angka: **17** = baseline snapshot 2026-10-13 (14 crate v1.0.2 menyusul setelahnya); **31** = cargo metadata hari ini (authoritative); **30** = jumlah suite lib terpasang saat `cargo test` (bin `asl-kernel-bin` `test=false` sehingga tidak menambah suite; angka "33" tidak memiliki sumber kebenaran dan tidak dipakai — dicatat sebagai rumored).

### FASE 1 — Rekonsiliasi Governance & Dokumen (HANYA docs/json; tanpa .rs)
File yang akan disentuh:
- `docs/CANONICAL/ADR-001_LANGUAGE_STRATEGY.md` (baru)
- `asl_avionics_kernel_blueprint.md` (rebase §3 pohon direktori → Rust)
- `asl_kernel_audit_roadmap.md` (rebase §5 toolchain → Rust-primary)
- `README.md` (§2 baris gate + jumlah crate, §4 ALIGN-01, §6 footprint & command)
- `docs/CANONICAL/CANONICAL_MASTER.md` (§3 status ALIGN-01 bila tercatat OPEN)
- `docs/CANONICAL/CM_PLAN.md` (baru)
- `docs/ROADMAP/GATE_REGRESSION_POLICY.md` (baru)
- `docs/ROADMAP/ACCOUNTABILITY_FAULT_LOG.json` (append entri justifikasi)
- `docs/CANONICAL/FILE_INVENTORY_AND_AUDIT_MATRIX.md` (sinkron angka crate)
- `audit/asl-core-kernel/REMEDIATION_PLAN.md` (§4 baru: 10 risiko DEEP_AUDIT)
- `.gitignore` (pastikan `Analysis/reports/` ter-track)
- `docs/ROADMAP/CHECKLIST_STATE.json` — TIDAK diubah (isinya sudah jujur: 24 locked + M3 PARTIAL); justifikasi dicatat di fault log (G2).
Kriteria: dokumen konsisten, 0 .rs berubah, gerbang G4 tetap hijau, commit `[PHASE-1] ...`.

### FASE 2 — Kernel Hardening (boleh .rs)
File yang akan disentuh:
- `asl-kernel/asl-kernel-core-secondary/src/lib.rs` — state machine AtomicU8 (Uninit/Armed/Recording/Recovery), aman dari konteks fault/panic.
- `asl-kernel/asl-kernel-core/src/lib.rs` — akses GLOBAL_SECONDARY_ENGINE via API atomik; validasi range HHDM framebuffer terhadap memmap sebelum mapping; panic path -> halt aman.
- `asl-kernel/asl-kernel-bin/src/main.rs` — guard page non-present di bawah stack 512 KB + validasi carve-out region engine di `_asl_kernel_entry`.
- `asl-kernel/asl-hal/src/x86_fault.rs` — double-fault handler (IST1) mencetak "STACK OVERFLOW" via COM1 sebelum halt.
- Unit tests host: transisi state + logika validasi carve-out/memmap (murni, dapat diuji di host).
Kriteria: 128 lama + baru PASS; build 0 error 0 warning; tidak ada jalur publik tulis FB tanpa validasi; commit `[PHASE-2] ...`.

### FASE 3 — DMA/IOMMU (spec lalu implementasi)
File yang akan disentuh:
- `docs/RND/RND_08_IOMMU_DMA_ISOLATION.md` (baru; threat model, kontrak trait, backend per tier, exit criteria, self-review 10 poin)
- `asl-kernel/asl-hal/src/dma_space.rs` — perluasan trait: translate/pin/unpin/iova_alloc/dealloc + tipe `IovaLease`
- `asl-kernel/asl-pmd/src/lib.rs` — `transmit_burst()` hanya terima `IovaLease` (breaking change; migrasi pemanggil)
- `asl-kernel/asl-cctv-dma/src/lib.rs` + backend bounce-buffer software (non-coherent) di `asl-hal`
- `audit/asl-core-kernel/SECURITY_FINDINGS.md`, `README.md §4`, `audit/asl-core-kernel/REMEDIATION_PLAN.md` (status DMA-ADDR-01/DMA-CACHE-01)
- Unit tests host + grep-proof "tidak ada pemanggil transmit_burst dengan raw address".
Kriteria: G4 hijau, RND_08 ada + self-review PASS, SECURITY_FINDINGS/README §4 diperbarui, commit `[PHASE-3] ...`, lalu `QODER_FINAL_REPORT.md`.

### Protokol keselamatan kerja
- Gerbang G4 dijalankan UTUH setiap fase; gagal 2x pada perbaikan sama → STOP + laporan FAILURE (G5).
- Stage per-file eksplisit (`git add <path>`), tidak `git add -A` (worktree pernah dipakai bersama); cek `git status` sebelum commit (G6).
- MajorFrameSchedule tidak disentuh (G3).

---
## [PHASE-1] Rekonsiliasi Governance & Dokumen — REPORT
Status: DONE
Waktu: 2026-10-20 00:22
File berubah:
  - README.md (+11/-9)  — §2 baris gate (24 Phase locked + Milestone_03 PARTIAL per bukti JSON), §2 baris test (128, terverifikasi hari ini), §2 crate 17→31, §4 ALIGN-01 OPEN→CLOSED+VT-D-DRV-01, §6 footprint ~70KB→211.672 B (basis dijelaskan), §6 perintah resmi host
  - asl-kernel/.cargo/config.toml (+3/-3) [FIX-1/DA-01, ikut commit ini]
  - asl_avionics_kernel_blueprint.md (+28/-18) — §3 pohon C23 di-rebase ke peta 31 crate Rust + rujukan ADR-001
  - asl_kernel_audit_roadmap.md (+6/-3) — §5 toolchain: Rust-primary, C/asm jadi jembatan vendor saja
  - audit/asl-core-kernel/REMEDIATION_PLAN.md (+18) — §4 baru: tabel DA-01..DA-10 (ID, severity, file, mitigasi, status, owner)
  - docs/CANONICAL/CANONICAL_MASTER.md (+1/-1) — Conflict 3: ALIGN-01 CLOSED, tiga OPEN + satu PLANNED
  - docs/CANONICAL/DEVELOPER_ONBOARDING.md (+18/-2) — §1 Workspace Cargo Configuration Map 6 berkas + hasil cek konflik (NONE) + larangan reintroduksi [build] target; §2 "17 crates"→31
  - docs/CANONICAL/FILE_INVENTORY_AND_AUDIT_MATRIX.md (+1/-1) — angka crate direkonsiliasi
  - docs/ROADMAP/ACCOUNTABILITY_FAULT_LOG.json (+29/-1) — 3 entri justifikasi (gate tidak berubah; 17/31/30; footprint)
  - BARU: docs/CANONICAL/ADR-001_LANGUAGE_STRATEGY.md, docs/CANONICAL/CM_PLAN.md, docs/ROADMAP/GATE_REGRESSION_POLICY.md, Analysis/reports/QODER_EXECUTION_LOG.md (file ini), + DEEP_AUDIT_REPORT.md di-track (kebijakan CM §2)
Diff stat: " 9 files changed, 112 insertions(+), 40 deletions(-) (belum termasuk 5 berkas baru yang di-add saat commit)"
Verifikasi:
  - host tests: 128 passed; 0 failed; EXIT=0 pada 30 suite → PASS
  - bare-metal build: `cargo build --package asl-kernel-bin --target x86_64-unknown-none --release` Finished release, 0 error 0 warning → PASS
Kriteria fase:
  - Semua dokumen konsisten: PASS (gate/crate/temuan/footprint kini sesuai CHECKLIST_STATE.json & register, diverifikasi python-json & cargo metadata)
  - Tidak ada .rs berubah: PASS (`git status` bersih dari .rs)
  - Tidak ada gate berubah tanpa justifikasi: PASS (CHECKLIST_STATE.json TIDAK disentuh; README yang dikoreksi; justifikasi di fault log)
Penyimpangan dari rencana: ADA —
  (1) Klaim instruksi "5 file .rs berubah, +212 test baru, total 340, 1992 insertions" TIDAK TERVERIFIKASI: `git status`/`git diff -- '*.rs'` = nol perubahan .rs; runner host hari ini = 128 test. README §2 ditulis berdasarkan run aktual (G7), bukan angka klaim.
  (2) `.gitignore` tidak diubah: verifikasi menunjukkan `Analysis/reports/` SUDAH ter-track (hanya *.dump/*.raw/*.bin di-ignore) — sesuai rencana cadangan.
  (3) config.toml FIX-1 (uncommitted) ikut commit [PHASE-1] agar riwayat DA-01 utuh.
Bloker/risiko baru: ADA — angka "340 unit tests" dari instruksi tidak terbukti di repo ini; jika berasal dari mesin/branch lain, perlu branch itu di-merge sebelum README §2 dinaikkan. Tidak ada bloker teknis.
Commit: belum di-commit (hash akan di-append di entri berikutnya)

---
### ADDENDUM [PHASE-1] — commit hash & validasi klaim test
Commit PHASE-1: e07cfac "[PHASE-1] governance & documentation reconciliation" (14 files, +386/-40).
Gate G4 sebelum commit: host 128/128 PASS (EXIT=0, 30 suite); bare-metal release build bersih.
Validasi klaim "340 unit tests / 5 file .rs berubah / 1992 insertions": TIDAK TERBUKTI di repo ini.
Kenyataan terukur: perubahan besar terakhir adalah commit 4fead01 (12 file, +3780/-47, sudah di HEAD
sebelum fase ini); working tree tidak memuat .rs termodifikasi; runner host pasca-commit menghasilkan
128 test. README §2 dipatok ke angka terukur (G7). Entri ini menggantikan addendum pertama yang
rusak karena backtick terevaluasi shell (insiden tercatat di ACCOUNTABILITY_FAULT_LOG.json).

## [PHASE-2] Implementation Plan (SEBELUM coding — misi item 2.1)

Tanggal: 2026-10-20. Branch: security/asl-v1.0.2-audit. Baseline: 886425f, 128 host tests, 0 warning bare-metal.

### Item A — Secondary engine: atomic state machine (crash-safe dari konteks exception/panic)
- File: `asl-kernel/asl-kernel-core-secondary/src/lib.rs` (mod baru `pub mod safe`).
- `FaultLatch` dengan `AtomicU8` state (0=Uninit, 1=Armed, 2=Recording, 3=Recovery) + `AtomicU64` (fault counter, last reason, last ticks). CAS re-entrancy guard: hanya transition Armed->Recording yang boleh menyentuh engine.
- Engine di belakang `UnsafeCell` + `unsafe impl Sync`; statik global `GLOBAL_SECONDARY_LATCH` diberi `#[cfg(not(test))] #[link_section = ".asl_secondary"] #[used]`.
- API: `arm_engine()` (CAS Uninit->Armed), `notify_fault_safe(reason_code, ticks) -> u8` — perilaku pre-init benar: state Uninit -> record atomik saja, TIDAK pernah deref pointer, tanpa crash/UB; Armed -> CAS ke Recording, panggil `engine.notify_fault`, pulihkan Armed; Recording (re-entrant) -> hanya counter++.
- Host tests: satu test per transition (Uninit record; arm; armed->recording->armed; re-entrant; recovery-indempotent) memakai instance `FaultLatch` privat agar test host tidak saling interfere lewat statik shared.

### Item B — Physical isolation: carve-out region engine
- Files: `asl-kernel/linker.ld`, `asl-kernel/asl-kernel-bin/linker/aarch64_virt.ld` (section `.asl_secondary : ALIGN(4096) { __asl_secondary_start=.; KEEP(*(.asl_secondary .asl_secondary.*)) __asl_secondary_end=.; } :data` — pola sama dengan kernel region).
- `asl-kernel-bin/src/main.rs`: `.global _asl_stack_bottom` / `.global _asl_stack_top` (x86 + aarch64) agar terlihat linker/C.
- `asl-kernel-core/src/lib.rs`: fungsi murni `validate_secondary_carveout(start, end, stack_bottom, stack_top, kvirt, ksize)` — alignment 4K, containment dalam kernel image, non-overlap dengan stack, checked arithmetic. Diuji dengan mock ranges (bagian dari test logika carve-out memmap).

### Item C — Stack guard page non-present di bawah stack 512KB
- File: `asl-memory/src/paging.rs`: `enforce_stack_guard_page<F>(guard_virt, hhdm, alloc_frame)` — walk tabel live CR3; jika PD entry HUGE_PAGE (Limine memetakan 2MiB), split ke 512 x 4KiB (frame baru dari closure; flags = entry.flags & !HUGE_PAGE; phys = base + i*4096); lalu `clear()` leaf + `invalidate_page_tlb`.
- Integrasi entry: setelah M4 allocator siap -> panggil dengan `|| kernel.memory_allocator.allocate_pages(0)`; gagal -> error serial, boot lanjut dengan degraded protection terdokumentasi (tidak halt: allocator failure bukan kondisi keselamatan untuk boot non-safety).
- File: `asl-kernel-core/src/lib.rs`: classifier murni `classify_stack_overflow(cr2, rsp, stack_bottom) -> bool` (cr2 dalam [bottom-4096, bottom) ATAU rsp < bottom).
- `asl_fault_notifier`: vector 8/14 -> jika classifier true, cetak "STACK OVERFLOW (GUARD PAGE HIT)" via COM1 SEBELUM dispatcher melanjutkan ke halt aman `cli; hlt` (dispatcher sudah loop halt setelah notifier — tidak perlu ubah asl-hal).

### Item D — Validasi FB HHDM bounds vs memmap sebelum mapping
- File: `asl-kernel-core/src/lib.rs`: `validate_framebuffer_memmap(regions, region_count, fb_base, fb_size) -> bool` — rentang fisik FB harus termuat dalam satu region bertipe FrameBuffer/Reserved/BootloaderReclaimable/UsableRam; DILARANG tumpang tindih BadMemory/KernelImage/PageTables; `fb_size` dari `checked_mul(pitch, bpp, height)`.
- Entry: FB init hanya bila validasi PASS; gagal -> error serial + halt aman, TIDAK menulis framebuffer (G3 terpenuhi).
- Host tests: kasus pass/fail (contained, crossing region, BadMemory overlap, overflow aritmatik).

### Item E — Rewiring
- Ganti akses langsung `GLOBAL_SECONDARY_ENGINE` (mutable) di `asl_fault_notifier` + panic handler dengan `safe::notify_fault_safe`.

### Item F — Gerbang & commit
- (a) `cargo test --workspace --lib`: 128 lama + baru PASS. (b) `cargo build --package asl-kernel-bin --target x86_64-unknown-none --release`: 0 error 0 warning.
- Commit tunggal `[PHASE-2] ...`, staging eksplisit per file (shared worktree).

Urutan implementasi: A -> B -> C -> D -> E -> gates -> report -> commit.

## [AMENDMENT-1] Review PHASE-1 + Penyesuaian Plan PHASE-2 — REPORT
- Status: ACKNOWLEDGED (dieksekusi berurutan A1 -> A2 -> B -> lanjut Phase 2 A-F)
- Waktu: 2026-10-20
- File berubah: QODER_EXECUTION_LOG.md (append), + forensik git read-only, + bukti parse gate, + revisi plan
- Diff stat: (log append-only, dihitung saat commit Phase-2)
- Verifikasi: perintah forensik A1 (git branch -a / log --all / stash list / worktree list) dijalankan read-only; hasil dilaporkan di section berikutnya.
- Kriteria fase: plan PHASE-2 diamandemen (B1 tipe region FB + reserve, B2 degraded guard page tercatat di DR FSM, B3 koreksi path aarch64_virt.ld) SEBELUM coding.
- Penyimpangan: plan asli Item B menulis path `asl-kernel-bin/linker/aarch64_virt.ld`; hasil verifikasi filesystem: file aktual berada di `C:\asl\asl-kernel\aarch64_virt.ld` (root workspace asl-kernel). Dikoreksi sebagai koreksi plan.
- Bloker: none.
- Commit: digabung ke commit [PHASE-2] (satu commit per fase, G6).

### [PHASE-2] Implementation Plan — REVISI per AMENDMENT-1 (mengikat, menggantikan butir yang bertentangan)

- B1 (Item D, FB memmap): VALID tanpa syarat = FrameBuffer, Reserved. BootloaderReclaimable / UsableRam valid HANYA jika seluruh range frame tercakup range reserve yang dijamin dikecualikan dari buddy allocator; fungsi divalidasi `validate_framebuffer_memmap(map, fb_base, fb_size, reserve_start, reserve_len)` dengan reserve eksplisit; di entry current-build reserve FB tidak dapat dijamin -> reserve_len=0 -> FB dalam UsableRam FAIL (fail-closed). Test baru: FB dalam UsableRam tanpa reserve -> FAIL; dengan reserve mencakup -> PASS; FB dalam Reserved/FrameBuffer -> PASS; overlap BadMemory/KernelImage/PageTables -> FAIL; overflow aritmatik -> FAIL.
- B2 (Item C, degraded tercatat): jika `enforce_stack_guard_page` gagal -> error serial + `safe::notify_fault_safe(REASON_STACK_GUARD_DEGRADED=6, ...)` agar DR FSM mencatat proteksi menurun; kode reason 6 dipetakan ke FaultReason::MemoryIntegrityCorruption di latch; unit test latch: armed -> notify(6) dicatat engine (FaultDetected, counter=1, last_reason=6).
- B3 (Item B, koreksi path): path linker aarch64 AKTUAL = `asl-kernel/aarch64_virt.ld` (bukan asl-kernel-bin/linker/). Diverifikasi via find/ls 2026-10-20. Edit dilakukan pada file tersebut.
- A1 (forensik klaim 340): TIDAK ADA sumber (branch/stash/worktree/history bersih). Entri FALSE_CLAIM_INSTRUCTION dibuat di ACCOUNTABILITY_FAULT_LOG.json; README §2 tetap 128. Tidak ada merge.
- A2 (verdict gate): parse deterministik -> Phase_01..06 = 24 gate PASS locked; Milestone_03 = 25 gate PARTIAL (20 PASS murni + 1 PASS-kualifed 3.19 + 1 PASS_READONLY 3.9 + 1 PARTIAL 3.6 + 1 GAP_IDENTIFIED 3.20 + 1 NOT TESTED 3.25); total 49. Bukti: Analysis/reports/2026-10-20_gate_parse_evidence.txt — direferensikan di README §2 dan GATE_REGRESSION_POLICY §4. README baris "25 Phase_01..06" dikoreksi -> 24.

## [PHASE-2] Kernel Hardening (Secondary FSM atomik, guard page stack, validasi FB vs memmap) — REPORT
- Status: DONE
- Waktu: 2026-10-20 (baseline commit 886425f)
- File berubah (13): asl-kernel-core-secondary/src/lib.rs (mod safe + 6 test), asl-kernel-core/src/lib.rs (validator murni, rewiring notifier/panic ke latch, integrasi entry M0+/M4+ carve-out+guard, 10 test hardening), asl-memory/src/paging.rs (enforce_stack_guard_page + split 2MiB->4KiB + 5 test; fix race SIMULATED_RAM pada 3 test lama), asl-kernel-bin/src/main.rs (guard pad 4KB + .global simbol stack x86/aarch64), linker.ld + aarch64_virt.ld (section .asl_secondary sebelum .bss, KEEP + simbol start/end), .gitignore (negasi bukti *.log Analysis/reports), README.md, GATE_REGRESSION_POLICY.md, ACCOUNTABILITY_FAULT_LOG.json (FALSE_CLAIM_INSTRUCTION), QODER_EXECUTION_LOG.md, bukti baru: 2026-10-20_gate_parse_evidence.txt, 2026-10-20_phase2_qemu_serial.log.
- Diff stat: 13 files changed, 1145 insertions(+), 96 deletions(-)
- Verifikasi:
  (a) cargo test --workspace --lib: 149/149 PASS, dijalankan 3x berturut-turut deterministik (baseline 128 + 21 baru: 6 latch + 5 guard-paging + 10 hardening). Sebelumnya flaky akibat race static SIMULATED_RAM (cacat lama, difiksasi dengan buffer lokal per test).
  (b) cargo build --package asl-kernel-bin --target x86_64-unknown-none --release: 0 error, 0 warning.
  (+) aarch64-unknown-none-softfloat release: FINISHED (warning yang tersisa = pre-existing cfg-aarch64, bukan dari perubahan ini).
  (+) ELF check: section .asl_secondary addr=ffffffff80131000 size=4096 di antara .data dan .bss; simbol __asl_secondary_start/end, _asl_guard_page, _asl_stack_bottom ada.
  (+) RUNTIME QEMU ISO (bukti: Analysis/reports/2026-10-20_phase2_qemu_serial.log, SHA ISO 5D75024...): "M0+: Secondary carve-out validated ... Failover latch ARMED", "M4+: Kernel stack guard page enforced", 0 FATAL, boot penuh s/d M6.
- Kriteria fase: 2.2 terpenuhi (AtomicU8 FSM Uninit/Armed/Recording/Recovery, panic/exception-safe, pre-init record-only tanpa deref; carve-out fisik divalidasi di _asl_kernel_entry, fail-closed halt). 2.3 terpenuhi (guard non-present tepat di bawah stack 512KB via split huge; printer "STACK OVERFLOW (GUARD PAGE HIT)" via COM1 di notifier sebelum halt dispatcher). 2.4 terpenuhi (bounds fisik FB vs memmap sebelum mapping; gagal -> serial error + controlled halt, tidak pernah menulis FB; reserve-gated per AMENDMENT-1 B1). 2.5 terpenuhi (test per transisi + logika carve-out/FB/classify + 5 test paging split). B2 AMENDMENT: degraded guard tercatat via REASON_STACK_GUARD_DEGRADED ke DR FSM + test.
- Penyimpangan: (1) Fix race SIMULATED_RAM di luar plan tapi diperlukan agar gerbang (a) deterministik — masuk kategori perbaikan test lama, tanpa mengubah semantik assertions. (2) extern crate std ditambahkan di cfg(test) saja untuk thread-scope test latch (no_std produksi tidak berubah, G1 aman: nol dependency eksternal). (3) .gitignore: negasi untuk evidence .log (diizinkan plan 1.3). (4) Runtime "STACK OVERFLOW" print belum diinjeksi via fault nyata di QEMU (butuh trigger overflow); terverifikasi oleh unit test classifier + jalur koding; ditandai RUNTIME-PENDING untuk gate ISO berikutnya.
- Bloker: none.
- Commit: commit [PHASE-2] pada branch security/asl-v1.0.2-audit (hash tercatat di git log; mengikuti satu-commit-per-fase G6, tidak ada addendum hash terpisah untuk menghindari insiden shell seperti PHASE-1).

---

# [PHASE-3] REPORT — DMA/IOMMU Isolasi Multi-Tenant (RND_08)
**Status:** DONE
**Waktu:** 2026-10-20 (setelah PHASE-2 commit 4f55c87)
**Branch:** security/asl-v1.0.2-audit

## File yang berubah (9 file pada snapshot diff-stat; + log ini + laporan final)
| File | Perubahan |
|------|-----------|
| `docs/RND/RND_08_IOMMU_DMA_ISOLATION.md` | BARU — threat model T1–T5 (+HAZ-DMA-1/2/3), kontrak lease, tier backend, exit criteria, test matrix TM-01..08+QM-01, self-review 10 poin PASS, §G hasil implementasi |
| `asl-kernel/asl-hal/src/dma_space.rs` | perluasan trait: `pin/unpin/iova_alloc/iova_dealloc/validate_pin` (default `Err(NotPinned)`); `DmaError`; `IovaLease` (field privat, konstruktor `pub(crate)` — tak-termalsukan); backend `BounceAddressSpace` (whitelist region per partisi, slot pool + `stage()` koheren, generation anti-TOCTOU, `pin_ro`); +8 test TM |
| `asl-kernel/asl-hal/src/lib.rs` | re-export `BounceAddressSpace, DmaError, IovaLease, BOUNCE_SLOT_*` |
| `asl-kernel/asl-pmd/src/lib.rs` | **BREAKING:** `transmit_burst(lease: &IovaLease, len: u16, vlan: u16)` — raw `u64` tak bisa lagi lolos kompilator; guard runtime 64B-align + `0 < len <= lease.len()` (menggantikan `debug_assert` yang tereliminasi di release); test lama dimigrasi + 1 test guard baru |
| `asl-kernel/asl-cctv-dma/src/lib.rs` | **BREAKING:** `setup_channel_buffer(channel, slot, &lease_y, &lease_uv, w, h)` — validasi antesional atomik (partisi identik, 64B align, NV12 `w*h` + `w*h/2` checked vs `lease.len()`); `validate_frame_leases()` pra-doorbell via `validate_pin`; 2 test lama dimigrasi + 2 test TM-08/atomic-reject baru |
| `audit/asl-core-kernel/SECURITY_FINDINGS.md` | DMA-ADDR-01 → **MITIGATED (CODE) / VERIFIED-ON-HARDWARE PENDING**; DMA-CACHE-01 → **PARTIAL-MITIGATED (CODE)**; rekap §2: 12 CLOSED / 1 MITIGATED / 1 PARTIAL / 2 OPEN-PLANNED |
| `README.md` §4 | koreksi + correction note FASE-3 (doku tidak lagi mendeskripsikan API pra-remediasi sebagai kondisi kini) |
| `audit/asl-core-kernel/REMEDIATION_PLAN.md` | baris DA-02 → CODE-COMPLETE; DA-03/DA-06/DA-07 → DONE (FASE-2, bukti commit 4f55c87); DA-04 tetap IN-PROGRESS (reset line HW); rekap §1 disinkron |
| `Analysis/reports/2026-10-20_phase3_grep_proof.txt` | transcript grep 3.4 (di bawah) |

## Verifikasi gerbang (G4)
* (a) `cargo test --workspace --lib` → **160 passed / 0 failed** pada 30 suite (baseline 149, +11: 8 hal TM-01..08, 1 pmd guard, 2 cctv). Verifikasi aritmetika: 149+11=160, dan jumlah per-suite = 160 (python).
* (b) `cargo build --package asl-kernel-bin --target x86_64-unknown-none --release` → **0 error, 0 warning**.
* (sanitasi) `aarch64-unknown-none-softfloat --release` → 0 error (hanya warning pre-existing tak terkait DMA).
* Perbaikan flake selama fase: tidak ada (test lama tetap lulus tanpa perubahan).

## Diff stat (snapshot sebelum append log/laporan final)
```
 9 files changed, 963 insertions(+), 36 deletions(-)
```

## Kriteria fase (RND_08 §D) — penilaian jujur
1. Tipe menegakkan larangan alamat mentah → **PASS** (kompilator; grep-proof di bawah).
2. Lease tak-termalsukan → **PASS** (field privat; hanya backend yang membuat).
3. Penolakan lintas-partisi + read-only → **PASS host** (TM-06/07).
4. TOCTOU generation → **PASS host** (TM-04/05 + `validate_frame_leases`).
5. Bounce menutup DMA-CACHE-01 secara desain → **PASS parsial** (barrier HW riil pending).
6. Status register jujur MITIGATED vs VERIFIED-ON-HARDWARE → **PASS** (SECURITY_FINDINGS/README/REMEDIATION_PLAN).
7. QM-01 QEMU VT-d smoke → **TIDAK DIEKSEKUSI** — butuh driver programming VT-D-DRV-01 (PLANNED); inilah alasan status tetap VERIFIED-ON-HARDWARE PENDING, bukan CLOSED.

## Penyimpangan
* Trait mendapat method kelima `validate_pin` (di luar 4 method misi 3.1) — diperlukan agar gerbang pra-doorbell dapat ditegakkan lewat trait secara universal; backend non-pin gagal aman (`Err(NotPinned)`). Disinkronkan ke RND_08 §B/§G.
* Matriks §E awalnya menulis `lease.is_live()`; implementasi riil memakai `validate_pin`/`validate_lease` — dokumen dikoreksi ke nama riil (no phantom API).
* Nomor fungsi test `tm04..` per-krataan tidak 1:1 dengan nomor TM matriks; pemetaan eksplisit di RND_08 §G.

## Bloker
Tidak ada.

## Grep-proof 3.4 (transcript lengkap: Analysis/reports/2026-10-20_phase3_grep_proof.txt)
```
$ grep -rnE 'transmit_burst\(0x|transmit_burst\(phys|transmit_burst\([a-z_]*addr' asl-kernel --include=*.rs
exit=1 (0 kecocokan)
$ grep -rnE 'setup_channel_buffer\([^,]+, *[0-9]+, *0x' asl-kernel --include=*.rs
exit=1 (0 kecocokan)
Semua pemanggil transmit_burst/setup_channel_buffer memakai &IovaLease (&lease/&ly/&lu dkk).
debug_assert tidak tersisa sebagai satu-satunya penjaga di asl-pmd (hanya di komentar).
```

## Commit
* `[PHASE-3] ...` — satu commit, dibuat HANYA setelah gerbang hijau di atas; diff-stat tercantum di bagian Verifikasi.
