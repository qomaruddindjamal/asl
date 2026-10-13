# ROADMAP MASTER: INTI STRATEGI PENGEMBANGAN ASL CORE KERNEL
## "Universal Avionics Operating System Framework" (Siklus Rekayasa 36 Bulan)

Dokumen ini adalah **Inti Strategi (Core Master Strategy)** dari seluruh roadmap pengembangan **ASL Core Kernel**. Dokumen ini merangkum filosofi eksekusi, pembagian 6 fase rekayasa, target deliverable, dan kriteria kelayakan terbang digital (*digital airworthiness*) setara standar **DO-178C DAL-A** untuk mengorkestrasi 150+ arsitektur perangkat keras dunia secara bertahap dan terukur.

---

## 1. INTI STRATEGI (THE CORE STRATEGY ESSENCE)

Inti dari strategi pengembangan ASL Core Kernel bertumpu pada **Prinsip Piramida Avionik Terbalik (Inverted Avionics Pyramid)**:

```
[TAHUN 3: GLOBAL INTEGRATION & CERTIFICATION]
Fase 6: Pengujian Armada Penuh (150 Target Hardware) & Rilis Publik v1.0
Fase 5: Ketahanan Antariksa (LEON3), Mainframe (s390x) & Visi Neural (NPU/GPU)
-----------------------------------------------------------------------------
[TAHUN 2: INFRASTRUKTUR TELEKOMUNIKASI, FIBER & REAL-TIME EDGE]
Fase 4: Buffer Kamera CCTV (Novatek/HiSilicon) & Pipeline Vektor AI
Fase 3: Switching Jaringan (Broadcom/Marvell) & Modem Fiber GPON/XGS-PON (ONT/OLT)
-----------------------------------------------------------------------------
[TAHUN 1: FONDASI AVIONIK & KONTROL MESIN DETERMINISTIK]
Fase 2: FADEC Real-Time Engine (Cortex-M, TriCore, ESP32) & Router MIPS
Fase 1: Wind Tunnel & FMS Core Executive (RISC-V 64-bit, ARMv8 64-bit, x86-64)
```

### 3 Hukum Eksekusi Strategis:
1. **Pemisahan Inti dan Driver (Microkernel Rigor):** Inti kernel (`avionics/`) tidak pernah memuat driver silikon langsung; semua perangkat keras dihubungkan melalui abstraksi *Universal Traits Rust* dan saluran pesan terenkripsi *AFDX Virtual Links*.
2. **Kemandirian Fase (Self-Contained Phases):** Setiap fase menghasilkan artefak kernel yang dapat di-boot dan diuji secara independen sebelum melangkah ke fase berikutnya.
3. **Keamanan Sebagai Prasyarat, Bukan Tambahan:** Tidak ada kode yang digabungkan ke cabang utama tanpa lolos verifikasi pembuktian matematika formal *Kani Model Checker* dan inspeksi memori *Miri*.

---

## 2. MATRIKS 6 FASE PENGEMBANGAN ASL CORE KERNEL

| Fase | Nama Fase & Fokus Utama | Rentang Waktu | Arsitektur Sasaran Kunci | Dokumen Detail |
| :---: | :--- | :---: | :--- | :---: |
| **FASE 1** | **Wind Tunnel & Avionics Core Foundation** | Bulan 1 – 6 | `RV64GC (RISC-V)`, `AArch64 (ARMv8/v9)`, `x86-64` | [`PHASE_01_CORE_FOUNDATION.md`](file:///c:/asl/docs/ROADMAP/PHASE_01_CORE_FOUNDATION.md) |
| **FASE 2** | **FADEC Real-Time Controllers & Edge Routing** | Bulan 7 – 12 | `ARM Cortex-M/R`, `Infineon TriCore`, `RV32IMAC`, `MIPS32/64` | [`PHASE_02_REALTIME_FADEC_AND_EDGE.md`](file:///c:/asl/docs/ROADMAP/PHASE_02_REALTIME_FADEC_AND_EDGE.md) |
| **FASE 3** | **Avionics Bus & Fiber Switching (Carrier Network)**| Bulan 13 – 18 | `Broadcom StrataXGS/DNX`, `Marvell Prestera`, `ONT & OLT PON ASIC`| [`PHASE_03_CARRIER_FIBER_SWITCHING.md`](file:///c:/asl/docs/ROADMAP/PHASE_03_CARRIER_FIBER_SWITCHING.md) |
| **FASE 4** | **Radar, Surveillance Vision & Neural Matrix** | Bulan 19 – 24 | `HiSilicon DaVinci`, `Novatek`, `Ambarella`, `TI C6000`, `CUDA PTX`| [`PHASE_04_RADAR_VISION_NEURAL.md`](file:///c:/asl/docs/ROADMAP/PHASE_04_RADAR_VISION_NEURAL.md) |
| **FASE 5** | **Mission-Critical, Mainframe & Space Endurance** | Bulan 25 – 30 | `SPARC LEON3 (ESA)`, `IBM s390x`, `IBM POWER10`, `LoongArch` | [`PHASE_05_SPACE_MAINFRAME_CRITICAL.md`](file:///c:/asl/docs/ROADMAP/PHASE_05_SPACE_MAINFRAME_CRITICAL.md) |
| **FASE 6** | **Full Fleet Integration & Global Certification** | Bulan 31 – 36 | Seluruh **150 Arsitektur Hardware** & Subsystem Vintage 8/16-bit | [`PHASE_06_FLEET_INTEGRATION_CERTIFICATION.md`](file:///c:/asl/docs/ROADMAP/PHASE_06_FLEET_INTEGRATION_CERTIFICATION.md) |

---

## 3. INDEKS DIREKTORI ROADMAP (`docs/ROADMAP/`)

Seluruh dokumen implementasi fase tersimpan rapi dan terisolasi pada folder `docs/ROADMAP/`:
* [**`README.md`**](file:///c:/asl/docs/ROADMAP/README.md) — Indeks dan navigasi panduan implementasi roadmap.
* [**`PHASE_01_CORE_FOUNDATION.md`**](file:///c:/asl/docs/ROADMAP/PHASE_01_CORE_FOUNDATION.md) — Rincian teknis, milestone, dan deliverables Fase 1.
* [**`PHASE_02_REALTIME_FADEC_AND_EDGE.md`**](file:///c:/asl/docs/ROADMAP/PHASE_02_REALTIME_FADEC_AND_EDGE.md) — Rincian teknis, milestone, dan deliverables Fase 2.
* [**`PHASE_03_CARRIER_FIBER_SWITCHING.md`**](file:///c:/asl/docs/ROADMAP/PHASE_03_CARRIER_FIBER_SWITCHING.md) — Rincian teknis, milestone, dan deliverables Fase 3.
* [**`PHASE_04_RADAR_VISION_NEURAL.md`**](file:///c:/asl/docs/ROADMAP/PHASE_04_RADAR_VISION_NEURAL.md) — Rincian teknis, milestone, dan deliverables Fase 4.
* [**`PHASE_05_SPACE_MAINFRAME_CRITICAL.md`**](file:///c:/asl/docs/ROADMAP/PHASE_05_SPACE_MAINFRAME_CRITICAL.md) — Rincian teknis, milestone, dan deliverables Fase 5.
* [**`PHASE_06_FLEET_INTEGRATION_CERTIFICATION.md`**](file:///c:/asl/docs/ROADMAP/PHASE_06_FLEET_INTEGRATION_CERTIFICATION.md) — Rincian teknis, milestone, dan deliverables Fase 6.
