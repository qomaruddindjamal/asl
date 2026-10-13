# FASE 4: RADAR, SURVEILLANCE VISION & NEURAL MATRIX (BULAN 19–24)
## Pipeline Video CCTV Nol-Kopi, Akselerator NPU, DSP Sinyal & Komputasi Paralel GPU

Dokumen ini mengatur sasaran rekayasa, deliverable arsitektural, dan protokol pengujian untuk **Fase 4**. Fase ini berfokus pada integrasi sensor visual, komputasi tensor neural, pengolahan sinyal digital (DSP), dan akselerasi GPU/TPU ke dalam ekosistem ASL Core Kernel.

---

## 1. SASARAN TEKNIS & ARSITEKTUR SASARAN
* **Arsitektur Utama:**
  1. `HiSilicon DaVinci NPU & VPU` — Video Codec H.264/H.265 & 3D Cube NPU (Hikvision / Dahua).
  2. `Novatek NT98xx Series` — Surveillance AI ISP SoC (Hikvision K/M Series).
  3. `Ambarella CVflow & SigmaStar` — Edge Computer Vision 8K & Dashcam Video SoC.
  4. `Texas Instruments TMS320C6000 & ADI SHARC` — VLIW Radar & High-End Audio DSP.
  5. `Qualcomm Hexagon QDSP6` — Multithreaded Vector VLIW DSP sinyal radio dan kamera AI.
  6. `NVIDIA CUDA PTX (sm_120) & Google TPU` — Akselerator matriks paralel SIMT & Systolic Array.

---

## 2. DELIVERABLES TEKNIS (BULAN 19 – BULAN 24)

### Bulan 19–20: ASL-Media Zero-Copy Video Frame Pipeline
* **DMA Ring Buffer Kamera:** Jalur transfer frame langsung dari sensor kamera CMOS (MIPI-CSI) ke VPU decoder tanpa penyalinan memori CPU (*Zero-Copy Buffer Sharing*).
* **Driver Multi-Channel H.265/H.264:** Manajemen encode/decode simultan 4–32 channel video pada SoC Novatek dan HiSilicon.
* **Smart ISP Control:** Pengaturan otomatis white-balance, eksposur, dan 3D Digital Noise Reduction (3D-DNR) via kernel API.

### Bulan 21–22: Unified Neural Driver Interface (UNDI) & Akselerasi NPU
* **Abstraksi Tensor Hardware:** Driver universal untuk memuat bobot model neural network kuantisasi INT8 / FP16 langsung ke SRAM internal NPU.
* **Asynchronous Execution Queue:** Pengiriman batch gambar ke unit NPU menggunakan command queue non-blocking; pemanggilan callback interrupt saat inferensi selesai.
* **Pipeline Systolic Array (TPU & Tensix):** Manajemen aliran data berantai pada arsitektur matriks tanpa siklus pembacaan register berulang.

### Bulan 23–24: DSP Remote Procedure Call (RPC) & Driver Komputasi GPU
* **Interkoneksi Shared-Memory DSP:** Protokol komunikasi dua arah antara CPU utama dan core DSP (TI C6000 / Hexagon) berlatensi < 5 mikrodetik.
* **Unified Virtual Memory (UVM) Handler:** Sinkronisasi alamat memori bersama antara CPU host dan GPU NVIDIA/AMD untuk beban kerja AI.
* **Kriteria Kelulusan Fase 4:** Streaming 16 channel video 4K berjalan mulus dengan utilisasi CPU di bawah 5%, inferensi deteksi wajah real-time stabil pada NPU.
