# LAPORAN AUDIT & ROADMAP ARSITEKTUR KERNEL ASL CORE
## Desain & Strategi Portabilitas Kernel Multi-Arsitektur Universal

Dokumen ini menyajikan hasil **Audit Teknis Komprehensif** terhadap 150 arsitektur di dalam katalog `ASL Core`, serta merumuskan **Roadmap Rekayasa Perangkat Lunak (Engineering Roadmap)** untuk membangun **ASL Kernel**—sebuah kernel sistem operasi modern yang modular, portabel, dan mampu mendukung seluruh spektrum perangkat keras (mulai dari mikrokontroler RTOS, router/switch jaringan, server cloud x86/ARM/RISC-V, hingga akselerator GPU/NPU/Systolic).

---

## 1. EXECUTIVE SUMMARY & HASIL AUDIT

Berdasarkan audit teknis, mustahil menerapkan pendekatan kernel monolitik tunggal konvensional untuk ke-150 arsitektur tersebut karena adanya disparitas karakteristik perangkat keras:
* Rentang lebar bus: **4-bit / 8-bit** (Intel 4004, AVR) hingga **64-bit / 128-bit** (x86-64, RV128I).
* Manajemen memori: Perangkat tanpa MMU (*Memory Management Unit*) hanya mendukung flat physical addressing, sementara server enterprise memerlukan 4-level/5-level paging virtual memory (x86-64 PML4/PML5, ARMv8 48-bit VA, RISC-V Sv39/Sv48).
* Paradigma komputasi: CPU Von Neumann/Harvard, GPU SIMT (CUDA/RDNA), DSP VLIW (TI C6000), Systolic Array (TPU), dan Programmable Packet Match-Action (P4 Tofino).

### Hasil Klasifikasi Teknis 150 Arsitektur untuk Kernel ASL:

```
[150 ARSITEKTUR ASL CORE]
       |
       +---> TIER 1: Host MMU General-Purpose CPU (71 Arsitektur) -> Native Preemptive Microkernel/Monolithic
       |     (x86-64, AArch64, RV64GC, MIPS64, POWER9/10, SPARC64, s390x, LoongArch, Tilera)
       |
       +---> TIER 2: Real-Time & Microcontroller / MPU (37 Arsitektur) -> ASL-Nano (Deterministic Tickless RTOS)
       |     (Cortex-M, Cortex-R, RV32E/IMAC, TriCore, PIC32, AVR32, SuperH, RL78)
       |
       +---> TIER 3: Coprocessor & DSP Audio/Voice (10 Arsitektur) -> DSP Subsystem Driver & RPC Hal
       |     (TI C6000, ADI SHARC, Qualcomm Hexagon HVX, Tensilica HiFi, CEVA)
       |
       +---> TIER 4: Akselerator Paralel SIMT/Systolic (9 Arsitektur) -> Compute Subsystem (DMA/HMM/UVM Driver)
       |     (NVIDIA CUDA PTX, AMD RDNA/CDNA, Intel Xe, Google TPU, Tenstorrent Tensix)
       |
       +---> TIER 5: Network Switching & Surveillance Video ASIC (15 Arsitektur) -> FastPath Hardware Offload Engine
       |     (Broadcom Tomahawk/Jericho, Marvell Prestera, P4 Tofino, HiSilicon DaVinci, Novatek)
       |
       +---> TIER 6: Arsitektur Vintage & Sub-32bit Legacy (8 Arsitektur) -> ASL-Legacy Emulation Container / JIT
             (MOS 6502, Z80, Motorola 6800/6809, Intel 4004/8008/8080, WDC 65C816)
```

---

## 2. PILIHAN PARADIGMA ARSITEKTUR KERNEL ASL

Untuk mendukung ekosistem heterogen ini, **ASL Kernel** dirancang menggunakan arsitektur **Hybrid Modular Microkernel** (mengadopsi pelajaran arsitektur sukses dari L4 microkernel, Fuchsia Zircon, dan Linux HAL):

```
+-------------------------------------------------------------------------------------------------+
|                                USERSPACE & POSIX COMPATIBILITY LAYER                            |
|                 (ASL C Library, Network Stack, Shell, System Services, Drivers Daemon)         |
+-------------------------------------------------------------------------------------------------+
|                                 ASL UNIFIED DRIVER MODEL (UDM)                                  |
|         [Ethernet/Switch]   [Video Codec/ISP]   [Wi-Fi Subsystem]   [Crypto/Security Engine]    |
+-------------------------------------------------------------------------------------------------+
|                                      ASL KERNEL CORE                                            |
|        IPC Engine  |  Scheduler (SMP/Tickless)  |  Capability-Based Security  |  VMM / Paging    |
+-------------------------------------------------------------------------------------------------+
|                                 HARDWARE ABSTRACTION LAYER (HAL)                                |
|        asl_cpu_context_switch() | asl_mmu_map() | asl_irq_dispatch() | asl_timer_tick()         |
+-------------------------------------------------------------------------------------------------+
|                                       ARCH-SPECIFIC BSPs                                        |
|  [arch/x86_64]   [arch/aarch64]   [arch/riscv64]   [arch/mips]   [arch/arm_cm]   [arch/powerpc] |
+-------------------------------------------------------------------------------------------------+
|                                      PHYSICAL HARDWARE                                          |
+-------------------------------------------------------------------------------------------------+
```

### Prinsip Desain Kunci:
1. **Separation of Mechanism and Policy:** Core kernel hanya mengelola alokasi thread, sinkronisasi primitive, dan IPC (*Inter-Process Communication*). Driver I/O berjalan di userspace (atau loadable kernel module untuk kinerja tinggi).
2. **Abstract MMU / No-MMU Interface:** Jika CPU memiliki MMU, kernel mengaktifkan virtual paging dan isolasi proses (Ring 3/EL0). Jika CPU hanya memiliki MPU atau flat memory (Cortex-M, RISC-V RV32E), kernel beralih ke proteksi region berbasis MPU atau model *Single Address Space Operating System (SASOS)*.
3. **Compile-Time Modular Profiles:**
   * **Profile A (ASL-Cloud / Server):** 64-bit, SMP masif (hingga 1024 core), NUMA, IOMMU, PCIe hotplug, eBPF-like packet filter.
   * **Profile B (ASL-Net / Carrier):** Terintegrasi langsung dengan switching fabric ASIC (Broadcom, Marvell, Nokia FP5) via DPDK-like polling mode driver.
   * **Profile C (ASL-Edge / CCTV):** Dilengkapi pipeline video frame buffer langsung ke hardware NPU (Novatek, HiSilicon, Rockchip).
   * **Profile D (ASL-Nano / RTOS):** Jejak biner di bawah 32 KB untuk mikrokontroler 32-bit (STM32, ESP32, CH32V).
   * **Profile E (ASL-Legacy):** Virtualisasi biner / emulasi interpreter untuk instruksi 8-bit/16-bit klasik.

---

## 3. STRUKTUR KODE SUMBER KERNEL ASL (`asl-kernel/`)

```
asl-kernel/
├── arch/                     # Implementasi spesifik masing-masing arsitektur
│   ├── x86_64/               # Boot, GDT/IDT, PML4 paging, APIC, context switch
│   ├── aarch64/              # Exception Levels (EL1/EL0), TTBR0/1, GICv2/v3, NEON
│   ├── aarch32/              # ARMv7-A, CP15, NVIC/GIC, Thumb-2
│   ├── arm_m/                # Cortex-M0/M3/M4/M7/M33, SysTick, PendSV, MPU
│   ├── riscv64/              # Sv39/Sv48 paging, PLIC/AIA, SBI calls, CSR registers
│   ├── riscv32/              # RV32I/IMAC, CLINT/PLIC, MPU PMP (Physical Memory Protection)
│   ├── mips/                 # MIPS32/64 R2-R6, CP0 registers, TLB handling
│   ├── powerpc/              # PPC32/64, BAT registers, Book E / OpenPOWER
│   ├── sparc/                # SPARC V8/V9, Register Windows, LEON3 fault tolerance
│   ├── s390x/                # IBM z/Architecture, control registers, channel I/O
│   ├── loongarch/            # LoongArch 64-bit LA464 CSRs, TLB, page tables
│   └── tile/                 # Tilera TILE-Gx mesh networking interconnect
├── kernel/                   # Core Kernel (Architecture-Independent)
│   ├── sched/                # Scheduler: O(1), CFS, and Fixed-Priority Preemptive RT
│   ├── ipc/                  # Zero-copy message passing & shared memory rings
│   ├── sync/                 # Mutex, spinlock, seqlock, RCU (Read-Copy Update)
│   ├── mm/                   # Buddy allocator, Slab/Slub allocator, VMA manager
│   ├── time/                 # High-resolution timers, tickless scheduler
│   └── irq/                  # Generic interrupt management & bottom-half tasklets
├── drivers/                  # Universal Driver Subsystem
│   ├── net/                  # Network drivers (GbE, 10G/100G, DPDK wrapper)
│   │   ├── switch/           # Switching ASIC drivers (Broadcom, Marvell Prestera, Realtek)
│   │   └── pon/              # ONT MAC & OLT DBA drivers (HiSilicon, Sanechips, Cortina)
│   ├── video/                # VPU & ISP surveillance drivers (Novatek, HiSilicon, Rockchip)
│   ├── serial/               # UART, 16550, ARM PL011, RISC-V NS16550A
│   └── bus/                  # PCI Express, AXI, AHB, SPI, I2C, CAN Bus
├── hal/                      # Hardware Abstraction Layer Interfaces (C headers)
│   ├── cpu.h                 # asl_cpu_halt(), asl_cpu_pause(), asl_cpu_context_save()
│   ├── mmu.h                 # asl_map_page(), asl_unmap_page(), asl_flush_tlb()
│   ├── irq.h                 # asl_enable_irq(), asl_disable_irq(), asl_register_handler()
│   └── dma.h                 # Cache coherency & DMA bounce buffers
└── legacy/                   # Emulasi runtime untuk chip vintage (MOS 6502, Z80, 8080)
```

---

## 4. ROADMAP IMPLEMENTASI KERNEL ASL (FASE 1 – 6)

```
[ROADMAP KERNEL ASL MULTI-ARSITEKTUR]

 TAHUN 1                              TAHUN 2                              TAHUN 3
 Q1    Q2    Q3    Q4          Q1    Q2    Q3    Q4          Q1    Q2    Q3    Q4
 +-----+-----+-----+-----+     +-----+-----+-----+-----+     +-----+-----+-----+-----+
 | FASE 1: Core Foundation|
 | x86_64, AArch64, RV64GC|
 +-----+-----+-----+-----+
             | FASE 2: Network & Embedded MIPS |
             | MIPS32/64, Cortex-M, ESP32, Power|
             +-----+-----+-----+-----+
                         | FASE 3: Carrier Telecom & Switching |
                         | Broadcom, Marvell, Nokia, ZTE, OLT  |
                         +-----+-----+-----+-----+
                                     | FASE 4: Enterprise, Mainframe & Space |
                                     | SPARC LEON, s390x, POWER10, LoongArch |
                                     +-----+-----+-----+-----+
                                                 | FASE 5: Compute, NPU & DSP |
                                                 | GPU, TPU, Hexagon, DaVinci |
                                                 +-----+-----+-----+-----+
                                                             | FASE 6: Legacy & Verif |
                                                             | Vintage, 150 Target Test|
                                                             +-----+-----+-----+-----+
```

### FASE 1: Core Foundation & Modern Tier-1 Support (Bulan 1 – 6)
* **Target Arsitektur:** `x86-64`, `AArch64 (ARMv8/v9)`, `RV64GC (RISC-V)`.
* **Deliverables:**
  1. Bootloader interface (UEFI, ARM Trusted Firmware ATF, U-Boot, RISC-V OpenSBI).
  2. Implementasi HAL dasar: Paging 64-bit (4-level paging), SMP multi-core scheduling, APIC / ARM GICv3 / RISC-V PLIC interrupt handling.
  3. Alokator memori inti: Physical Frame Allocator (Buddy System) + Virtual Slab Allocator.
  4. Inter-Process Communication (IPC) berlatensi ultra-rendah dan generic UART debugging.
* **Target Board Pengujian:** QEMU virt x86_64/arm64/riscv64, Raspberry Pi 5, StarFive VisionFive 2.

### FASE 2: Network Edge & Microcontroller Subsystem (Bulan 7 – 12)
* **Target Arsitektur:** `MIPS32/64`, `ARM Cortex-M (M0+/M3/M4/M7/M33)`, `RV32IMAC`, `Xtensa LX6/LX7`, `PowerPC (e500)`.
* **Deliverables:**
  1. Sub-kernel **ASL-Nano**: Tickless preemptive scheduler hemat memori tanpa MMU (menggunakan MPU).
  2. MIPS TLB handling dan context-switch untuk router MikroTik mipsbe dan Ubiquiti EdgeRouter.
  3. Dukungan FreeRTOS compatibility wrapper pada ASL-Nano untuk menjalankan ekosistem IoT.
* **Target Board Pengujian:** STM32F4/H7, ESP32, Raspberry Pi Pico (RP2040/RP2350), MikroTik RB750r2, Ubiquiti ER-X.

### FASE 3: Carrier Silicon, Switching & Fiber Optic (Bulan 13 – 18)
* **Target Arsitektur:** `Marvell Prestera`, `Broadcom StrataXGS/StrataDNX`, `HiSilicon Balong/SD511x`, `ZTE Sanechips ZX2791xx`, `Nokia Quillion`, `Cortina PON`.
* **Deliverables:**
  1. **ASL FastPath Network Engine:** Framework zero-copy packet switching langsung dari port Ethernet ke hardware switching fabric.
  2. Driver PON MAC & Dynamic Bandwidth Allocation (DBA) untuk modem ONT (ZTE F609, Huawei HG8245H) dan OLT.
  3. Abstraksi kontroler switch L2/L3 (VLAN, LACP, QoS, TCAM rule injection).
* **Target Board Pengujian:** MikroTik CRS326, Switch Marvell Armada, modul ONT SFP GPON/EPON.

### FASE 4: Mission-Critical, Mainframe & Space Grade (Bulan 19 – 24)
* **Target Arsitektur:** `SPARC V8/V9 (LEON3 Space-Grade, SPARC64)`, `IBM z/Architecture (s390x)`, `IBM POWER9/POWER10`, `LoongArch (Loongson 3A6000)`.
* **Deliverables:**
  1. Dukungan fault-tolerant Triple-Modular Redundancy (TMR) dan scrubbing memori untuk radiasi antariksa pada LEON3.
  2. Dukungan arsitektur mainframe IBM s390x (Channel I/O, storage keys, LPAR virtualization).
  3. Matrix Math Accelerator (MMA) driver pada IBM POWER10 dan translasi hardware LBT pada LoongArch.
* **Target Board Pengujian:** QEMU s390x, QEMU sparc64, board evaluasi Loongson 3A6000, FPGA board GR712RC (LEON3).

### FASE 5: Surveillance Video, NPU, DSP & GPU Compute (Bulan 25 – 30)
* **Target Arsitektur:** `HiSilicon DaVinci`, `Novatek NT98xx`, `Ambarella CVflow`, `Qualcomm Hexagon QDSP6`, `TI TMS320C6000`, `NVIDIA CUDA PTX`, `Google TPU`.
* **Deliverables:**
  1. **ASL-Media Subsystem:** Buffer ring DMA nol-kopi antara sensor kamera CMOS dan unit hardware H.264/H.265 video codec pada DVR/NVR CCTV.
  2. Unified Neural Driver Interface (UNDI) untuk eksekusi inferensi model TFLite/ONNX pada berbagai NPU.
  3. Remote Procedure Call (RPC) interkoneksi ke DSP audio untuk low-latency audio processing.
* **Target Board Pengujian:** Board kamera NVR Novatek/Rockchip RK3588, board TI LaunchPad C2000/C6000, NVIDIA Jetson Orin.

### FASE 6: Vintage Emulation, Full Integration & CI/CD Certification (Bulan 31 – 36)
* **Target Arsitektur:** Arsitektur 8/16-bit klasik (`MOS 6502`, `Zilog Z80`, `Motorola 68000 m68k`, `Intel 8086/8080`, `WDC 65C816`).
* **Deliverables:**
  1. **ASL-Legacy Container Engine:** Sandboxed bare-metal runtime yang mampu memuat binary lama secara native atau melalui just-in-time (JIT) dynamic translation ke instruction set host.
  2. Automated Cross-Architecture CI/CD Pipeline (LLVM/Clang & GCC multi-target build suite).
  3. Pengujian regresi otomatis terhadap ke-150 arsitektur menggunakan platform emulator simulasi (QEMU, MAME, Simics, Spike RISC-V).

---

## 5. REKOMENDASI TOOLCHAIN & TEKNOLOGI PENDUKUNG

> **Rebaseline 2026-10-20 (ADR-001):** Rust adalah bahasa utama (primary) — 31 crate `#![no_std]` telah dibuktikan boot & lulus 128 test host. C/assembly turun status menjadi *jembatan FFI/vendor-toolchain per-arsitektur*, bukan bahasa utama kernel.

| Domain | Teknologi yang Direkomendasikan | Alasan & Justifikasi |
| :--- | :--- | :--- |
| **Bahasa Pemrograman Utama** | **Rust (Bare-metal `#![no_std]`)** | Status AKTUAL: seluruh kernel, HAL, vault, scheduler, storage, driver ditulis Rust; memory safety + determinisme ARINC 653; 128 unit tests host lolos (lihat `docs/CANONICAL/ADR-001_LANGUAGE_STRATEGY.md`). |
| **Jembatan Arsitektur Non-Rust** | **C23 / Assembly vendor (GCC, SDCC, vbcc, TI CCS, Tasking, HighTec)** | Hanya untuk target tanpa backend Rust (IA-64, TriCore, retro 8/16-bit, DSP vendor) melalui FFI/`extern "C"` di belakang trait `asl-hal`; tidak menambah dependency ke core. |
| **Compiler Suite** | **rustc 1.80+ (LLVM) + rustup target add** | Target aktif: `x86_64-unknown-none` (runnable), `aarch64-unknown-none-softfloat` (build), host `x86_64-pc-windows-msvc` (test). |
| **Automated Testing & Emulasi** | **QEMU Multi-Arch, Spike (RISC-V), Renode (IoT Multi-Node)** | Memungkinkan pengujian otomatis kernel terhadap puluhan arsitektur board secara simultan dalam pipeline CI/CD tanpa memerlukan rak hardware fisik. |

---

## 6. KESIMPULAN & LANGKAH PERTAMA (NEXT IMMEDIATE STEPS)

Membangun **ASL Core Kernel** bukan berarti menulis 150 OS berbeda, melainkan membangun **arsitektur mikrokernel berlapis (Layered Microkernel)** dengan **Hardware Abstraction Layer (HAL)** yang sangat bersih. 

Langkah pertama yang dapat segera dieksekusi:
1. Menginisialisasi repositori source code `asl-kernel/` dengan struktur HAL modular.
2. Mengimplementasikan target **Tier-1 Reference Platform** pada arsitektur **RISC-V 64-bit (RV64GC)** dan **ARMv8-A (AArch64)** menggunakan QEMU virt board.
3. Menyusun header interface abstrak `hal/cpu.h`, `hal/mmu.h`, dan `hal/irq.h` sebagai kontrak universal untuk seluruh 150 arsitektur.
