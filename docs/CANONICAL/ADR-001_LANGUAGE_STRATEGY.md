# ADR-001: STRATEGI BAHASA — RUST-PRIMARY

- **Status:** ACCEPTED (2026-10-20)
- **Decision Driver:** Hasil audit independen `Analysis/reports/DEEP_AUDIT_REPORT.md` §1 (kontradiksi dokumen C23 vs implementasi Rust) dan kebutuhan ketertelusuran DO-178C dokumen→kode.
- **Supersedes:** `asl_avionics_kernel_blueprint.md` §3 (pohon file C23) dan `asl_kernel_audit_roadmap.md` §5 baris "Bahasa Utama = C23" — kedua dokumen telah di-rebase mengikuti ADR ini pada 2026-10-20.

## 1. Keputusan
**Rust `#![no_std]` adalah bahasa implementasi utama dan kanonik ASL Core Kernel.** C/assembly hanya dipergunakan sebagai *jembatan vendor-toolchain per arsitektur* (IA-64, TriCore, DSP vendor, retro 8/16-bit) di belakang trait `asl-hal`, bukan sebagai bahasa core.

## 2. Justifikasi berbasis bukti (G7 — sumber kebenaran)
1. **31 workspace crate** telah terpasang dan terpelihara dalam Rust (`cargo metadata --no-deps --format-version 1`, 2026-10-20).
2. **172 unit test host PASS / 0 FAIL** pada 30 suite lib (`cargo test --workspace --lib`, 2026-10-20; disinkronkan di PHASE-6B-CLOSEOUT — angka ini dulunya **128** saat ADR ini ditulis era v1.0.1, lalu naik bertahap 128 → 149 → 160 → 171 → 172 sepanjang misi remediasi; sumber kebenaran `Analysis/reports/ci_2026-10-20.log` langkah [1/4] dan 11 run hosted `ci_hosted_*.log`).
3. **Bare-metal runtime verified:** ELF64 `x86_64-unknown-none` **243.568 B** (Windows; **243.560 B** di runner Linux — delta pautan 8 B per-platform, riwayat di `Analysis/reports/elf_footprint_history.txt`; angka **211.672 B** yang dulu tertulis di sini adalah footprint era v1.0.1) boot hingga `M6 Cold Boot Complete` + shell `ASL-OS>` — bukti terkuat: **run penerimaan G9 berjendela** `Analysis/reports/g9_acceptance_serial_2026-10-20.log` (2026-10-20 20:48, jendela QEMU terbuka, tanpa `-display none`), ditambah log era v1.0.1 `Analysis/reports/boot_working.log` yang sejak CLOSEOUT T5 memang berada di direktori yang disebut kalimat ini (sebelumnya masih di root repo sehingga rujukan ini salah alamat).
4. Ekosistem `no_std` matang: linker script sendiri, `.cargo/config.toml` per-target, panic handler, global allocator, inline asm MMIO/serial/IDT — semuanya terbukti berjalan.
5. Memory safety ter-enforcement oleh compiler (checked_add di jalur keamanan, borrow-checker pada struktur data kernel) — menurunkan beban verifikasi MC/DC DAL-A dibanding C23.

## 3. Konsekuensi
- Positif: satu bahasa untuk kernel+driver+test; traceability dokumen→kode langsung menunjuk crate/file.
- Negatif/terkelola: target tanpa backend Rust (±35 arsitekatur per `RND_06_TARGET_FEASIBILITY.md`) wajib melalui FFI bridge atau emulasi sandbox (`asl-legacy-sandbox`); kontrak `asl-hal` menjadi satu-satunya titik integrasi.
- Aturan turunannya: **G1** tidak ada dependency eksternal baru; seluruh kode baru no_std; instruksi non-Rust tidak boleh masuk core partition di luar `extern "C"` yang di-audit.
