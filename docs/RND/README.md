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
     * Piramida verifikasi kode kernel setara standar aviasi DO-178C DAL-A & EAL7+.
     * Pembuktian matematika logika bebas panic menggunakan Kani Rust Model Checker dan Miri UB Detector.
     * Kerangka kerja simulator virtual lintas 150 arsitektur (*QEMU Multi-System, Renode IoT, Spike Vector, GPGPU-Sim, Mininet P4*).
     * 5 Gerbang kelayakan (*Gate Criteria*) sebelum melangkah ke Fase Roadmap Implementasi.

---

*Seluruh dokumen R&D ini tersimpan rapi dan terisolasi di dalam direktori `c:\asl\docs\RND` untuk menjaga kebersihan root proyek.*
