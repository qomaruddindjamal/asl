# FASE 2: FADEC REAL-TIME CONTROLLERS & EDGE ROUTING (BULAN 7–12)
## Sub-Kernel ASL-Nano, Mikrokontroler No-MMU, Otomotif ASIL-D & Router MIPS

Dokumen ini mengatur sasaran rekayasa, deliverable arsitektural, dan protokol pengujian untuk **Fase 2**. Fase ini memperluas ASL Core ke ranah kontrol mesin deterministik (*FADEC Subsystem*), otomotif, mikrokontroler hemat energi, dan infrastruktur edge router.

---

## 1. SASARAN TEKNIS & ARSITEKTUR SASARAN
* **Arsitektur Utama:**
  1. `ARM Cortex-M (M0+, M3, M4, M7, M33, M85)` — Mikrokontroler komersial (RP2040, STM32F4/H7, nRF52840).
  2. `ARM Cortex-R (R4, R5, R52)` — Prosesor real-time deterministik lockstep otomotif (TI Hercules, Zynq UltraScale+).
  3. `Infineon TriCore (AURIX)` — Standar ECU mesin mobil dunia bersertifikasi ISO 26262 ASIL-D.
  4. `RISC-V 32-bit (RV32IMAC / RV32EC)` — Mikrokontroler IoT Wi-Fi 6 (ESP32-C3, WCH CH32V003).
  5. `MIPS32 & MIPS64` — Router SOHO dan gateway jaringan (MikroTik RB750r2, Ubiquiti ER-X, EdgeRouter PRO).
  6. `Xtensa LX6 / LX7` — SoC IoT nirkabel (ESP32 dual-core & ESP32-S3).

---

## 2. DELIVERABLES TEKNIS (BULAN 7 – BULAN 12)

### Bulan 7–8: Rekayasa Sub-Kernel "ASL-Nano" (No-MMU Tickless RTOS)
* **Jejak Biner Minimalis:** Kompilasi profil kernel berukuran < 32 KB Flash dan < 8 KB RAM.
* **Model Memori MPU:** Mengabstraksikan Memory Protection Unit (MPU 8/16 region) untuk isolasi thread tanpa virtual paging (Single Address Space OS).
* **Tickless Preemptive Scheduler:** Menghemat daya baterai dengan mematikan interupsi timer berkala saat sistem idle (`SysTick` dinamis).

### Bulan 9–10: Otomotif ASIL-D & Deterministik Lockstep
* **Dual-Core Lockstep Management:** Driver pemantau fault-detection pada ARM Cortex-R5 dan Infineon TriCore; memicu interupsi hardware instan bila ada perbedaan bit register antar-core.
* **Protokol Bus Industri:** Driver native untuk CAN Bus 2.0B, CAN-FD, dan bus deterministik FlexRay.
* **Integrasi TrustZone MCU:** Konfigurasi Secure Attribution Unit (SAU) pada ARM Cortex-M33 (RP2350) untuk perlindungan firmware anti-tamper.

### Bulan 11–12: MIPS Exception Handler & Edge Router Networking
* **TLB Refill Fastpath:** Handler assembly kustom untuk register CP0 MIPS (TLB refill di bawah 15 siklus clock).
* **Endianness Independent Driver:** Memastikan driver ethernet mendukung mode Big Endian (`mipsbe` MikroTik) dan Little Endian (`mipsel` MediaTek MT7621).
* **Kriteria Kelulusan Fase 2:** ASL-Nano berjalan mulus di STM32, ESP32, dan MikroTik mipsbe; latensi interrupt terukur < 500 nanodetik pada core real-time.
