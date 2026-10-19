# INDEKS RISET & PENGEMBANGAN (R&D) ASL CORE KERNEL

Repositori dokumen di dalam folder `docs/RND/` adalah fondasi ilmiah, arsitektural, dan hasil kajian teknis mendalam sebelum pelaksanaan tahapan pada roadmap implementasi ASL Core Kernel.

---

## DAFTAR DOKUMEN RISET & ANALISIS R&D:

1. [**`RND_01_RUST_FEASIBILITY.md`**](file:///c:/asl/docs/RND/RND_01_RUST_FEASIBILITY.md)
   * **Judul:** Feasibilitas Arsitektur Kernel Rust pada 150+ Arsitektur Hardware.
   * **Topik Utama:**
     * Analisis mode `#![no_std]` murni tanpa dependensi OS.
     * Taksonomi Rust Target Compiler (Tier-1, Tier-2, Tier-3, LLVM, dan GCC Codegen Backend).
     * Desain Rust Traits (`UniversalCpuCore`, `UniversalMemoryProtection`, `UniversalInterruptMatrix`) untuk abstraksi hardware berbiaya nol (*Zero-Cost Abstractions*).
     * Pembuktian matematis hilangnya kelas kerentanan memori C (*Buffer Overflow, Use-After-Free, Double-Free, Race Condition*).

2. [**`RND_02_AVIONICS_ROLE_ALLOCATION.md`**](file:///c:/asl/docs/RND/RND_02_AVIONICS_ROLE_ALLOCATION.md)
   * **Judul:** Pembagian Peran Sistem Avionik dari 150+ Arsitektur Hardware.
   * **Topik Utama:**
     * Klasifikasi komprehensif seluruh 150 arsitektur ke dalam 6 Pos Peran Pesawat Terbang (*Flight Management System, Flight Control Computers, FADEC Engine Controllers, Radar/Vision/NPU Suite, Actuator & Fiber Switching Fabric, Auxiliary Blackbox*).
     * Matriks pemetaan peran individual dari nomor 1 s.d. nomor 150 tanpa ada arsitektur yang terlewat.

3. [**`RND_03_SECURITY_ARMOR_SPECIFICATION.md`**](file:///c:/asl/docs/RND/RND_03_SECURITY_ARMOR_SPECIFICATION.md)
   * **Judul:** Doktrin Pertahanan Keamanan Siber Mutlak Kernel Rust (The Impenetrable Vault).
   * **Topik Utama:**
     * 6 Lapisan pertahanan siber multi-lapis (*Defense-in-Depth*).
     * Protokol bus terenkripsi AFDX Virtual Link berkecepatan tinggi dengan traffic shaping.
     * Peniadaan hak akses Superuser/Root melalui sistem *Capability-Based Security Tokens*.
     * Pertahanan hardware MTE, Pointer Authentication (PAC), dan Shadow Stack.
     * Mekanisme isolasi partisi kedap udara ARINC 653 serta fitur pemulihan mandiri instan (*Auto-Purge & Self-Healing < 1 ms*).

4. [**`RND_04_TESTING_AND_FORMAL_VERIFICATION.md`**](file:///c:/asl/docs/RND/RND_04_TESTING_AND_FORMAL_VERIFICATION.md)
   * **Judul:** Protokol R&D, Pengujian Otomatis & Verifikasi Formal.
   * **Topik Utama:**
     * Piramida verifikasi kode kernel mengarah standar aviasi DO-178C (ladder DAL-C->A, ADR-002) & EAL7+.
     * Pembuktian matematika logika bebas panic menggunakan Kani Rust Model Checker dan Miri UB Detector.
     * Kerangka kerja simulator virtual lintas 150 arsitektur (*QEMU Multi-System, Renode IoT, Spike Vector, GPGPU-Sim, Mininet P4*).
     * 5 Gerbang kelayakan (*Gate Criteria*) sebelum melangkah ke Fase Roadmap Implementasi.

5. [**`RND_05_CRITICAL_REMEDIATION.md`**](file:///c:/asl/docs/RND/RND_05_CRITICAL_REMEDIATION.md)
   * **Judul:** Desain Teknis Remediasi 4 Temuan Kritis (Level Kode Rust).
   * **Topik Utama:**
     * Penegakan batas keselarasan 64-byte (`ALIGN-01`) pada struktur data DMA.
     * Abstraksi pemeliharaan cache (`DMA-CACHE-01`) dan Isolated Bounce Buffer.
     * Translasi dan firewall alamat DMA (`DMA-ADDR-01`) via IOMMU Translation Space.
     * Dual-track atomics pada SPARC V8/V9 (`ATOMIC-SPARC-01`).

6. [**`RND_06_TARGET_FEASIBILITY.md`**](file:///c:/asl/docs/RND/RND_06_TARGET_FEASIBILITY.md)
   * **Judul:** Matriks Kelayakan 150 Target Silikon & Evaluasi Kompiler.
   * **Topik Utama:**
     * Klasifikasi 4 Tier Realitas Eksekusi Kompiler: Native Rust Tier-A, Custom Spec Tier-B, C-Toolchain FFI Tier-C, dan Sandboxed Retro Tier-D.
     * Disposisi arsitektur lawas IA-64, s390x, dan chip retro 8-bit (Z80, 6502).

7. [**`RND_07_FORMAL_VERIFICATION.md`**](file:///c:/asl/docs/RND/RND_07_FORMAL_VERIFICATION.md)
   * **Judul:** Rencana Verifikasi Formal Berlapis & Studi Kelayakan Ferrocene.
   * **Topik Utama:**
     * Model verifikasi 4-layer (Miri UB, Kani SAT/SMT, TLA+/PlusCal, Ferrocene).
     * Verifikasi temporal ketiadaan deadlock pada SPSC Ring Buffer AFDX.

8. [**`RUST_MICROKERNEL_ENGINEERING_GUIDELINES.md`**](file:///c:/asl/docs/RND/RUST_MICROKERNEL_ENGINEERING_GUIDELINES.md)
   * **Judul:** Panduan Standar Rekayasa Microkernel Rust (Efisiensi Low-Spec & Responsivitas).
   * **Topik Utama:**
     * Doktrin Zero-Heap di driver path untuk alokasi RAM minimal (teruji di 4 MB RAM).
     * Bounded Polling Loop & Timeout Recovery untuk responsivitas tinggi anti-hang.
     * Standar disiplin penulisan `#![no_std]` dan mitigasi `unsafe` code.

---

*Seluruh dokumen R&D ini tersimpan rapi dan terisolasi di dalam direktori `c:\asl\docs\RND` untuk menjaga kebersihan root proyek.*
