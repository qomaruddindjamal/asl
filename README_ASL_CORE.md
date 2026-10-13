# INDEX UTAMA ARSIP KERNEL ASL CORE & ROADMAP STRATEGIS

Dokumen ini adalah ringkasan indeks permanen untuk seluruh cetak biru, katalog arsitektur, dan roadmap strategis pengembangan **ASL Core Kernel** di direktori `c:\asl`.

---

## 1. DOKUMEN MASTER & CETAK BIRU KERNEL
* [**`asl_avionics_kernel_blueprint.md`**](file:///c:/asl/asl_avionics_kernel_blueprint.md)
  * **Fokus:** Filosofi desain **"The Avionics Framework"** (arsitektur kernel berbasis Integrated Modular Avionics pesawat terbang modern / ARINC 653).
  * **Komponen:** Cockpit Executive, Flight Control Computers (FCC), FADEC Engine Control, Radar & Vision Suite, Actuators & Hydraulics, dan Bus AFDX Virtual Links.
* [**`asl_kernel_audit_roadmap.md`**](file:///c:/asl/asl_kernel_audit_roadmap.md)
  * **Fokus:** Hasil audit teknis 6 tier komputasi, struktur direktori source code kernel (`asl-kernel/`), dan **Flight Plan / Roadmap 36 Bulan (Fase 1–6)**.

---

## 2. ENSIKLOPEDIA & REFERENSI ARSITEKTUR HARDWARE
* [**`asl_core.md`**](file:///c:/asl/asl_core.md)
  * **Fokus:** Ensiklopedia tunggal **150 Arsitektur Komputasi, Board, CPU, ISA & Silikon Jaringan** (100% unik, terverifikasi tanpa duplikasi).
* [**`ont_olt_architectures.md`**](file:///c:/asl/ont_olt_architectures.md)
  * **Fokus:** Arsitektur spesifik sistem optik FTTH: Modem ONT sisi pelanggan (ZTE, Huawei, Nokia, Fiberhome) dan OLT sisi sentral operator (Huawei MA5800, ZTE C320/C600, Nokia Quillion).
* [**`network_and_surveillance_architectures.md`**](file:///c:/asl/network_and_surveillance_architectures.md)
  * **Fokus:** Analisis chipset router & switch (MikroTik, Cisco, UniFi, TP-Link, D-Link) serta DVR/NVR surveillance CCTV.

---

## 3. RINGKASAN ROADMAP STRATEGIS 36 BULAN (FLIGHT PLAN)
1. **Fase 1 (Bulan 1–6): Wind Tunnel & Avionics Core** — Core Executive, Partisi Spasial ARINC 653, dan HAL universal untuk target awal (RISC-V 64-bit, ARMv8 64-bit, x86-64).
2. **Fase 2 (Bulan 7–12): FADEC Subsystem (Real-Time Controllers)** — Integrasi mikrokontroler deterministik (Cortex-M, TriCore, ESP32) dan core jaringan edge MIPS (MikroTik, EdgeRouter).
3. **Fase 3 (Bulan 13–18): High-Speed Avionics Bus & Network Silicon** — Integrasi zero-copy hardware packet switching (Broadcom, Marvell) dan modul transceiver fiber GPON/XGS-PON.
4. **Fase 4 (Bulan 19–24): Radar, Vision & Neural Matrix** — Integrasi akselerator video kamera CCTV (Novatek, HiSilicon, Ambarella), DSP audio, dan pipeline paralel GPU/TPU.
5. **Fase 5 (Bulan 25–30): Mission-Critical & Deep Space Endurance** — Sertifikasi toleransi radiasi pada SPARC LEON3 (ESA) dan mainframe IBM z/Architecture (s390x).
6. **Fase 6 (Bulan 31–36): Full Fleet Integration** — Integrasi armada penuh mencakup seluruh 150 arsitektur ke dalam rilis publik ASL Core Kernel v1.0 Universal Edition.
