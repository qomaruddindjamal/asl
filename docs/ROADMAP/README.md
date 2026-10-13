# DIREKTORI ROADMAP STRATEGIS ASL CORE KERNEL

Folder `docs/ROADMAP/` memuat rencana eksekusi komprehensif 36 bulan untuk membangun **ASL Core Kernel** sebagai sistem operasi avionik universal yang mendukung 150+ arsitektur perangkat keras dunia secara bertahap, deterministik, dan aman.

---

## DAFTAR DOKUMEN MASTER & FASE ROADMAP:

1. [**`ROADMAP_MASTER.md`**](file:///c:/asl/docs/ROADMAP/ROADMAP_MASTER.md)
   * **Inti Strategi (The Core Essence):** Filosofi eksekusi piramida avionik terbalik, 3 hukum rekayasa modular, dan matriks rangkuman 6 fase roadmap.

2. [**`PHASE_01_CORE_FOUNDATION.md`**](file:///c:/asl/docs/ROADMAP/PHASE_01_CORE_FOUNDATION.md)
   * **Fase 1 (Bulan 1–6): Wind Tunnel & Avionics Core Foundation**
   * *Target:* `RISC-V 64-bit (RV64GC)`, `ARMv8/v9 (AArch64)`, `x86-64`.
   * *Fokus:* Boot protocol (OpenSBI, ATF, Limine), Alokator memori Buddy/Slab, Partisi Spasial ARINC 653, dan Ring-buffer AFDX IPC.

3. [**`PHASE_02_REALTIME_FADEC_AND_EDGE.md`**](file:///c:/asl/docs/ROADMAP/PHASE_02_REALTIME_FADEC_AND_EDGE.md)
   * **Fase 2 (Bulan 7–12): FADEC Real-Time Controllers & Edge Routing**
   * *Target:* `ARM Cortex-M/R`, `Infineon TriCore`, `RV32IMAC`, `MIPS32/64`, `Xtensa`.
   * *Fokus:* Sub-kernel ASL-Nano (< 32 KB), Isolasi MPU, Tickless Preemptive Scheduler, Dual-Core Lockstep Otomotif ASIL-D, dan TLB router MikroTik/Ubiquiti.

4. [**`PHASE_03_CARRIER_FIBER_SWITCHING.md`**](file:///c:/asl/docs/ROADMAP/PHASE_03_CARRIER_FIBER_SWITCHING.md)
   * **Fase 3 (Bulan 13–18): Avionics Bus & Fiber Switching (Carrier Network)**
   * *Target:* `Broadcom StrataXGS/DNX`, `Marvell Prestera`, `P4 Tofino`, `ONT & OLT GPON/XGS-PON`.
   * *Fokus:* Zero-copy Polled-Mode Driver (PMD), TCAM wire-speed rule ingestion, GEM framing modem ONT (ZTE/Huawei), dan Algoritma DBA OLT.

5. [**`PHASE_04_RADAR_VISION_NEURAL.md`**](file:///c:/asl/docs/ROADMAP/PHASE_04_RADAR_VISION_NEURAL.md)
   * **Fase 4 (Bulan 19–24): Radar, Surveillance Vision & Neural Matrix**
   * *Target:* `HiSilicon DaVinci`, `Novatek`, `Ambarella`, `TI C6000`, `Hexagon DSP`, `CUDA PTX`, `TPU`.
   * *Fokus:* Pipeline frame buffer CCTV kamera DMA nol-kopi, Unified Neural Driver Interface (UNDI), dan RPC interkoneksi DSP sinyal radio.

6. [**`PHASE_05_SPACE_MAINFRAME_CRITICAL.md`**](file:///c:/asl/docs/ROADMAP/PHASE_05_SPACE_MAINFRAME_CRITICAL.md)
   * **Fase 5 (Bulan 25–30): Mission-Critical, Mainframe & Space Endurance**
   * *Target:* `SPARC LEON3 (ESA)`, `IBM z/Architecture (s390x)`, `IBM POWER10`, `LoongArch`.
   * *Fokus:* Toleransi radiasi kosmik Triple-Modular Redundancy (TMR), Channel I/O & Storage Keys perbankan s390x, dan Hardware Translasi LBT.

7. [**`PHASE_06_FLEET_INTEGRATION_CERTIFICATION.md`**](file:///c:/asl/docs/ROADMAP/PHASE_06_FLEET_INTEGRATION_CERTIFICATION.md)
   * **Fase 6 (Bulan 31–36): Full Fleet Integration & Global Certification**
   * *Target:* Seluruh **150 Arsitektur Hardware** & Subsystem Vintage (`MOS 6502`, `Z80`, `m68k`, `8086`, `SNES`).
   * *Fokus:* Multi-Arch CI/CD Test Matrix, Sandboxed Legacy Container, Sertifikasi Aviasi DO-178C DAL-A, dan Peluncuran Resmi ASL Core Kernel v1.0.

---

*Seluruh dokumen roadmap ini terisolasi rapi di folder `c:\asl\docs\ROADMAP` demi menjaga kebersihan lingkungan kerja root repositori.*
