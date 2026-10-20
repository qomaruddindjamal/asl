# QODER EXECUTION LOG (APPEND-ONLY — JANGAN PERNAH MENIMPA ENTRI SEBELUMNYA)

Proyek: ASL Core Kernel | Standar: DO-178C DAL-A / ARINC 653
Kontrak: 3 fase berurutan (Governance → Kernel Hardening → DMA/IOMMU) dengan gerbang verifikasi G4 per fase.
[ADR-002, 2026-10-20 FASE-4]: baris "Standar" di atas adalah teks header historis (entri lama tidak ditimpa). Status kanonik kini: **DO-178C-aligned, ladder DAL-C->A (lihat docs/CANONICAL/ADR-002_CERTIFICATION_LADDER.md)**; kontrak aktual berjalan = 4 fase (PHASE-4 Closeout & Reframing ditambahkan via AMENDMENT-2).

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

---

# [PHASE-4] Closeout & Reframing — EXECUTION PLAN
**Diterima:** 2026-10-20 (AMENDMENT-2) | **Status:** IN-PROGRESS | **Sifat:** docs-only, tidak menyentuh file `.rs`, tidak mengubah CHECKLIST_STATE.json (G2)

| Step | Isi | Gerbang keluar |
|------|-----|----------------|
| 4A.1 | Ringkasan PHASE-2 REPORT (10 baris, R3): nama test baru (transisi latch, validasi carve-out, FB pass/fail termasuk UsableRam-tanpa-reserve FAIL), bukti B2 (`notify_fault_safe` guard-degraded), baseline 149 = 128 + 21 | angka cocok dengan `4f55c87` |
| 4A.2 | Hasil A1 (klaim 340): forensik branch/stash/worktree, entri `FALSE_CLAIM_INSTRUCTION` dikutip apa adanya | ada di fault log |
| 4A.3 | Path + ringkasan `2026-10-20_gate_parse_evidence.txt` (A2) dan `2026-10-20_phase3_grep_proof.txt` | file terbaca |
| 4B.1 | BARU `docs/CANONICAL/ADR-002_CERTIFICATION_LADDER.md`: keputusan "DO-178C-aligned", target pertama DAL-C di x86_64; ladder DAL-C(v1.1)→B→A dengan exit criteria per anak tangga (MC/DC, tim verifikasi independen, DO-330 tool qualification Gemini_Assist/Sub_Assist sebelum output jadi bukti); justifikasi | self-contained |
| 4B.2 | Sinkron header "DO-178C DAL-A" sebagai STATUS → "DO-178C-aligned, ladder DAL-C→A (lihat ADR-002)" di README §1, header log, RND_08, dokumen lain; konten teknis lain tidak diubah | grep bersih |
| 4C.1–3 | RND_08: koreksi wording §F#4; paragraf "Concurrency model" di §B (single-owner backend, `&mut self` kontrak, limitasi dievaluasi ulang saat SMP); §G diberi label KANONIKAL TRACE TABLE TM↔test; rename nomor test = opsional, DITUNDA | docs-only |
| 4D.1–2 | REMEDIATION_PLAN §5 "Hardware Verification Gate": tabel prioritas QM-01[P1], STACK-OVERFLOW runtime injection[P1], cache-barrier HW[P2], ATOMIC-SPARC[P3], DA-04/05/09/10 per severity; kolom ID/exit criteria/bukti/prasyarat HW/VM/owner; P1 exit criteria harus bisa diverifikasi via QEMU/VirtualBox | docs-only |
| Gerbang | `cargo test --workspace --lib` = 160/160; build x86_64-unknown-none release 0/0; commit tunggal `[PHASE-4]` staging per-file eksplisit; lalu regenerasi QODER_FINAL_REPORT.md (PHASE-1..4 + ADR-002) | G4 |

---

# [PHASE-4] Closeout & Reframing — REPORT
**Status:** DONE
**Waktu:** 2026-10-20 (setelah PHASE-3 commit `b17e572`)
**Sifat:** docs-only — `git status` fase ini hanya berisi file `.md`; tidak ada `.rs` tersentuh; `CHECKLIST_STATE.json` tidak diubah (G2)

## FASE 4A — PAKET BUKTI REVIEW

### 4A.1 PHASE-2 REPORT dalam 10 baris (R3)
1. Commit `4f55c87` (13 file, +1145/−96); gerbang saat itu 149/149 = **baseline 128 + 21 test baru** (aritmetika terverifikasi).
2. Transisi latch (6 test, `asl-kernel-core-secondary`): `test_latch_preinit_uninit_records_without_engine_touch`, `test_latch_arm_only_from_uninit`, `test_latch_armed_notify_records_via_engine_then_recovery`, `test_latch_recovery_terminal_counts_without_engine_reentry`, `test_latch_guard_page_degraded_reason_recorded_in_engine`, `test_latch_concurrent_notify_counts_all_and_keeps_one_engine_writer` (std::thread::scope, 4 thread).
3. Validasi carve-out fisik (5 test, `asl-kernel-core`): `test_carveout_valid_layout`, `test_carveout_rejects_misaligned_unordered_and_zero`, `test_carveout_rejects_outside_kernel_image`, `test_carveout_rejects_stack_overlap`, `test_carveout_rejects_virtual_overflow`.
4. Guard page paging (5 test, `asl-memory`): `test_guard_page_splits_huge_page_and_clears_only_guard_leaf`, `test_guard_page_rejects_invalid_addresses`, `test_guard_page_not_mapped_without_pml4`, `test_guard_page_on_unmapped_pd_installs_empty_pt`, `test_guard_page_allocation_failure_keeps_huge_mapping_intact`.
5. FB pass/fail (4 test + 1 classify): `test_fb_in_framebuffer_or_reserved_region_passes` (VALID tanpa syarat per B1), `test_fb_in_usable_ram_requires_full_reserve_coverage` (UsableRam/BootloaderReclaimable PASS hanya dengan reserve penuh), **`test_fb_overlap_poisoned_or_crossing_region_fails` memuat kasus UsableRam-tanpa-reserve → FAIL**, `test_fb_arithmetic_edge_cases_fail_closed`, `test_classify_stack_overflow_guard_hit_and_sp_below`.
6. Bukti B2: kegagalan `enforce_stack_guard_page` memanggil `safe::notify_fault_safe(REASON_STACK_GUARD_DEGRADED=6, ...)` di `asl-kernel-core/src/lib.rs:956-957`; reason 6 dipetakan ke MemoryIntegrityCorruption dan tercatat di engine — teruji oleh `test_latch_guard_page_degraded_reason_recorded_in_engine`.
7. Runtime proof QEMU: serial `M0+: ... Failover latch ARMED.` + `M4+: Kernel stack guard page enforced` (log `2026-10-20_phase2_qemu_serial.log`, 49 baris, 0 FATAL).
8. Bug flaky pra-FASE-2 diperbaiki deterministik: data race `SIMULATED_RAM` diganti buffer lokal per test; 3x149/149 berturut-turut.
9. Deviasi tercatat: injeksi overflow runtime end-to-end RUNTIME-PENDING (kini HV-2 P1 di gate §5); `extern crate std` hanya di cfg(test); negation `.gitignore` scoped.
10. Build gerbang saat itu: x86_64-unknown-none release 0 error 0 warning; aarch64 OK (warning pre-existing).

### 4A.2 Hasil A1 (klaim hantu "340 tests / 5 file / 1992 insertions")
**Branch TIDAK ditemukan.** Forensik: `git branch -a` hanya feature/asl-v1.0.2, main, security/asl-v1.0.2-audit (+remote mirror); `git log --all` tanpa commit cocok (kandidat terdekat 4fead01 = 9 .rs / 3780 insertions); `git stash list` kosong; `git worktree list` hanya C:/asl; grep "340" di seluruh rev-history hanya menyentuh catatan deviasi log agen itu sendiri; terukur riil 128 (saat itu). **Status: FALSE_CLAIM_INSTRUCTION / REJECTED — bukan merge-pending; tidak ada merge.** Entri fault log (dikutip): actor `instruksi-eksternal (klaim agent pihak ketiga via pesan misi)`, gate_id `README-TESTS-BASELINE`, result `REJECTED`, 6 item evidence git-forensik, notes: "Klaim hantu ... tidak memiliki sumber di repo ini ... klaim agent tanpa bukti dilarang masuk dokumen kanonik; README section 2 tetap dipatok 128 sampai branch sumber ter-merge dan terukur ulang. Tidak ada merge yang dilakukan." (Sumber: `docs/ROADMAP/ACCOUNTABILITY_FAULT_LOG.json`, fault_history entri FALSE_CLAIM_INSTRUCTION; JSON valid.)

### 4A.3 Artefak bukti
* `Analysis/reports/2026-10-20_gate_parse_evidence.txt` (A2): parse deterministik python `CHECKLIST_STATE.json` — Phase_01..06 = **24 gates, semua PASS, locked** (6+4+4+4+3+3); Milestone_03 = 25 gates **PARTIAL, unlocked** (20 PASS + 1 PASS_READONLY + 1 PASS qualified + 1 PARTIAL + 1 GAP_IDENTIFIED + 1 NOT TESTED); grand total 49; verdict final: klaim "25/25" TIDAK sesuai struktur — yang locked = 24.
* `Analysis/reports/2026-10-20_phase3_grep_proof.txt` (misi 3.4): grep lengkap — semua pemanggil `transmit_burst`/`setup_channel_buffer` memakai `&IovaLease`; pola raw-address (`transmit_burst(0x...` dsb) = 0 kecocokan (exit=1); `debug_assert` tidak tersisa sebagai penjaga satu-satunya di asl-pmd.

## FASE 4B — ADR-002 REFRAMING
* BARU `docs/CANONICAL/ADR-002_CERTIFICATION_LADDER.md`: keputusan DO-178C-**aligned**; target sertifikasi pertama **DAL-C pada satu arsitektur (x86_64) di v1.1**; DAL-A = aspirasi ujung tangga. Ladder DAL-C → B → A dengan exit criteria per anak tangga: MC/DC coverage terukur, traceability penuh, **DO-330 tool qualification Gemini_Assist/Sub_Assist SEBELUM outputnya boleh dipakai sebagai bukti verifikasi**, tim verifikasi independen, hardware gate P1 selesai, keterlibatan otoritas formal (bukan self-assessment). Justifikasi 5 poin berbasis bukti (tanpa otoritas, tanpa angka MC/DC, tanpa TQL, hardware gate terbuka, preseden FALSE_CLAIM).
* Sinkron header status → "DO-178C-aligned, ladder DAL-C->A (lihat ADR-002)" pada 15 titik: README §1, CANONICAL_MASTER (2 baris), DEVELOPER_ONBOARDING, RND_08, RND_05, RND_07, LAPORAN_AUDIT_ULANG_DAN_STATUS, SECURITY_FINDINGS, REMEDIATION_PLAN, ASL-V1.0.2-IMPLEMENTATION-REPORT / RECOVERY-ARCHITECTURE / TEST-MATRIX, docs/RND/README. Header historis QODER_EXECUTION_LOG TIDAK ditimpa — pointer ADR-002 ditambahkan di bawahnya (kepatuhan append-only).
* Yang sengaja TIDAK diubah: nama literal artefak teknis (`compile_error!` guard IdentityDmaSpace, fitur cargo `dal-a`, komentar kontrak `.rs`, string banner uname, entri log lama) — bukan klaim status proyek (ADR-002 §4).

## FASE 4C — CLEANUP MINOR RND_08
* 4C.1 §F#4: "PASS parsial-ke-cokelat" → "PASS (parsial — bounce path menutup DMA ke buffer non-koheren caller; cache-maintenance HW diverifikasi di gate hardware terpisah)".
* 4C.2 §B: paragraf **Concurrency model** baru — backend `DmaAddressSpace` single-owner; serialisasi SMP tanggung jawab pemilik backend via kontrak `&mut self`; limitasi dievaluasi ulang saat SMP diaktifkan (kaitan DA-10 / HV-8).
* 4C.3 §G ditandai **KANONIKAL TRACE TABLE TM<->test**; rename nomor fungsi test = DITUNDA (opsional, low-priority).

## FASE 4D — HARDWARE VERIFICATION GATE
* `REMEDIATION_PLAN.md` §5 baru: 8 baris HV-1..HV-8 (kolom ID/prioritas/exit criteria/bukti/prasyarat HW-VM/owner). P1: HV-1 QM-01 VT-d smoke (jalur QEMU q35 `-device intel-iommu`, tanpa hardware fisik), HV-2 injeksi runtime STACK-OVERFLOW end-to-end (VM cold-boot). 4D.2 dipenuhi: setiap P1 punya exit criteria terverifikasi via QEMU/VirtualBox; HV-3/4 punya jalur simulator (Gem5/TSIM3); hanya HV-5 dan konfirmasi akhir HV-3/4/8 mutlak butuh board fisis.

## Verifikasi gerbang (G4)
* `git status` fase ini: c* `git status` fase ini: hanya file `.md` modified/baru — **nol `.rs` berubah**.
* `cargo test --workspace --lib` → **160 passed / 0 failed** (tetap, sesuai tuntutan gerbang fase).
* `cargo build --package asl-kernel-bin --target x86_64-unknown-none --release` → **0 error, 0 warning**.

## Penyimpangan
* Jumlah titik sinkron header (15) melebihi daftar eksplisit misi (README, log, RND_08, "dokumen lain") — mencakup semua dokumen yang menulis DAL-A sebagai cap status; konten teknis tidak tersentuh. Keputusan tercatat.
* "DAL-A preferred" di §B RND_08 dibaca sebagai klaim praktik → ditulis ulang "praktik terkuat yang dipilih proyek, lihat ADR-002".

## Bloker
Tidak ada.

## Commit
Tunggal `[PHASE-4] closeout & certification reframing`, staging per-file eksplisit; QODER_FINAL_REPORT.md diregenerasi mencakup PHASE-1..4 dengan kategori VERIFIED / CODE-COMPLETE / PENDING + referensi ADR-002, ikut commit yang sama (di-fast-aut amend sebelum apa pun dipublikasikan; hash final dicatat pasca-commit).

## Addendum commit PHASE-4 (catatan deviasi transparan, 2026-10-20)
* Commit utama `[PHASE-4]` = `d25f283` (16 file .md, +192/−54; nol `.rs`) — merupakan hasil fast-aut amend yang diizinkan rencana commit di atas (hash awal `3211de4`; amend dilakukan sebelum apa pun dipublikasikan, isi berubah satu baris referensi hash saja).
* Amend mengganti hash sehingga referensi di QODER_FINAL_REPORT menjadi salah; dikoreksi via commit susulan kecil `[PHASE-4][doc-fix]` = `1c75e07` (satu baris). Konsekuensi: fase ini menjadi 2 commit — Penyimpangan dari G6 (satu commit per fase) direkam resmi di sini; alasan: menghindari amend berulang yang tak pernah bisa menuliskan hash dirinya sendiri (self-reference), dan memperbaiki dokumen lewat commit auditabel lebih dipilih daripada force/mendiamkan angka salah. Isi teknis tidak berubah.

---

# [PHASE-5] Hardware Gate P1 + Infrastruktur Verifikasi — EXECUTION PLAN
**Diterima:** 2026-10-20 (misi PHASE-5) | **Aturan:** G1–G7 + R1–R4 + **G8 baru** (tanpa rewrite riwayat: nol amend/rebase/reset; hash commit tidak pernah ditulis di dalam commitnya sendiri — dicatat di entri/commit berikutnya; koreksi = commit baru yang menyebut hash terkoreksi)

| Sub-fase | Isi | Gerbang keluar | Commit |
|---|---|---|---|
| 5A Higiene bukti | Verifikasi jumlah commit PHASE-4 via `git log --oneline -6` + status `32ba946`; README §6 footprint -> format terukur + `Analysis/reports/elf_footprint_history.txt` (211.672 B @ pre-Fase-2; 224.944 B @ pasca-Fase-3) | log & README cocok dengan `ls`/git | `[PHASE-5A]` |
| 5B HV-1 QM-01 | Jalur VT-d minimum di `asl-hal`: parser DMAR/ACPI (no_std, tanpa dependency baru — G1), root table + context entry + second-level page table per domain, pin=install leaf, QI invalidate, fault fault-record; host tests logika tabel; wiring runtime bila handover memuat ACPI; QEMU q35 `-device intel-iommu` smoke; exit criteria (1) boot serial VT-d terdeteksi+domain terpasang, (2) DMA IOVA unmapped DITOLAK ber-log, (3) transmit via IovaLease sukses | test + build 0/0 + serial log (hasil apa adanya, termasuk bila prasyarat VM kurang — tidak ada klaim melebihi bukti) | `[PHASE-5B]` |
| 5C HV-2 | Injeksi stack-overflow runtime end-to-end via jalur test khusus (bukan jalur produksi): cold-boot VM, overflow menembus bawah stack 512KB -> serial `STACK OVERFLOW (GUARD PAGE HIT)` + DR FSM Recording reason benar + halt aman; bukti serial | serial log teramati | `[PHASE-5C]` |
| 5D CI minimal | Pipeline: test 160+; build x86_64-none release 0/0; QEMU boot smoke sampai M6; summary; gagal = PR terblokir. Konfigurasi CI + skrip runner lokal output `Analysis/reports/ci_<tanggal>.log` | runner lokal lulus 3 stage | `[PHASE-5D]` |
| Akhir | Regenerasi QODER_FINAL_REPORT.md (PHASE-1..5); ADR-002 §5: tambahkan kategori **VERIFIED-ON-VM** (perbedaannya dengan VERIFIED-ON-HARDWARE dinyatakan eksplisit) | tanpa klaim melampaui kategori | (masuk sub-fase terkait) |

---

## [PHASE-5A] HIGIENE BUKTI — REPORT
**Status:** SELESAI — gerbang hijau, commit tunggal (hash dicatat pasca-commit per G8, lihat adendum di bawah entri ini pada commit berikutnya).
**Waktu:** 2026-10-20

### File berubah (docs/evidence-only; nol perubahan .rs)
| File | Perubahan |
|---|---|
| `README.md` | §6 footprint: angka hardcoded lama `211.672 B` ( basi pra-Fase-2 ) diganti format bukti-terukur: `224.944 B ≈ 220 KB, diukur 2026-10-20 di commit b17e572, lihat Analysis/reports/elf_footprint_history.txt` + aturan anti-regresi ("jangan kutip angka footprint tanpa baris bukti di file tsb"). |
| `Analysis/reports/elf_footprint_history.txt` | BARU, append-only: 211.672 B @ pra-Fase-2 (commit `4fead01`+FIX-1, tercatat via `[PHASE-1] 886425f`) dan 224.944 B @ pasca-Fase-3 (`b17e572`; artefak tetap sah karena Fase-4 dst docs-only) + metode pengukuran ulang. |
| `Analysis/reports/QODER_EXECUTION_LOG.md` | Entri ini (plan PHASE-5 sudah appended sebelumnya pada sesi ini). |

### Verifikasi (bukan asumsi)
- `git log --oneline -6` (dijalankan 2026-10-20): PHASE-4 = **3 commit** — `d25f283` [PHASE-4] utama, `1c75e07` [PHASE-4][doc-fix] koreksi referensi hash, `32ba946` [PHASE-4][doc-fix] adendum penyimpangan aturan satu-commit. **Resolusi konflik R3 vs REPORT:** R3 ("3") BENAR; narasi PHASE-4 REPORT yang mengimplikasikan 2 commit adalah keliru — `32ba946` statusnya = commit koreksi menyimpang yang secara eksplisit MENCATAT penyimpangan amend-insiden (bukan bagian dari commit utama). Sesuai G8, koreksi ini dibiarkan sebagai catatan baru, tidak menulis-ulang apa pun.
- Ukuran ELF aktual: `ls -la asl-kernel/target/x86_64-unknown-none/release/asl-kernel` → **224944 B** (build gerbang 2026-10-20 01:17, era `b17e572`; tidak berubah oleh docs-only Fase-4). Cocok dengan angka di user brief.
- Sumber historis 211.672 B: grep konfirmasi di log baris 14 (plan Fase-1, HEAD `4fead01`) + `ACCOUNTABILITY_FAULT_LOG.json` entri FIX-1.

### Gerbang (G4)
- `cargo test --workspace --lib`: **160 passed / 0 failed** (30 suite; hasil grep terverifikasi, tidak ada baris "N failed" dengan N>0).
- `cargo build --package asl-kernel-bin --target x86_64-unknown-none --release`: **0 error, 0 warning** (grep count baris error|warning = 0).

### Kriteria fase 5A
(a) Jumlah commit PHASE-4 terverifikasi + status `32ba946` teridentifikasi dan tercatat di log — PASS. (b) README §6 pakai format terukur + rujukan file bukti — PASS. (c) `elf_footprint_history.txt` dibuat dengan 2 baris history — PASS.

### Penyimpangan
- Tidak ada. Tidak ada amend/rebase/reset (G8 dipatuhi; hash commit 5A sengaja TIDAK ditulis di dalam commit/entri 5A-nya sendiri).

### Bloker
- Tidak ada.

### Commit
- `[PHASE-5A] ...` — staging eksplisit per file: `README.md`, `Analysis/reports/elf_footprint_history.txt`, `Analysis/reports/QODER_EXECUTION_LOG.md`. Log serial untracked milik aktor lain (asl_vm_serial*.log, qemu_serial*.log, vbox_check.log) TIDAK distage. Hash commit akan dicatat di adendum log ini pada commit `[PHASE-5B]` (G8).


---

## [PHASE-5B] HV-1 QEMU VT-d SMOKE (QM-01, VT-D-DRV-01) — REPORT
**Status:** SELESAI — ketiga kriteria keluar tercapai pada QEMU q35 `-device intel-iommu`; gerbang hijau; commit tunggal (hash commit ini TIDAK ditulis di dalam commit-nya sendiri per G8 — dicatat pada entri [PHASE-5C] berikutnya).
**Waktu:** 2026-10-20
**Kategori kenaikan (ADR-002 §5):** **DMA-ADDR-01 → VERIFIED-ON-VM** (bukan lagi sekadar MITIGATED/CODE). Kategori VERIFIED-ON-VM ditambahkan EKSPLISIT ke ADR-002 §5 pada commit ini karena belum ada sebelumnya.

### G8 — pemenuhan kewajiban hash PHASE-5A
Hash commit `[PHASE-5A]` = **`591d3a4`** (verifikasi: `git log --oneline -3` 2026-10-20: `591d3a4 [PHASE-5A] ...` adalah HEAD sebelum working tree PHASE-5B). Ini mencatat hash 5A yang dijanjikan pada laporan 5A ("Hash commit akan dicatat ... pada commit [PHASE-5B]").

### File berubah
| File | Perubahan |
|---|---|
| `asl-kernel/asl-hal/src/vtd.rs` | BARU (~900 baris, no_std): parser ACPI RSDP→XSDT/RSDT→DMAR (toleran layout u8-length legacy & u16-length QEMU), encoder root/context/second-level 3-level, QI descriptor, fault record FRCD, walk murni `sl_walk` (model host) + `sl_walk_hw` (mirror indeks >>30/>>21/>>12 QEMU), backend `VtdAddressSpace` (kontrak lease RND_08). 11 host test qp01..qp11. |
| `asl-kernel/asl-hal/src/lib.rs` | `pub mod vtd;` + re-export register map & tipe. |
| `asl-kernel/asl-kernel-core/src/vtd_boot.rs` | BARU: probe runtime fail-soft — scan RSDP (EBDA+window 0xE0000), parse DMAR→DRHD reg_base, baca VER/CAP/ECAP per-dword, bangun root+context(256 pass-through + devfn 0x08 SL 3-level)+SL PDP/PD/PT di .bss, program RTADDR→IQA→GCMD(TE\|SRTP\|QIE), verifikasi GSSTS, push QI invalidate+wait, preflight walk tabel nyata (mapped lolos + unmapped DITOLAK), TX-smoke lease. |
| `asl-kernel/asl-kernel-core/src/lib.rs` | wiring `vtd_boot::qm01_vtd_boot_probe(kernel_virt_base, kernel_phys_base)` setelah PCI Discovery. |
| `scripts/test_qm01_vtd_qemu.ps1` | BARU: harness QEMU q35 `-device intel-iommu,intremap=off,caching-mode=on` → serial COM1 + stderr QEMU ke Analysis/reports. |
| `Analysis/reports/qm01_vtd_serial_2026-10-20.log` | BARU: bukti serial cold-boot (ketiga kriteria). |
| `Analysis/reports/qm01_vtd_qemu_stderr_2026-10-20.log` | BARU: stderr QEMU — bukti residual benign (lihat Deviasi). |
| `docs/CANONICAL/ADR-002_CERTIFICATION_LADDER.md` | §5 tambah kategori VERIFIED-ON-VM + pernyataan eksplisit "≠ VERIFIED-ON-HARDWARE". |
| `audit/asl-core-kernel/REMEDIATION_PLAN.md` | §2 DMA-ADDR-01→VERIFIED-ON-VM, VT-D-DRV-01→VERIFIED-ON-VM, DA-02→VERIFIED-ON-VM; §5 HV-1→LULUS ON-VM; ringkasan OPEN→1. |
| `audit/asl-core-kernel/SECURITY_FINDINGS.md` | DMA-ADDR-01→VERIFIED-ON-VM; distribusi status diperbarui. |
| `Analysis/reports/elf_footprint_history.txt` | +1 baris 243.384 B (PHASE-5B tree, +18.440 B / +8,2% vs b17e572). |
| `README.md` | §6 footprint → 243.384 B ≈ 238 KB (PHASE-5B tree). |

### Iterasi debugging berbasis bukti (transparan, bukan klaim melebihi bukti)
Run pertama QM-01 (sebelum sesi ini) mencapai DMAR-parse/TE/DENY/TX tapi **CAP/ECAP=0** dan **QI NOT drained**. Akar masalah ditemukan empiris, bukan asumsi:
1. **Register map SALAH.** Ground truth = `intel_iommu_internal.h` QEMU upstream (di-grab via curl, bukan tebakan): RTADDR=0x20 (bukan 0x08), CAP=0x08, ECAP=0x10, GCMD=0x18, GSTS=0x1c, IQH=0x80, IQT=0x88, IQA=0x90. Setelah koreksi → **CAP=0x80D2008C222F06C6 ECAP=0xF02** (nyata, non-zero). Ini memperbaiki item yang sama (register map) — hit G5: attempt ke-2 pada register map berhasil, tidak perlu STOP.
2. **QI perlu GCMD.SRTP + GCMD.QIE, bukan hanya TE.** `vtd_handle_gcmd_write` QEMU: SRTP(bit30) me-latch root pointer, QIE(bit26) mengaktifkan QI. Tanpa keduanya, qi_enabled=false dan queue tak diproses. Setelah GCMD=TE\|SRTP\|QIE → **GSSTS.TES=1, RTPS=1, QIES=1**.
3. **Deskriptor IOTLB butuh granularitas GLOBAL (bit4).** stderr QEMU: "iotlb inv desc lo=0x2 (type mismatch: 0x0)" — `switch(lo & VTD_INV_DESC_IOTLB_G)` jatuh ke default tanpa bit 1<<4. Koreksi lo=0x12.
4. **Deskriptor WAIT butuh salah satu SW/IF/FN.** stderr QEMU: "wait desc lo=5 (unknown type)" — `vtd_process_wait_desc` menolak SW=0/IF=0/FN=0. Koreksi lo=0x45 (FN=1, no-op valid per VT-d 7.10). Setelah keduanya → **QI DRAINED (IQH==tail)**.
5. **Context entry layout SALAH untuk QEMU 10.2.** stderr run insial: "invalid context entry lo=7 (level 2 not supported)". Ground truth: TT ada di **lo[3:2]** (bukan lo[2:1]); SSPTPTR = second-level ROOT pointer di **lo[63:12]**; AGAW di **hi[2:0]**; CAP.SAGAW QEMU hanya iklankan level 3/4 → **level 2 DITOLAK hardware**. Solusi: rantai SL **3-level** (PDP→PD→PT), AGAW=39-bit. Setelah koreksi, walk `sl_walk_hw` mirror indeks hardware dan lolos di atas memori tabel nyata.

### Gerbang (G4) — hasil terverifikasi
- `cargo test --workspace --lib`: **171 passed / 0 failed** (asl-hal naik 30→31 via qp11 walk 3-level; tidak ada regresi).
- `cargo build --package asl-kernel-bin --target x86_64-unknown-none --release`: **0 error, 0 warning**.
- ISO direbuild: 4.118.528 B.

### Kriteria keluar QM-01 (exit criteria misi 5B) — pada serial log cold-boot
1. **VT-d terdeteksi & domain per partisi terpasang** → `[ASL-VTD] DMAR OK: DRHD count=1 reg_base=0xFED90000`; `Domain installed: partition=7 BDF=00:08.0 TT=SL(3-level) root_phys=0xFEDB000 sl_pdp=0xFED7000 sl_pd=0xFEDD000 sl_pt=0xFEDA000`; `Global TE ... ENABLED (GSSTS.TES=1); RootPtr latched (RTPS)=1; QI enabled (QIES)=1` — **PASS**.
2. **DMA ke IOVA unmapped DITOLAK (bukti log fault)** → `[ASL-VTD] DENY-DIAG: unmapped IOVA 0x10080000 REJECTED by SL walk; fault record bytes0=0x10080000` — **PASS** (walk 3-level di atas tabel yang benar-benar ditulis).
3. **Transmit via IovaLease melalui jalur VT-d sukses** → `[ASL-VTD] TX-SMOKE OK: IovaLease iova=0x10000000 -> dma phys 0xFE4A000 via SL domain; validate_pin OK` — **PASS**.

### Deviasi
- Tidak ada perubahan MajorFrameSchedule, tidak ada tulis framebuffer tanpa validasi (G3 aman). Tidak ada dependency eksternal baru (G1 aman — parser ACPI/VT-d ditulis manual no_std). Tidak ada amend/rebase/reset (G8).
- Log serial actor lain (asl_vm_serial*.log, qemu_serial*.log, vbox_check.log) TIDAK distage.
- **stderr QEMU residual benign:** satu baris `vtd_root_entry_rsvd_bits_check: invalid root entry: hi=0xf000ff53...` muncul SEBELUM GCMD.SRTP me-latch root pointer yang benar (probe ACPI awal membaca entry mentah/0xFF saat RTADDR belum valid). Setelah SRTP + tulis tabel, entry jadi valid (dibuktikan TES/RTPS/QIES=1 dan DRAINED). Dicatat apa adanya; tidak diklaim nol-stderr.

### Bloker
Tidak ada.

### Commit
`[PHASE-5B] ...` — staging eksplisit per file (13 path di tabel di atas). Hash commit ini dicatat pada entri [PHASE-5C] berikutnya (G8).

## [PHASE-5C] HV-2 VM COLD-BOOT STACK-OVERFLOW INJECTION — REPORT
**Status:** SELESAI — ketiga kriteria keluar (serial verbatim + DR FSM Recording dengan reason benar + safe halt) TERAMATI pada QEMU q35 cold-boot dengan artefak identik produksi; gerbang hijau; commit tunggal (hash commit ini TIDAK ditulis di dalam commit-nya sendiri per G8 — dicatat pada entri [PHASE-5D] berikutnya).
**Waktu:** 2026-10-20
**Kategori kenaikan (ADR-002 §5):** **DA-07/HV-2 → VERIFIED-ON-VM** (deteksi + jalur halt dibuktikan on-VM; VERIFIED-ON-HARDWARE tetap terbuka).

### G8 — pemenuhan kewajiban hash PHASE-5B
Hash commit `[PHASE-5B]` = **`965c3e2`** (`git log --oneline -3` 2026-10-20: `965c3e2 [PHASE-5B] HV-1 QM-01 VT-d smoke ...` = HEAD sebelum working tree PHASE-5C). Kewajiban ini dijanjikan pada entri 5B ("Hash commit ini dicatat pada entri [PHASE-5C] berikutnya").

### File berubah
| File | Perubahan |
|---|---|
| `asl-kernel/asl-kernel-core/src/lib.rs` | (a) NOTIFIKASI PRODUKSI: crash record stack-overflow kini mencetak bukti FSM — `pre_state/post_state/reason` latch segera setelah `notify_fault_safe`, memakai nilai return latch yang selama ini diabaikan; (b) BARU `hv2_stack_overflow_injection_probe()` + descent, seluruhnya digerbang `#[cfg(feature = "test-hv2-stack-overflow")]` sehingga TIDAK ADA di artefak release nominal; dipanggil SETELAH `enforce_stack_guard_page` (jalur identik kondisi produksi). |
| `asl-kernel/asl-hal/src/x86_fault.rs` | #PF (vektor 14) dipindah ke **IST1** (sebelumnya IST=0). Akar masalah fungsional: saat RSP sudah di bawah dasar stack, CPU tidak bisa menulis frame exception ke stack yang rusak sehingga #PF berubah menjadi #DF dan vektor/CR2 penyebab HILANG. |
| `asl-kernel/asl-kernel-core-secondary/src/lib.rs` | `safe::latch_reason()` (accessor reason terakhir yang terekam) + 1 host test signature fault HV-2 (#PF v14 -> reason MemoryIntegrity -> Recovery). |
| `asl-kernel/asl-kernel-core/Cargo.toml`, `asl-kernel/asl-kernel-bin/Cargo.toml` | fitur `test-hv2-stack-overflow` (pola sama dengan `test-gate3d`; default tetap kosong). |
| `scripts/test_hv2_stack_overflow_qemu.ps1` | BARU: harness build-ber-fitur + master ISO HV-2 + cold-boot QEMU q35 headless + poll marker `GUARD PAGE HIT`; menghapus log run sebelumnya agar stderr selalu segar. |
| `Analysis/reports/hv2_stack_overflow_serial_2026-10-20.log` | BARU: bukti serial cold-boot 47 baris (exit criteria 1-3). |
| `Analysis/reports/hv2_qemu_stderr_2026-10-20.log` | BARU: stderr QEMU = 0 byte (tidak ada galat perangkat). |
| `Analysis/reports/nominal_regression_serial_2026-10-20.log` | BARU: bukti anti-regresi — ISO release nominal (tanpa fitur) tetap capai M6 + prompt `ASL-OS>` tanpa exception palsu setelah perubahan IST1. |
| `audit/asl-core-kernel/REMEDIATION_PLAN.md` | §2 DA-07 -> VERIFIED-ON-VM; §5 HV-2 -> LULUS ON-VM dengan bukti + catatan jujur soal screenshot; ringkasan §1 +1 baris. |
| `audit/asl-core-kernel/SECURITY_FINDINGS.md` | Distribusi status: baris VERIFIED-ON-VM untuk jalur DA-07/HV-2 (register 16 temuan tidak berubah — DA-07 adalah baris rencana remediasi). |
| `Analysis/reports/elf_footprint_history.txt` | +1 baris 243.568 B (tree PHASE-5C build DEFAULT, +184 B / +0,08% vs 965c3e2). |
| `README.md` | §6 footprint -> 243.568 B ≈ 238 KB (tree PHASE-5C). |

### Iterasi debugging berbasis bukti (transparan)
1. **Marker serial sudah benar, jadi teksnya tidak diubah.** Temuan: `early_serial_write_str("STACK OVERFLOW (GUARD PAGE HIT)\n")` sudah ada sejak Phase-2 (lib.rs:790) dan terverifikasi ada di HEAD `965c3e2` — kriteria 1 terpenuhi tanpa perubahan string.
2. **Rekursi frame 64 KiB TIDAK menembus stack.** Bukti (diagnostik sementara): `rsp` identik 0xFFFFFFFF8019B050 pada depth 0..512, jadi LLVM melipat frame lokal besar (satu-satunya akses byte terlipat ke register) dan RSP tak pernah bergerak; keluaran `UNEXPECTED: recursion returned`. Ganti bentuk: penurunan RSP eksplisit lewat `asm` (`sub rsp, reg`) yang tidak bisa dilipat.
3. **Warning `unconditional_recursion` pada build fitur** -> diberikan syarat berhenti yang secara aritmetika tak terjangkau (guard selalu menembus lebih dulu); rekursi menjadi kondisional dan build fitur kembali 0 warning.
4. **Overflow dilaporkan sebagai #DF, bukan #PF (vektor 8, reason=4).** Akar: #PF terdaftar dengan IST=0, sehingga saat RSP sudah di bawah stack frame exception tidak dapat ditulis. Koreksi: #PF -> IST1. Setelah itu fault asli terlaporkan: `VECTOR: 0x000E`, `ERROR: 0x2` (supervisor write, not-present), `CR2` di halaman guard, reason=3.
5. **Satu kali injeksi lolos tanpa fault (`pushfq` LOLOS).** Pada varian sementara RSP mendarat di halaman yang TERPETAKAN; dilaporkan apa adanya, TIDAK diklaim sebagai gap proteksi. Varian final menentukan target secara deterministik dan mencetaknya SEBELUM injeksi: guard label == stack_bottom-4096 dengan `Paging proof: guard -> NOT MAPPED`, lalu store 8 byte ke `rsp-64` yang pasti di dalam halaman guard. Run final: #PF pada 0xFFFFFFFF801360F8. Observasi tambahan: guard label bergeser 1 halaman antar build ber-fitur (.bss tumbuh karena kode fitur) — karena itu alamat tidak pernah diasumsikan, selalu dibaca dari label + bukti terjemahan halaman.
6. **Screenshot tidak tersedia (dinyatakan jujur).** Harness memakai `-display none` (headless; tidak ada tooling tangkap layar di lingkungan ini), sehingga bukti yang dipakai adalah log serial COM1, bukan citakan layar. Tidak ada klaim screenshot palsu.

### Gerbang (G4) — hasil terverifikasi
- `cargo test --workspace --lib`: **172 passed / 0 failed** (naik 171 -> 172 via test signature latch HV-2; tidak ada regresi).
- `cargo build --package asl-kernel-bin --target x86_64-unknown-none --release` (DEFAULT, tanpa fitur): **0 error, 0 warning**. Build ber-fitur `test-hv2-stack-overflow` juga 0 error / 0 warning.
- ISO release nominal diregenerasi: 4.118.528 B, SHA256 `330C953E70B1E356904E42D1BD6B09CF91B95EAE80DCC1DEDF60C9C9F40E5C1C`. Artefak HV-2 terpisah: `target/hv2-stack-overflow.iso` 4.118.528 B, SHA256 `8f6fcd1635769d7d54f582c11b1c52a3eb2526b60feb9de3f0e88a70da2f7792` (tidak distage — artefak build).
- Anti-regresi: ISO nominal cold-boot tetap M0 -> M6 -> `ASL-OS>` tanpa exception (log bukti di tabel atas).

### Kriteria keluar HV-2 (exit criteria misi 5C) — dari `hv2_stack_overflow_serial_2026-10-20.log`
1. **Serial mencetak persis `STACK OVERFLOW (GUARD PAGE HIT)`** -> baris verbatim hadir setelah banner exception `VECTOR: 0x000E (#PF - Page Fault)`, `CR2: 0xFFFFFFFF801360F8` (di dalam guard), `RSP: 0xFFFFFFFF80136100 < stack_bottom 0xFFFFFFFF80137000` — **PASS**.
2. **DR FSM ber-status Recording dengan reason benar** -> `[ASL-HV2] DR FSM latch: pre_state=1 post_state=3 reason=3 (2=Recording,3=Recovery; 3=MemoryIntegrity)`. pre_state=1 (Armed) adalah state SESABELUM notify; transisi nyata ARMED -> Recording -> Recovery, reason=3 = REASON_MEMORY_INTEGRITY untuk #PF; pra-injeksi tercatat `state=1 count=0` dan `Paging proof: guard NOT MAPPED` — **PASS** (catatan formulasi: latch MELEWATI Recording lalu terminal di Recovery, jadi serial menampilkan bukti keduanya, bukan status Recording yang tahan-lama).
3. **Safe halt teramati** -> `[ASL-CORE] Secondary kernel notified. Controlled Halt Active.` adalah baris TERAKHIR log; dispatcher `cli; hlt` dan QEMU `-no-reboot -no-shutdown` tidak menghasilkan episode boot kedua (log memuat tepat satu sekuens M0 -> crash) — **PASS**.

### Deviasi
- Tidak ada dependency eksternal baru (G1); tidak ada perubahan MajorFrameSchedule; tidak ada tulis framebuffer tanpa validasi (G3). Tidak ada amend/rebase/reset --hard; koreksi selalu commit baru (G8).
- **Perubahan kode produksi di luar teks marker:** penempatan #PF pada IST1. Diizinkan karena misi 5C eksplisit mengizinkan perubahan kode jalur stack-overflow pada fase ini, dan tanpanya kriteria 1-2 tidak dapat tercapai (overflow murni selalu terdistorsi menjadi #DF). Dampaknya meluas ke semua #PF kernel — dibuktikan tidak menimbulkan regresi boot nominal.
- Log serial actor lain (asl_vm_serial*, qemu_serial*, qemu_qm01_vtd_* dan qemu_hv2_* kanonik hasil regenerasi harness, vbox_check.log) TIDAK distage; bukti yang di-commit adalah SALINAN bertanggal.
- Satu run perantara melaporkan "store ke guard LOLOS tanpa #PF"; tidak dijadikan temuan proteksi karena penyebabnya target injeksi yang salah (halaman terpetakan), dan varian final membuktikan fault.

### Bloker
Tidak ada.

### Commit
`[PHASE-5C] ...` — staging eksplisit per file (14 path: 6 sumber/build + 1 harness + 3 bukti + 3 register/README + log eksekusi ini). Hash commit ini dicatat pada entri [PHASE-5D] berikutnya (G8).

## [PHASE-5D] CI MINIMAL PER PUSH/PR — REPORT (2026-10-20)

Kontrak fase: pipeline minimal per push/PR berisi (1) `cargo test --workspace --lib`; (2) build `x86_64-unknown-none --release` **0 error / 0 warning** wajib; (3) QEMU cold-boot smoke **harus mencapai M6**; (4) ringkasan. Pipeline merah = tidak boleh merge. Konfigurasi masuk repo + runner lokal dengan keluaran `Analysis/reports/ci_<tanggal>.log`.

### G8 — pemenuhan kewajiban hash PHASE-5C
**Hash commit `[PHASE-5C]` = `2b6193a`** — "HV-2 VM cold-boot stack-overflow injection ... -> DA-07/HV-2 VERIFIED-ON-VM" (diverifikasi `git log --oneline`: `2b6193a` di atas `965c3e2` [PHASE-5B]). Hash ini ditulis di entri BERIKUTNYA, bukan di dalam commit 5C; tidak ada amend/rebase/reset di branch misi.

### File berubah (docs + CI tooling; nol perubahan `.rs`, nol perubahan CHECKLIST_STATE.json)

| Path | Status | Catatan |
|---|---|---|
| `scripts/ci_local.ps1` | baru | runner lokal fail-closed 4 langkah; keluaran ASCII ke `Analysis/reports/ci_<tanggal>.log`; transcript per langkah `…log.step{1,2,3_iso}.log` |
| `.github/workflows/asl-ci-minimal.yml` | baru | workflow hosted untuk langkah [1]-[2] saja; komentar kepala menyatakan alasan langkah [3]-[4] tidak bisa hosted + larangan env `RUSTFLAGS` |
| `Analysis/reports/ci_2026-10-20.log` | baru (bukti) | ringkasan pipeline HIJAU 4/4, stamp commit `2b6193a+dirty` |
| `Analysis/reports/qemu_ci_smoke_serial.log` | baru (bukti) | serial 55 baris cold-boot q35 hasil langkah [3] — capai `M6: Kernel Cold Boot Complete` + prompt `ASL-OS>` |
| `README.md` | ubah | §6: blok perintah CI + tabel cakupan hosted vs lokal + kebijakan "pipeline merah = tidak boleh merge" + catatan ISO tidak byte-reproducible; §7: koreksi nuansa `.gitignore` (`*.log` dikecualikan TETAPI `Analysis/reports/*.log` di-include-ulang) |

### Gerbang (G4) — hasil terverifikasi, bukan asumsi
- **(a) `cargo test --workspace --lib`** → **172 passed / 0 failed** (total naik dari 171 ke 172 karena test latch 5C; angka tercantum di `ci_2026-10-20.log` baris `[1/4]`).
- **(b) `cargo build --package asl-kernel-bin --target x86_64-unknown-none --release`** → **0 error / 0 warning**, ELF **243.568 B** (identik dengan entri `elf_footprint_history.txt` [PHASE-5C]; fase ini tidak mengubah kode sehingga tidak ada baris footprint baru — aturan "kutip angka = ada baris sumber" tetap terpenuhi).
- Runner lokal dieksekusi penuh sampai tree final fase ini: **4/4 PASS** (`KEPUTUSAN: Pipeline HIJAU`), stamp commit `2b6193a+dirty`. Langkah [3] memakai ISO 4.118.528 B SHA256 `6DDF4E6C98EDDB55AA1F6CEBF27842BD8AA034383B040C735F1514C68ECD94BF`. Run pertama (sebelum README §6/§7 diedit) mencatat SHA256 `2B055DC2…464F49` dengan ukuran byte yang sama — perbedaan hash ini adalah bukti langsung bahwa build ISO tidak byte-reproducible (xorriso menanam volume timestamp), sehingga hash yang sah dikutip hanyalah hash yang tercantum di artefak `ci_2026-10-20.log` yang di-commit.

### Kriteria keluar misi 5D
1. **Config CI masuk repo** → `PASS` (`.github/workflows/asl-ci-minimal.yml` ter-track; tidak ada dependensi eksternal baru, G1 tetap berlaku — hanya file YAML + skrip PowerShell yang sudah memakai toolchain yang ada).
2. **Runner lokal dengan output `Analysis/reports/ci_<tanggal>.log`** → `PASS` (file ASCII/CRLF, bisa dibaca `file` sebagai "ASCII text"; hasil run nyata tertinggal di repo sebagai bukti).
3. **Gerbang (1) test, (2) build 0/0, (3) QEMU smoke sampai M6, (4) ringkasan** → `PASS 4/4` dengan bukti serial.
4. **"Pipeline merah = tidak boleh merge" dinyatakan di dokumen** → `PASS` (README §6 tabel + bullet kebijakan merge; §7 mengikat CI ke gerbang hygiene; `ci_local.ps1` mencetak kalimat yang sama dan exit code non-zero pada langkah gagal — fail-closed, bukan hanya teks).

### Deviasi dan batas kejujuran
- **Workflow hosted BELUM dieksekusi.** Branch `security/asl-v1.0.2-audit` belum di-push dan lingkungan ini tidak punya access runner GitHub Actions, jadi klaim untuk `.github/workflows/asl-ci-minimal.yml` adalah **CODE-COMPLETE (config authored, belum dijalankan di hosted runner)**, bukan VERIFIED. Langkah [1]-[2]-nya adalah terjemahan literal dari dua gerbang G4 yang sudah dibuktikan hijau lewat `ci_local.ps1` pada host yang sama.
- Validasi YAML yang tersedia = pembacaan struktur manual; `pyyaml` tidak terpasang di host dan tidak dipasang (aturan: instalasi tool oleh user), sehingga parse machine-over belum dilakukan.
- `ci_local.ps1` menjalankan test host dengan `--target x86_64-pc-windows-msvc` eksplisit; G4(a) menulis `cargo test --workspace --lib` tanpa `--target`. Keduanya ekuivalen di repo ini (config default target host), dan jalur yang dipakai workflow hosted adalah bentuk tanpa `--target` — angka 172 berasal dari run bertarget eksplisit.
- Transcript per-langkah (`ci_2026-10-20.log.step1.log` dsb., ±23 KB) sengaja TIDAK di-stage: keluaran wajib misi adalah `ci_<tanggal>.log`, dan bukti runtime yang di-stage dibatasi pada serial QEMU (55 baris) agar repo tidak berisi dump toolchain.
- Log harness milik aktor lain / run sebelumnya (`qemu_serial*.log`, `qemu_qm01_vtd_*`, `qemu_hv2_*`, `asl_vm_serial*`, `vbox_check.log`) tidak di-stage; hanya artefak yang fase ini hasilkan.
- Ditemukan angka usang di **README §2** tabel status: "**128 unit tests passed**" dan ISO "**3.917.824 bytes**". Sumber kebenaran saat ini = 172 test dan 4.118.528 B (keduanya terbukti di `ci_2026-10-20.log`). Tidak dikoreksi di commit 5D agar fase tetap scoped ke CI; **dibawa ke GERBANG AKHIR** bersama regenerasi `QODER_FINAL_REPORT.md`. Dicatat di sini supaya tidak hilang.
- Tidak ada perubahan `.rs` sehingga tidak ada risiko regresi; namun build+test+boot tetap dijalankan penuh (bukan disimpulkan dari "docs-only").

### Bloker
Tidak ada. Tidak ada kegagalan beruntun (G5 tidak terpicu) pada fase ini.

### Commit
`[PHASE-5D] <ringkasan>` — staging eksplisit per file (6 path: workflow hosted, `scripts/ci_local.ps1`, `Analysis/reports/ci_2026-10-20.log`, `Analysis/reports/qemu_ci_smoke_serial.log`, `README.md`, dan entri log ini sendiri). Hash commit fase ini dicatat pada entri GERBANG AKHIR berikutnya (G8).

## GERBANG AKHIR (FINAL GATE) — REPORT (2026-10-20)

Kontrak: "160+ tests PASS, build 0/0, regenerasi QODER_FINAL_REPORT.md yang mencakup PHASE-1..5 dengan kategori VERIFIED-ON-VM dinyatakan eksplisit BUKAN VERIFIED-ON-HARDWARE."

### G8 — pemenuhan kewajiban hash PHASE-5D
**Hash commit `[PHASE-5D]` = `6353c77`** — "CI minimal per push/PR: scripts/ci_local.ps1 fail-closed 4-langkah ..." (diverifikasi `git show --stat 6353c77`: 6 file, +348/−2). Tidak ada amend/rebase/reset; entri ini adalah commit/entri berikutnya yang mencatat hash fase sebelumnya, sesuai G8.

### Gerbang final — hasil TERUKUR (bukan disalin dari entri lama)
`scripts/ci_local.ps1` dijalankan penuh pada tree final (setelah README/register/laporan diedit), **4/4 PASS**:
- `[1/4]` **172 host tests passed / 0 failed** — syarat "160+ tests PASS" terpenuhi (margin +12).
- `[2/4]` build `--target x86_64-unknown-none --release` **0 error / 0 warning**, ELF **243.568 B**.
- `[3/4]` QEMU q35 cold-boot smoke: **M6 tercapai + prompt `ASL-OS>`** (serial 55 baris, stderr QEMU 0 baris), ISO 4.118.528 B SHA256 `BB97F25ED3E5DBEE44C53FD179586B0A6BE0230C1E1AC85E8E5111C723ACC959`.
- `[4/4]` ringkasan `KEPUTUSAN: Pipeline HIJAU`, stamp commit `6353c77+dirty`.
Artefak yang ter-commit: `Analysis/reports/ci_2026-10-20.log` (run gerbang akhir). Serial `qemu_ci_smoke_serial.log` dihasilkan ulang oleh runner dan **identik byte** dengan yang sudah ter-commit, sehingga tidak menjadi delta — bukti boot deterministik.

### Catatan hash ISO (koreksi append-only terhadap entri [PHASE-5D] di atas)
Entri `[PHASE-5D]` mengutip SHA256 `6DDF4E6C…` dari run pertama. Artefak `ci_2026-10-20.log` yang kini ter-commit berisi run terakhir dengan SHA256 `BB97F25E…`, **ukuran byte identik 4.118.528**. Keduanya benar untuk run-nya masing-masing: build ISO tidak byte-reproducible (xorriso menanam volume timestamp). Entri lama tidak ditulis-ulang (R1); koreksi berupa entri baru yang menyebut nilai lama, sesuai G8. Hash ISO yang sah dikutip hanyalah yang ada di artefak ter-commit.

### File berubah di gerbang akhir (nol `.rs`, nol CHANGE terhadap CHECKLIST_STATE.json)
| Path | Perubahan | Alasan |
|---|---|---|
| `Analysis/reports/QODER_FINAL_REPORT.md` | diregenerasi total (PHASE-1..5 + gerbang akhir) | exit criteria misi; versi lama hanya sampai PHASE-4 dan SALAH LABEL: mendefinisikan "VERIFIED-ON-HARDWARE" sebagai bukti VM |
| `README.md` §2 | angka test 128 → **172/172** (dengan riwayat terukur 128→149→160→171→172); ukuran ISO 3.917.824 → **4.118.528 B** + catatan non-reproducibility; **+3 baris status baru** (IOMMU/VT-d VERIFIED-ON-VM, stack-overflow VERIFIED-ON-VM, CI minimal) | G7 — angka wajib dari sumber kebenaran; entri FALSE_CLAIM lama mensyaratkan "patok 128 sampai terukur ulang", syaratnya kini terpenuhi |
| `README.md` §4 | rekap register jadi 12 CLOSED / 2 VERIFIED-ON-VM / 1 PARTIAL-MITIGATED-CODE / 1 OPEN; item `DMA-ADDR-01` dan `VT-D-DRV-01` tidak lagi tertulis MITIGATED-CODE/PLANNED | sinkron dengan `SECURITY_FINDINGS.md` §1–§2 pasca-5B |
| `audit/asl-core-kernel/SECURITY_FINDINGS.md` | baris tabel `VT-D-DRV-01` **PLANNED → VERIFIED-ON-VM (PHASE-5B)** dengan catatan koreksi eksplisit di selnya | drift internal register: rekap §2 sudah VERIFIED-ON-VM, baris tabel belum |
| `docs/ROADMAP/ACCOUNTABILITY_FAULT_LOG.json` | **+2 entri** `fault_history` (PHASE-5C `SCOPE_DEVIATION_PRODUCTION_CODE` untuk #PF→IST1; GERBANG AKHIR `DOCUMENTATION_DRIFT_CORRECTED`); JSON tervalidasi parser, `violations_recorded` tetap 0 | akuntabilitas charter: deviasi produksi dan koreksi drift wajib tercatat tertulis |
| `Analysis/reports/ci_2026-10-20.log` | di-refresh dari run gerbang akhir (stamp `6353c77+dirty`, ISO SHA256 `BB97F25E…`) | bukti harus sesuai tree yang di-commit |
| `Analysis/reports/qemu_ci_smoke_serial.log` | dijalankan ulang oleh runner, **isi identik** dengan versi ter-commit PHASE-5C/5D sehingga tidak muncul sebagai delta (`git status` bersih untuk path ini) | bukti boot deterministik — tidak ada perubahan serial antar-run |
| koreksi laporan lama | `4f55c87` diffstat ditulis **+1161/−96** (terukur `git show --stat`), laporan lama menulis +1145 | G7 |

### Batas klaim yang dijaga (disiplin kategori)
* **VERIFIED-ON-HARDWARE = 0 item.** Semua bukti 5B/5C berhenti di **VERIFIED-ON-VM**; setiap baris status menyebut sisa pekerjaan hardware secara eksplisit.
* Workflow hosted `.github/workflows/asl-ci-minimal.yml` berstatus **CODE-COMPLETE**, bukan VERIFIED: branch belum di-push dan tidak ada access runner GitHub Actions dari lingkungan ini; parser YAML machine (pyyaml) tidak terpasang dan tidak dipasang — validasi yang dilakukan = pembacaan struktur manual.
* **Tidak ada screenshot** untuk HV-1/HV-2 (QEMU headless `-display none`, tidak ada tooling tangkap layar). Klaim bersandar pada log serial saja; ini dinyatakan di laporan akhir, bukan ditutup-tutupi.
* Baris register yang menulis "171/171 host test" (PHASE-5B) adalah angka **as-of run saat itu** dan sengaja tidak ditulis-ulang; total terkini 172/172 tercatat di README §2, laporan akhir, dan artefak CI.
* Transcript per-langkah CI (`ci_*.log.step*.log`) dan log harness aktor lain/run sebelumnya (`qemu_serial*`, `qemu_qm01_vtd_*`, `qemu_hv2_*`, `asl_vm_serial*`, `vbox_check.log`) tidak di-stage.

### Deviasi
Tidak ada pelanggaran G1–G8. Deviasi yang ada (perubahan produksi #PF→IST1 di 5C, workflow hosted yang tak bisa menutup langkah 3–4, absennya screenshot) seluruhnya sudah tercatat tertulis di fault log dan/atau entri log fase masing-masing. Tidak ada kegagalan beruntun, jadi G5 tidak terpicu.

### Bloker
Tidak ada.

### Commit
`[PHASE-5] GERBANG AKHIR …` — staging eksplisit per file (6 path berubah: laporan akhir, log ini, artefak CI, README §2+§4, SECURITY_FINDINGS, ACCOUNTABILITY_FAULT_LOG). Hash commit ini TIDAK ditulis di dalam commitnya (G8); entri berikutnya (jika ada) atau ringkasan akhir yang mencatatkannya.

## [PHASE-6A] PARSIAL — PUSH BRANCH + PEMBUKTIAN HOSTED CI — REPORT (2026-10-20)

Misi baru (PHASE-7 MILESTONE_03 CLOSEOUT) diterima dengan **preconditional hard gate**: "PHASE-6A tuntas: branch ter-push, PR merged, hosted CI hijau (VERIFIED-ON-CI), QODER_EXECUTION_LOG ditutup, rilis v1.1.0 terpublikasi. Jika belum — STOP, selesaikan 6A dulu."

### G8 — pemenuhan kewajiban hash GERBANG AKHIR
**Hash commit `[PHASE-5] GERBANG AKHIR` = `8b829d8`** (diverifikasi `git log --oneline -3`: `8b829d8` di atas `6353c77` [PHASE-5D]). Hash PHASE-6A sendiri TIDAK ditulis di dalam commit-nya — dicatat pada entri berikutnya bila ada.

### Audit precondition sebelum eksekusi (terukur, bukan diasumsikan)
| Prasyarat 6A | Kondisi saat audit | Sumber |
|---|---|---|
| Branch ter-push | **TIDAK** — lokal ahead **14 commit** dari tip `origin` (`95bac29`, state pra-misi); tidak ada upstream | `git rev-list --count 95bac29..HEAD` = 14; `git rev-parse --abbrev-ref @{u}` = fatal; `git ls-remote --heads` |
| PR merged untuk kerja ini | **TIDAK** — hanya PR #1 & #2, keduanya MERGED 2026-10-17 (state pra-misi) | `gh pr list --state all` |
| Hosted CI hijau | **TIDAK** — `gh run list` hanya 5 run "Code scanning AI findings"; **nol run `asl-ci-minimal`** karena file workflow belum ter-push | `gh run list --limit 5` |
| Execution log ditutup | **TIDAK** — entri terakhir GERBANG AKHIR; `grep -c PHASE-6` = 0 | `Analysis/reports/QODER_EXECUTION_LOG.md` |
| Rilis v1.1.0 | **TIDAK** — tag lokal `v1.0.0`,`v1.0.1`; release GitHub `v1.0.0`,`v1.0.1-alfa` (dua Pre-release) | `git tag -l`; `gh release list` |
| T0 RND_09 | **BELUM ADA** — `docs/RND/` berhenti di RND_08 | `ls docs/RND/` |
**Keputusan: PHASE-7 (T0..T4) TIDAK dimulai.** Otorisasi pemilik repo yang diberikan: **"Push branch saja, tanpa PR/release"**; kebijakan PR: "biarkan terbuka, jangan di-merge" (tidak relevan karena PR tidak dibuat).

### Aksi yang dijalankan (review dulu, lalu push)
1. **Review isi outgoing** (protokol keamanan): `git diff --name-only 95bac29..HEAD` = **61 file, +9105/−152** — dokumen governance/register/bukti + kode kernel; **nol** path `tools/`, `AGENT.md`, `GEMINI.md`, ISO, ELF, atau `.env`. Scan konten keluar: `git diff | grep -iE "gho_|api.?key|secret|password|token=|Administrator|Users\\\\"|C:\\Users"` → **0 hit** (tidak ada rahasia/personal path yang ikut keluar).
2. **Push**: `git push origin security/asl-v1.0.2-audit` → `95bac29..8b829d8` (**fast-forward**, tanpa `--force`, tanpa `-u` agar tidak mengubah git config). Verifikasi tip remote: `git ls-remote --heads origin security/asl-v1.0.2-audit` = `8b829d8bb5d7ff958c3875638888952573579f05` = HEAD lokal. **G8 terpenuhi: nol rewrite riwayat.**
3. **Pantau hosted CI**: workflow `asl-ci-minimal` terpicu oleh push → run `39399083819`; `gh run watch --exit-status` selesai `WATCH_EXIT=0`; `gh run view --json` = `conclusion: success`, `event: push`, `headSha: 8b829d8…`, job `test-and-build` 20 s, semua step ✓.
4. **Verifikasi log mentah, bukan hanya status hijau** (`gh run view 39399083819 --log`, 841 baris, disaring ke artefak ter-commit `Analysis/reports/ci_hosted_39399083819_2026-10-20.log`):
   - `[1/2] cargo test --workspace --lib` → **30 baris `test result: ok`, TOTAL TERHITUNG 172 passed / 0 failed** di `ubuntu-latest` (sama dengan host Windows).
   - `[2/2] cargo build --package asl-kernel-bin --target x86_64-unknown-none --release` → **14 baris Compiling/`Finished`** (membantah dugaan "langkah di-skip cache"; durasi 20 s wajar untuk 31 crate `no_std` kecil yang di-paralel), `Finished \`release\` profile [optimized] target(s) in 1.96s`, **nol warning / nol error** di langkah ini.
   - Bukti footprint: `-rwxr-xr-x 2 runner runner 243560 Oct 20 09:13 target/x86_64-unknown-none/release/asl-kernel`.

### Temuan baru dari hosted run (dinyatakan jujur)
* **RISIKO YANG SAYA NYATAKAN SEBELUM PUSH TERBUKTI TIDAK TERJADI:** rustflags target dari `asl-kernel/.cargo/config.toml` (`-T linker.ld`, `-z max-page-size=0x1000`, `relocation-model=static`, `code-model=kernel`) **berhasil melakukan link di Linux juga** — sebelumnya hanya dibuktikan di Windows. Tidak ada penyesuaian linker yang perlu dibuat.
* **DELTA UKURAN BINER PER PLATFORM:** ELF Linux **243.560 B** vs Windows **243.568 B** (selisih 8 B; source/rustflags/target/profil identik). README §6 tetap mengutip 243.568 B karena itulah biner yang di-boot di VM lokal; baris riwayat baru ditambahkan ke `elf_footprint_history.txt` (append-only) supaya angka tidak tercampur.
* **Warning informatif runner** (bukan kegagalan): `actions/checkout@v4` masih menarget Node.js 20 (deprecated, dipaksa jalan di Node 24); label `ubuntu-latest` bermigrasi ke Ubuntu 26 mulai 2026-10-19. Dicatat untuk nanti, tidak diubah di fase ini.
* **Profil `test` hosted memuat warning non-blocking** (mis. penamaan variant di `asl-ont-gpon`). Gerbang 0-warning misi ini berlaku untuk build **release** x86_64-unknown-none, bukan profil test — ditegakkan oleh parsing langkah `[2/4]` `ci_local.ps1` (hasil 0/0).

### Gerbang (G4) pada tree ini — diverifikasi ulang, bukan disimpulkan
* `cargo test --workspace --lib` (host Windows, dijalankan lagi setelah edit README/register/laporan): **172 passed / 0 failed**.
* `cargo build --package asl-kernel-bin --target x86_64-unknown-none --release`: **0 warning / 0 error**, ELF **243.568 B**.
* Tidak ada perubahan `.rs` pada fase ini → risiko regresi nol; pembuktian hosted di atas memakai `headSha 8b829d8` (tree GERBANG AKHIR), commit docs PHASE-6A berikutnya hanya menyentuh dokumen/bukti.

### File berubah (docs/evidence-only, nol `.rs`)
`README.md` (§2 baris CI → VERIFIED-ON-CI + baris VM; §6 bukti hosted + delta ELF; §7 status remote + batas "LOCAL-ONLY"), `Analysis/reports/QODER_FINAL_REPORT.md` (kategori **VERIFIED-ON-CI** didefinisikan, baris PHASE-6A di §1, §6/§7 disinkronkan, gerbang prasyarat PHASE-7 dinyatakan belum terpenuhi), `Analysis/reports/elf_footprint_history.txt` (+1 baris Linux), **`Analysis/reports/ci_hosted_39399083819_2026-10-20.log` (baru)**, `Analysis/reports/QODER_EXECUTION_LOG.md` (entri ini).

### Deviasi & hal yang TIDAK saya lakukan
* **PR tidak dibuat, tidak ada merge, tidak ada tag/release** — sesuai otorisasi "push branch saja".
* `python tools/Sub_Assist.py --hygiene` **MASIH MELAPORKAN SAMPAH**: 9 file `.log` di root repo (`boot_final.log`, `boot_gdt_test.log`, `boot_nominal.log`, `boot_tss_test.log`, `boot_working.log`, `test_de_output.log`, `test_de_output2.log`, `test_de_output3.log`, `trace_gt.log`). Files itu ter-ignore git dan **bukan dibuat agen ini**, jadi tidak dihapus (menyingkirkan kerja pihak lain tanpa izin = risiko kehilangan bukti). README §7 mensyaratkan hygiene 100% bersih → **kondisi nyata belum 100%**; dicatat sebagai deviasi terbuka milik pemilik repo, bukan disembunyikan.
* Log mentah 841 baris tidak di-commit utuh (memuat URL workspace runner); yang di-commit ekstrak 66 baris ASCII berisi baris bukti. Raw sementara ada di `logs/` (ter-ignore).
* Tidak ada perubahan `CHECKLIST_STATE.json` (G2), tidak ada perubahan `MajorFrameSchedule` (G3), tidak ada dependency baru (G1). G5 tidak terpicu (nol kegagalan).

### Bloker untuk melanjutkan ke PHASE-7
Precondition 6A masih **belum tuntas** pada 3 item yang membutuhkan keputusan/otorisasi pemilik repo: (1) PR + merge, (2) penutupan `QODER_EXECUTION_LOG` sebagai bagian closeout, (3) tag + rilis `v1.1.0`. Tanpa itu, teks misi PHASE-7 mengharuskan STOP.

### Commit
`[PHASE-6A] <ringkasan>` — staging eksplisit per file (5 path: README, laporan akhir, riwayat footprint, artefak bukti hosted CI baru, log eksekusi ini). Hash fase ini dicatat pada entri berikutnya bila ada (G8).
