# FASE 1: WIND TUNNEL & AVIONICS CORE FOUNDATION (BULAN 1–6)
## Pembangunan Inti Eksekutif FMS, Partisi ARINC 653 & Dukungan Platform Tier-1

Dokumen ini mengatur sasaran rekayasa, deliverable arsitektural, dan protokol pengujian untuk **Fase 1**. Fase ini berfokus pada pembangunan inti mikrokernel Rust, sistem isolasi partisi ruang dan waktu, serta membidik 3 arsitektur general-purpose modern (Tier-1).

---

## 1. SASARAN TEKNIS & ARSITEKTUR SASARAN
* **Arsitektur Utama:**
  1. `RISC-V 64-bit (RV64GC)` — Standar arsitektur terbuka bebas lisensi (SiFive FU740, VisionFive 2).
  2. `ARMv8-A / ARMv9-A (AArch64)` — Arsitektur server efisiensi dan komputasi mobile (Raspberry Pi 5, Apple Silicon).
  3. `x86-64 (AMD64 / Intel 64)` — Arsitektur komputasi workstation, laptop, dan cloud server.
* **Target Lingkungan Eksekusi:** Bare-metal Rust `#![no_std]` di atas platform emulator QEMU (`qemu-system-riscv64`, `qemu-system-aarch64`, `qemu-system-x86_64`) dan hardware dev board fisik.

---

## 2. DELIVERABLES TEKNIS (BULAN 1 – BULAN 6)

### Bulan 1–2: Universal Boot Protocol & Hardware Abstraction Layer (HAL)
* **Bootloader Interface:**
  * RISC-V: Integrasi dengan OpenSBI (Supervisor Binary Interface) via panggilan ecall.
  * ARM64: Integrasi dengan ARM Trusted Firmware (ATF / TF-A) pada Exception Level 1 (EL1).
  * x86-64: Protokol bootloader modern (Limine / UEFI GOP framebuffer handshake).
* **Kontrak HAL Rust:**
  * Implementasi Trait `UniversalCpuCore` untuk penanganan context-switch register callee-saved.
  * Serial UART logger universal untuk output diagnostik awal (*early printk / panic handler*).

### Bulan 3–4: Manajemen Memori & Partisi Spasial ARINC 653
* **Physical Frame Allocator:** Alokator frame fisik berbasis *Buddy Allocator* yang deterministik tanpa fragmentasi.
* **Virtual Page Table Manager:**
  * Paging 4-level x86-64 (PML4) dan ARM64 (TTBR0/TTBR1 48-bit Virtual Addressing).
  * Paging Sv39/Sv48 pada RISC-V 64-bit.
* **ARINC 653 Spatial Partitioning:** Isolasi memori antar-partisi yang menjamin tidak ada kode partisi pengguna yang dapat menyentuh memori kernel atau partisi tetangga.

### Bulan 5–6: Scheduler Deterministik & Saluran Komunikasi AFDX IPC
* **Core Scheduler:**
  * Algoritma *Fixed-Priority Preemptive Scheduler* dengan jendela waktu deterministik (*Time-Slot ARINC 653*).
  * Penanganan interupsi lokal berbasis APIC (x86), GICv3 (ARM), dan PLIC (RISC-V).
* **AFDX-Style Ring Buffer IPC:**
  * Saluran pertukaran pesan antar-thread berbasis *zero-copy lock-free ring buffer* terenkripsi.
* **Kriteria Kelulusan Fase 1:** Kernel berhasil boot di 3 arsitektur (QEMU & Pi 5), mengeksekusi 100 thread terisolasi, dan lolos uji Miri UB detector tanpa error.
