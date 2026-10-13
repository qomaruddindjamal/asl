# ENSIKLOPEDIA 150 ARSITEKTUR KOMPUTASI, HARDWARE, JARINGAN & BOARD

Dokumen master ini menggabungkan seluruh arsitektur prosesor (CPU, ISA, SoC), hardware jaringan (MikroTik, Cisco, UniFi, TP-Link, D-Link, Nokia, ZTE, Huawei), sistem pengawasan/surveillance (DVR/NVR), arsitektur telekomunikasi, dan akselerator komputasi.

Total entri: **150 Arsitektur Unik (No Duplicates)**, diverifikasi secara sistematis.

---

## DAFTAR ISI & KATEGORI

* **Kategori 1: x86 & Arsitektur CISC Klasik hingga Modern** (Nomor 1 – 12)
* **Kategori 2: Keluarga Arm Architecture** (Nomor 13 – 28)
* **Kategori 3: Arsitektur Terbuka RISC-V** (Nomor 29 – 42)
* **Kategori 4: Keluarga MIPS Architecture** (Nomor 43 – 52)
* **Kategori 5: Power Architecture & PowerPC** (Nomor 53 – 62)
* **Kategori 6: SPARC Family** (Nomor 63 – 70)
* **Kategori 7: Mainframe & Arsitektur Enterprise Historis** (Nomor 71 – 78)
* **Kategori 8: Mikrokontroler & Embedded Kontemporer** (Nomor 79 – 92)
* **Kategori 9: Mikroprosesor Retro & Vintage Legendaris** (Nomor 93 – 102)
* **Kategori 10: Digital Signal Processors (DSP)** (Nomor 103 – 110)
* **Kategori 11: GPU & Akselerator Komputasi Paralel** (Nomor 111 – 116)
* **Kategori 12: Arsitektur Khusus, Many-Core & Emerging Modern** (Nomor 117 – 120)
* **Kategori 13: Arsitektur Switching & Routing Silicon Jaringan Carrier/Enterprise** (Nomor 121 – 128)
* **Kategori 14: Arsitektur Subsystem Wi-Fi & Gateway SoC Jaringan** (Nomor 129 – 131)
* **Kategori 15: Arsitektur Video Surveillance, AI ISP & NPU Kamera** (Nomor 132 – 135)
* **Kategori 16: Arsitektur Mobile Application Processor, Baseband & NPU** (Nomor 136 – 140)
* **Kategori 17: Arsitektur Komputasi Vintage & Milestone Tambahan** (Nomor 141 – 148)
* **Kategori 18: Arsitektur Heterogen & AI Mesh Modern** (Nomor 149 – 150)

---

### KATEGORI 1: x86 & ARSITEKTUR CISC KLASIK HINGGA MODERN (1–12)

1. **x86-64 (AMD64 / Intel 64)**
   * **Tipe:** CISC 64-bit | **Vendor Utama:** AMD, Intel
   * **Deskripsi:** Arsitektur standar desktop, laptop, server cloud, dan SBC berperforma tinggi (LattePanda, ODROID-H3, MikroTik CHR). 16 general register (RAX–R15), instruksi SIMD (SSE, AVX-512, AVX10).

2. **IA-32 (x86-32 / i386)**
   * **Tipe:** CISC 32-bit | **Vendor Utama:** Intel
   * **Deskripsi:** Arsitektur 32-bit komersial legendaris perintis Protected Mode, paging hardware, dan virtual memory; dipakai pada era PC Windows 95–XP dan board Intel Galileo (Quark SoC).

3. **Intel 8086 / 8088**
   * **Tipe:** CISC 16-bit | **Vendor Utama:** Intel
   * **Deskripsi:** Fondasi keluarga x86 dunia. Pengalamatan memori tersegmentasi 1 MB (bus alamat 20-bit). Menjadi otak IBM PC 5150 asli.

4. **Intel 80186**
   * **Tipe:** CISC 16-bit Embedded | **Vendor Utama:** Intel
   * **Deskripsi:** Mengintegrasikan interrupt controller, DMA controller, timer, dan chip-select logic langsung ke dalam satu silikon. Banyak dipakai pada pengontrol instrumen industri dan avionik.

5. **Intel 80286**
   * **Tipe:** CISC 16-bit | **Vendor Utama:** Intel
   * **Deskripsi:** Memperkenalkan Protected Mode berbasis ring proteksi (Ring 0–3) dan virtual memory addressing hingga 16 MB. Menjadi basis komputer IBM PC/AT.

6. **Intel IA-64 (Itanium / EPIC)**
   * **Tipe:** EPIC 64-bit | **Vendor Utama:** Intel, HP
   * **Deskripsi:** Explicitly Parallel Instruction Computing. Mengelompokkan tiga instruksi ke dalam bundle 128-bit dengan predication penuh untuk server mission-critical enterprise (HP Integrity).

7. **AMD K6-2 / 3DNow!**
   * **Tipe:** CISC 32-bit SIMD | **Vendor Utama:** AMD
   * **Deskripsi:** Mikroarsitektur x86 Socket 7 pertama yang menghadirkan instruksi floating-point SIMD untuk akselerasi grafik dan multimedia 3D sebelum munculnya SSE.

8. **Transmeta Crusoe**
   * **Tipe:** VLIW 128-bit + CMS | **Vendor Utama:** Transmeta
   * **Deskripsi:** Core fisik VLIW sederhana yang dipadukan dengan Code Morphing Software (CMS) untuk menerjemahkan instruksi x86 secara dinamis dengan daya sangat rendah (Sony VAIO PictureBook).

9. **Transmeta Efficeon**
   * **Tipe:** VLIW 256-bit + CMS Generasi 2 | **Vendor Utama:** Transmeta
   * **Deskripsi:** Varian 256-bit VLIW dengan bus HyperTransport dan on-die memory controller terintegrasi untuk subnotebook dan embedded PC industri.

10. **Cyrix 6x86 / MII**
    * **Tipe:** CISC 32-bit Superscalar | **Vendor Utama:** Cyrix, National Semiconductor
    * **Deskripsi:** Arsitektur x86 superscalar dengan speculative execution dan register renaming yang kompatibel penuh dengan soket Socket 7 standar.

11. **VIA C3 / Eden (Centaur Architecture)**
    * **Tipe:** CISC 32-bit Ultra-Low Power | **Vendor Utama:** VIA Technologies, Centaur
    * **Deskripsi:** Prosesor hemat energi pelopor form factor Mini-ITX. Menyertakan mesin enkripsi hardware PadLock (AES/SHA) langsung pada level instruksi silikon.

12. **VIA Nano (Isaiah Architecture)**
    * **Tipe:** CISC 64-bit Out-of-Order | **Vendor Utama:** VIA Technologies
    * **Deskripsi:** Arsitektur 64-bit superscalar out-of-order dari VIA yang dirancang untuk komputasi embedded, netbook, dan sistem digital signage industri.

---

### KATEGORI 2: KELUARGA ARM ARCHITECTURE (13–28)

13. **AArch64 (ARMv8-A / ARMv9-A)**
    * **Tipe:** RISC 64-bit | **Vendor Utama:** Arm Ltd, Apple, Qualcomm, Broadcom
    * **Deskripsi:** 31 register 64-bit (X0–X30), SIMD NEON/SVE2. Otak dari Raspberry Pi 4/5, Apple Silicon (M1–M4), AWS Graviton, router Wi-Fi 6/7, dan MikroTik CCR modern.

14. **AArch32 (ARMv7-A)**
    * **Tipe:** RISC 32-bit Application Profile | **Vendor Utama:** Arm Ltd, Qualcomm, TI
    * **Deskripsi:** Inti Cortex-A7/A9/A15 dengan MMU Linux penuh dan NEON SIMD. Digunakan luas pada BeagleBone Black, MikroTik hEX S, serta modem ONT fiber GPON (ZTE F609, Huawei HG8245H).

15. **ARMv6 (ARM11)**
    * **Tipe:** RISC 32-bit | **Vendor Utama:** Arm Ltd, Broadcom
    * **Deskripsi:** Menghadirkan instruksi media SIMD dasar. Menjadi prosesor utama Raspberry Pi 1, Raspberry Pi Zero (Broadcom BCM2835), dan iPhone 2G generasi pertama.

16. **ARMv5TE / ARM9 (ARM926EJ-S)**
    * **Tipe:** RISC 32-bit Embedded | **Vendor Utama:** NXP, Atmel, Samsung
    * **Deskripsi:** Dilengkapi ekstensi DSP matematika dan akselerasi Jazelle DBX Java bytecode. Dipakai pada router ADSL rumahan klasik dan iPod Classic.

17. **ARMv4T / ARM7TDMI**
    * **Tipe:** RISC 32-bit Classic | **Vendor Utama:** Arm Ltd
    * **Deskripsi:** Pelopor instruksi Thumb 16-bit terkompresi hemat memori. Menjadi otak konsol Nintendo Game Boy Advance dan sub-processor audio Nintendo DS.

18. **ARM Cortex-M0 / M0+ (ARMv6-M)**
    * **Tipe:** RISC 32-bit Microcontroller Ultra-Low Power | **Vendor Utama:** Raspberry Pi, NXP, ST
    * **Deskripsi:** Silikon ultra-ringkas (~12.000 gerbang) untuk perangkat baterai. Otak dari Raspberry Pi Pico (RP2040) dan Arduino Zero (SAMD21).

19. **ARM Cortex-M3 (ARMv7-M)**
    * **Tipe:** RISC 32-bit Microcontroller Mainstream | **Vendor Utama:** STMicroelectronics
    * **Fitur:** Set instruksi Thumb-2 murni, NVIC deterministik, hardware divide. Otak board legendaris STM32F103 "Blue Pill" dan Arduino Due.

20. **ARM Cortex-M4 / M4F (ARMv7E-M)**
    * **Tipe:** RISC 32-bit DSP Microcontroller | **Vendor Utama:** STMicroelectronics, Nordic
    * **Fitur:** Instruksi DSP single-cycle, SIMD arithmetic, dan opsional hardware FPU. Otak dari STM32F4 Discovery, Teensy 3.2, dan chip Bluetooth nRF52840.

21. **ARM Cortex-M7 (ARMv7E-M)**
    * **Tipe:** RISC 32-bit High-Performance MCU | **Vendor Utama:** NXP, STMicroelectronics
    * **Fitur:** 6-stage superscalar pipeline, cache L1 I/D, Double Precision FPU, clock hingga 1 GHz. Digunakan pada Teensy 4.0 / 4.1 dan STM32H7.

22. **ARM Cortex-M33 (ARMv8-M Mainline)**
    * **Tipe:** RISC 32-bit Secure MCU | **Vendor Utama:** Raspberry Pi, Nordic, ST
    * **Fitur:** Menghadirkan proteksi hardware TrustZone pada mikrokontroler IoT. Otak dari Raspberry Pi Pico 2 (RP2350) dan nRF5340 dual-core BLE.

23. **ARM Cortex-M55 / M85 (ARMv8.1-M)**
    * **Tipe:** RISC 32-bit TinyML Microcontroller | **Vendor Utama:** Renesas, ST
    * **Fitur:** Dilengkapi Helium Vector DSP untuk akselerasi eksekusi model machine learning / edge vision langsung pada mikrokontroler (Renesas RA8D1).

24. **ARM Cortex-R4 / R5 (ARMv7-R)**
    * **Tipe:** RISC 32-bit Real-Time Profile | **Vendor Utama:** Texas Instruments, Xilinx
    * **Fitur:** Eksekusi deterministik tanpa jitter, MPU, mode dual-core lockstep untuk sertifikasi keselamatan otomotif (TI Hercules TMS570, Xilinx Zynq UltraScale+).

25. **ARM Cortex-R52 / R82 (ARMv8-R)**
    * **Tipe:** RISC 32/64-bit Real-Time Virtualized | **Vendor Utama:** Arm Ltd
    * **Fitur:** Hardware virtualization untuk lingkungan real-time; mampu mengisolasi beberapa OS RTOS pada satu silikon ECU mobil otonom dan kontroler NVMe enterprise.

26. **ARM Cortex-A53 / A55**
    * **Tipe:** RISC 64-bit Efficiency Core | **Vendor Utama:** Rockchip, Amlogic, Broadcom
    * **Fitur:** Pipeline in-order hemat daya; menjadi backbone jutaan TV box, router Wi-Fi 6, DVR surveillance, dan Raspberry Pi 3.

27. **ARM Cortex-A76 / A78 / X-Core Series**
    * **Tipe:** RISC 64-bit Out-of-Order Flagship | **Vendor Utama:** Rockchip, Qualcomm, MediaTek
    * **Fitur:** Arsitektur Out-of-Order superscalar berperforma desktop. Digunakan pada Raspberry Pi 5 (Broadcom BCM2712) dan Orange Pi 5 (Rockchip RK3588).

28. **ARM SecurCore (SC000 / SC300)**
    * **Tipe:** RISC 32-bit Tamper-Proof | **Vendor Utama:** Infineon, NXP, ST
    * **Fitur:** Dilengkapi perlindungan fisik anti-DPA (Differential Power Analysis) dan enkripsi bus memori untuk smartcard SIM 5G, paspor elektronik, dan kartu ATM/kredit EMV.

---

### KATEGORI 3: ARSITEKTUR TERBUKA RISC-V (29–42)

29. **RV32I (Base Integer 32-bit)**
    * **Tipe:** Open-standard RISC 32-bit | **Organisasi:** RISC-V International
    * **Deskripsi:** Set instruksi dasar minimalis hanya 47 instruksi dengan 32 register (x0–x31 di mana x0 hardwired ke 0). Fondasi desain soft-core FPGA dan riset komputasi.

30. **RV32E / RV32EC (Embedded Base)**
    * **Tipe:** RISC 32-bit Embedded | **Organisasi:** RISC-V International
    * **Deskripsi:** Mengurangi jumlah register menjadi 16 (x0–x15) guna meminimalkan area footprint silikon ASIC seminimal mungkin.

31. **RV32IMAC**
    * **Tipe:** RISC 32-bit MCU Profile | **Vendor Utama:** Espressif, GigaDevice
    * **Deskripsi:** Mendukung perkalian/pembagian integer (M), atomic memory (A), dan instruksi terkompresi 16-bit (C). Otak dari SoC IoT nirkabel ESP32-C3 dan ESP32-C6.

32. **RV32IMAFDC (RV32GC)**
    * **Tipe:** RISC 32-bit General Purpose | **Vendor Utama:** Andes Technology, Western Digital
    * **Deskripsi:** Dilengkapi unit floating-point presisi tunggal (F) dan ganda (D); konfigurasi lengkap komputasi ilmiah dan kontroler industri 32-bit.

33. **RV64I (Base Integer 64-bit)**
    * **Tipe:** RISC 64-bit Standard | **Organisasi:** RISC-V International
    * **Deskripsi:** Memperlebar seluruh register dan jalur data menjadi 64-bit native untuk komputasi enterprise dan server.

34. **RV64GC (RV64IMAFDC)**
    * **Tipe:** RISC 64-bit Application Profile | **Vendor Utama:** StarFive, Allwinner, SiFive
    * **Deskripsi:** Standar resmi minimum untuk menjalankan OS Linux (Debian, Ubuntu, Fedora) pada RISC-V. Digunakan pada VisionFive 2 (JH7110), Nezha D1, dan Lichee Pi 4A.

35. **RV64GCV (Vector Extension V1.0)**
    * **Tipe:** RISC 64-bit Vector SIMD | **Vendor Utama:** SpacemiT, Banana Pi
    * **Deskripsi:** Dilengkapi ekstensi register vektor dinamis (VLEN) untuk akselerasi pemrosesan citra, audio, dan inferensi model AI (Banana Pi BPI-F3 / SpacemiT K1).

36. **RV128I**
    * **Tipe:** RISC 128-bit Future ISA | **Organisasi:** RISC-V International
    * **Deskripsi:** Pengalamatan memori datar hingga 128-bit untuk riset komputasi skala exabyte/zettabyte di masa depan.

37. **SiFive E-Core (E21 / E31 / E76)**
    * **Tipe:** RISC-V Commercial Core | **Vendor Utama:** SiFive
    * **Deskripsi:** Inti mikrokontroler real-time 32-bit komersial efisiensi tinggi pada board HiFive1 Rev B (FE310 SoC).

38. **SiFive U-Core (U54 / U74 / P550)**
    * **Tipe:** RISC-V Commercial Linux Core | **Vendor Utama:** SiFive
    * **Deskripsi:** Inti Linux 64-bit superscalar berkecepatan tinggi dengan cache koheren multiprosesor pada board HiFive Unmatched.

39. **T-Head XuanTie C906 / C910**
    * **Tipe:** High-Performance RISC-V Core | **Vendor Utama:** Alibaba T-Head
    * **Deskripsi:** Core open-source out-of-order superscalar berkinerja tinggi yang mampu berjalan di atas 2 GHz pada Sipeed Lichee RV dan Milk-V Meles.

40. **CH32V (WCH QingKe RISC-V Core)**
    * **Tipe:** Ultra-Low Cost RISC-V MCU | **Vendor Utama:** WCH (WinChipHead)
    * **Deskripsi:** Mikrokontroler RISC-V dengan harga sekitar $0.10 yang menggantikan mikrokontroler 8-bit tradisional (board CH32V003, CH32V203, CH32V307).

41. **Western Digital SweRV (EH1 / EH2 / EL2)**
    * **Tipe:** Open-Source Superscalar RISC-V Core | **Vendor Utama:** Western Digital
    * **Deskripsi:** Dual-issue superscalar pipeline 32-bit yang dirancang dan di-open-source-kan untuk kontroler storage SSD/HDD berkecepatan tinggi.

42. **PULP Platform (PULPino / RI5CY / OpenPULP)**
    * **Tipe:** Parallel Ultra-Low-Power RISC-V | **Institusi:** ETH Zürich, Univ. of Bologna
    * **Deskripsi:** Arsitektur multi-core klaster dengan ekstensi instruksi DSP/AI hardware untuk komputasi biomedis dan drone otonom mini (Crazyflie drone).

---

### KATEGORI 4: KELUARGA MIPS ARCHITECTURE (43–52)

43. **MIPS I (R2000 / R3000)**
    * **Tipe:** RISC 32-bit Original | **Vendor Utama:** MIPS Computer Systems
    * **Deskripsi:** 32 register, pipeline 5-tahap klasik tanpa interlock hardware. Otak konsol Sony PlayStation 1 (R3000A custom @ 33.8 MHz) dan workstation DECstation 3100.

44. **MIPS II (R6000)**
    * **Tipe:** RISC 32-bit Enhanced | **Vendor Utama:** MIPS Computer Systems
    * **Deskripsi:** Menambahkan instruksi sinkronisasi multiprocessing Load-Linked dan Store-Conditional (LL/SC) untuk server enterprise era UNIX awal.

45. **MIPS III (R4000 / VR4300)**
    * **Tipe:** RISC 64-bit Pioneer | **Vendor Utama:** MIPS Technologies, NEC
    * **Deskripsi:** Mikroprosesor RISC 64-bit komersial pertama di dunia dengan arsitektur superpipelined. Menjadi otak konsol Nintendo 64 (NEC VR4300) dan workstation SGI Indigo2.

46. **MIPS IV (R8000 / R10000)**
    * **Tipe:** RISC 64-bit Out-of-Order | **Vendor Utama:** SGI / MIPS Technologies
    * **Deskripsi:** 4-way superscalar out-of-order execution dengan instruksi FPU multiply-accumulate (MADD) pada superkomputer SGI Origin 2000 dan visual workstation SGI Onyx2.

47. **MIPS V**
    * **Tipe:** RISC 64-bit Paired-Single SIMD | **Vendor Utama:** MIPS Technologies
    * **Deskripsi:** Mengimplementasikan instruksi paired-single floating point dan ekstensi grafis MIPS-3D untuk pemrosesan geometri grafik 3D.

48. **MIPS32 (Release 1 hingga 6)**
    * **Tipe:** RISC 32-bit Standar Industri | **Vendor Utama:** Broadcom, MediaTek, Qualcomm Atheros
    * **Deskripsi:** Standar 32-bit yang mendominasi jutaan router rumahan OpenWrt, MikroTik SOHO (RB750r2, RB951Ui), EdgeRouter X (MediaTek MT7621AT), TP-Link WR840N.

49. **MIPS64 (Release 1 hingga 6)**
    * **Tipe:** RISC 64-bit Enterprise & Edge | **Vendor Utama:** Cavium (Marvell), Broadcom
    * **Deskripsi:** Eksekusi paket jaringan throughput tinggi dengan akselerasi coprocessor crypto. Otak dari Ubiquiti EdgeRouter (ER-4, ER-8, ERLite-3) dan UniFi Security Gateway (USG-3P).

50. **microMIPS**
    * **Tipe:** Variable-length 16/32-bit RISC | **Vendor Utama:** Microchip, MIPS
    * **Deskripsi:** Kompresi set instruksi yang menghemat ukuran biner memori hingga 35% tanpa mengurangi kecepatan eksekusi 32-bit (Microchip PIC32MZ).

51. **Ingenic XBurst (XBurst1 / XBurst2)**
    * **Tipe:** Ultra-Low Power MIPS32 Compatible | **Vendor Utama:** Ingenic Semiconductor
    * **Deskripsi:** Pipeline kustom berkecepatan 1.0–1.5 GHz dengan konsumsi daya di bawah 500 mW. Banyak dipakai pada kamera IP CCTV Wyze/Xiaomi, smartwatch, dan console retro Anbernic.

52. **Loongson-1 / Loongson-2 (GS232 / GS464 MIPS Cores)**
    * **Tipe:** MIPS-Compatible Chinese Domestic CPU | **Vendor Utama:** Loongson Technology
    * **Deskripsi:** Implementasi core MIPS 64-bit buatan Tiongkok dengan ekstensi multimedia sebelum Loongson bermigrasi penuh ke arsitektur LoongArch.

---

### KATEGORI 5: POWER ARCHITECTURE & POWERPC (53–62)

53. **IBM POWER1 / POWER2**
    * **Tipe:** Multi-chip RISC Enterprise | **Vendor Utama:** IBM
    * **Deskripsi:** Salah satu arsitektur superscalar pertama di dunia yang mengeksekusi instruksi branch, integer, dan floating-point secara paralel pada workstation IBM RS/6000.

54. **PowerPC 601 / 603 / 604 (G2/G3)**
    * **Tipe:** RISC 32-bit AIM Alliance | **Vendor Utama:** Apple, IBM, Motorola
    * **Deskripsi:** Hasil konsorsium Apple-IBM-Motorola untuk komputer personal Apple Power Macintosh 6100/7100 dan iMac G3 legendaris.

55. **PowerPC G4 (MPC7400 / AltiVec)**
    * **Tipe:** RISC 32-bit dengan SIMD AltiVec | **Vendor Utama:** Motorola / Freescale
    * **Deskripsi:** Menyertakan unit vektor SIMD 128-bit AltiVec (Velocity Engine) untuk komputasi multimedia real-time pada Apple Power Mac G4 dan board radar militer VME.

56. **PowerPC 970 (PowerPC G5)**
    * **Tipe:** RISC 64-bit High-Performance | **Vendor Utama:** IBM
    * **Deskripsi:** Turunan prosesor server enterprise POWER4 yang diadaptasi ke komputer desktop pada Apple Power Mac G5 dan iMac G5.

57. **IBM POWER9**
    * **Tipe:** 64-bit Enterprise & Supercomputer | **Vendor Utama:** IBM
    * **Deskripsi:** Mendukung PCIe 4.0 dan NVLink 2.0 native dengan interkoneksi GPU NVIDIA Volta; menggerakkan superkomputer ORNL Summit dan workstation OpenPOWER Talos II.

58. **IBM POWER10**
    * **Tipe:** 64-bit Cloud Datacenter & Enterprise | **Vendor Utama:** IBM
    * **Deskripsi:** Fabrikasi 7nm dengan Matrix Math Accelerator (MMA) untuk analitik AI real-time pada server transaksi perbankan IBM Power E1080.

59. **PowerPC 750CL / Broadway**
    * **Tipe:** RISC 32-bit Custom Gaming | **Vendor Utama:** IBM
    * **Deskripsi:** Varian efisiensi tinggi berbasis G3 dengan ekstensi paired-single floating-point pada Nintendo GameCube ("Gekko") dan Nintendo Wii ("Broadway").

60. **PowerPC Espresso**
    * **Tipe:** Tri-core RISC 32-bit Custom | **Vendor Utama:** IBM
    * **Deskripsi:** Desain tri-core multi-chip module (MCM) terintegrasi dengan eDRAM on-die besar pada konsol Nintendo Wii U.

61. **Xenon (PowerPC Architecture)**
    * **Tipe:** Tri-Core 64-bit SMT Custom | **Vendor Utama:** IBM
    * **Deskripsi:** 3 core fisik identik yang masing-masing menjalankan 2 thread (total 6 hardware threads) @ 3.2 GHz dengan unit VMX128 SIMD pada konsol Microsoft Xbox 360.

62. **Power Architecture e200 / e500 (NXP QorIQ / MPC5xxx)**
    * **Tipe:** RISC Automotive & Telecommunication | **Vendor Utama:** Freescale / NXP
    * **Deskripsi:** Mematuhi Book E Power Architecture, tahan suhu ekstrem dan vibrasi dengan dual-core lockstep pada ECU kontrol otomotif, avionik pesawat, dan router lawas MikroTik RB1100AHx2.

---

### KATEGORI 6: SPARC FAMILY (63–70)

63. **SPARC V7**
    * **Tipe:** RISC 32-bit Foundation | **Vendor Utama:** Sun Microsystems
    * **Deskripsi:** Mengimplementasikan konsep register windows yang mengurangi overhead pemanggilan fungsi (subroutine call) secara signifikan pada workstation Sun-4.

64. **SPARC V8**
    * **Tipe:** RISC 32-bit Standar IEEE 1754 | **Vendor Utama:** Sun Microsystems, Fujitsu
    * **Deskripsi:** Mendukung perkalian dan pembagian integer hardware 32-bit dan spesifikasi memori multiprocessing formal (TSO/PSO) pada Sun SPARCstation 5 dan SPARCstation 20.

65. **LEON2 / LEON3 / LEON4 (Space-Grade SPARC V8)**
    * **Tipe:** Radiation-Hardened Open-Source SPARC V8 | **Institusi:** ESA (European Space Agency), Cobham Gaisler
    * **Deskripsi:** Dilengkapi Triple-Modular Redundancy (TMR) anti-radiasi kosmik. Menjadi standar satelit orbit geostasioner dan wahana penjelajah antariksa ESA Mars Express.

66. **SPARC V9 (UltraSPARC I hingga IV)**
    * **Tipe:** RISC 64-bit | **Vendor Utama:** Sun Microsystems
    * **Deskripsi:** Pengalamatan 64-bit penuh, out-of-order execution, dan instruksi SIMD Visual Instruction Set (VIS) pada workstation Sun Ultra series dan server Enterprise 10000.

67. **UltraSPARC T1 / T2 (Niagara / CoolThreads)**
    * **Tipe:** High-Throughput Many-Core SPARC V9 | **Vendor Utama:** Sun Microsystems
    * **Deskripsi:** Pelopor throughput computing: T1 memiliki 8 core / 32 threads, sedangkan T2 memiliki 8 core / 64 threads dengan efisiensi daya luar biasa pada server web Sun Fire T1000/T2000.

68. **Fujitsu SPARC64 (VI hingga X+)**
    * **Tipe:** Mission-Critical 64-bit Mainframe SPARC | **Vendor Utama:** Fujitsu
    * **Deskripsi:** Arsitektur tangguh dengan ECC menyeluruh pada semua register dan cache, out-of-order execution masif pada server enterprise Fujitsu/Oracle M10 series.

69. **Fujitsu SPARC64 VIIIfx / IXfx**
    * **Tipe:** Supercomputer HPC SPARC | **Vendor Utama:** Fujitsu, RIKEN
    * **Deskripsi:** Dilengkapi ekstensi HPC-ACE dengan register floating point ganda; menggerakkan superkomputer K-Computer Jepang yang pernah menduduki peringkat 1 dunia.

70. **OpenSPARC (T1 & T2 RTL)**
    * **Tipe:** Open-Source Hardware V9 | **Vendor Utama:** Sun Microsystems
    * **Deskripsi:** Kode sumber sirkuit terpadu (RTL Verilog) prosesor komersial 64-bit pertama yang dibuka bebas ke publik oleh Sun pada tahun 2006 untuk riset multiprosesor FPGA.

---

### KATEGORI 7: MAINFRAME & ARSITEKTUR ENTERPRISE HISTORIS (71–78)

71. **IBM System/360 (S/360)**
    * **Tipe:** CISC Mainframe Pioneer | **Vendor Utama:** IBM
    * **Deskripsi:** Menetapkan standar komputasi modern pada tahun 1964: byte 8-bit, heksadesimal, floating-point standar, dan isolasi arsitektur dari implementasi fisik (dipakai misi NASA Apollo).

72. **IBM System/370 (S/370)**
    * **Tipe:** CISC Mainframe Virtual Memory | **Vendor Utama:** IBM
    * **Deskripsi:** Memperkenalkan Dynamic Address Translation (DAT) untuk virtual memory addressing dan multiprocessing pada mainframe perbankan global era 1970–1980-an.

73. **IBM ESA/390 (Enterprise Systems Architecture/390)**
    * **Tipe:** 31-bit Enterprise Mainframe | **Vendor Utama:** IBM
    * **Deskripsi:** Menggunakan arsitektur pengalamatan 31-bit dengan Access Registers untuk melompat antar-address space tanpa batas memori 2 GB pada server IBM 9672.

74. **IBM z/Architecture (s390x)**
    * **Tipe:** CISC 64-bit Mainframe Modern | **Vendor Utama:** IBM
    * **Deskripsi:** Mainframe 64-bit modern yang menjamin 100% backward-compatibility hingga kode tahun 1964; mendukung on-chip AI accelerator dan enkripsi kuantum (IBM z16, mengamankan 90% transaksi kartu kredit dunia).

75. **DEC Alpha (Alpha AXP / 21064, 21164, 21264)**
    * **Tipe:** RISC 64-bit Pure Architecture | **Vendor Utama:** Digital Equipment Corporation (DEC)
    * **Deskripsi:** Dirancang dari nol murni 64-bit tanpa membawa beban 32-bit; prosesor komersial pertama yang menembus clock 1 GHz pada server Cray T3E dan search engine AltaVista.

76. **HP PA-RISC (Precision Architecture / PA-7000 hingga PA-8900)**
    * **Tipe:** RISC 32/64-bit Enterprise | **Vendor Utama:** Hewlett-Packard
    * **Deskripsi:** Arsitektur tanpa microcode dengan instruksi branch delay slot fleksibel untuk server enterprise HP 9000 dan sistem operasi HP-UX.

77. **Motorola 88000 (m88k / 88100, 88110)**
    * **Tipe:** RISC 32-bit Clean Design | **Vendor Utama:** Motorola
    * **Deskripsi:** Desain RISC murni dengan chip terpisah untuk prosesor (88100) dan cache/MMU controller (88200) sebelum Motorola bergabung ke aliansi PowerPC.

78. **CDC 6600 (Seymour Cray Architecture)**
    * **Tipe:** Arsitektur Superkomputer Awal | **Vendor Utama:** Control Data Corporation
    * **Deskripsi:** Diakui sebagai superkomputer pertama di dunia; memisahkan CPU pusat berkecepatan tinggi dari 10 Peripheral Processor (PP) independen yang bertugas menangani seluruh I/O secara mandiri.

---

### KATEGORI 8: MIKROKONTROLER & EMBEDDED KONTEMPORER (79–92)

79. **Microchip PIC16 / PIC18 (8-bit Baseline & Enhanced)**
    * **Tipe:** Harvard Architecture 8-bit RISC | **Vendor Utama:** Microchip Technology
    * **Deskripsi:** Hanya memiliki ~35 instruksi sederhana, sangat kebal terhadap interferensi noise listrik di industri perakitan, mesin cuci, dan board PICkit.

80. **Microchip dsPIC33 / PIC24**
    * **Tipe:** 16-bit MCU + DSP Engine | **Vendor Utama:** Microchip Technology
    * **Deskripsi:** Dilengkapi dua akumulator 40-bit dan instruksi Multiply-Accumulate (MAC) satu siklus untuk inverter solar panel, kontrol motor BLDC, dan SMPS.

81. **Microchip PIC32 (PIC32MX / PIC32MZ)**
    * **Tipe:** 32-bit Microcontroller | **Vendor Utama:** Microchip Technology
    * **Deskripsi:** Berbasis core MIPS32 M4K / microAptiv dengan kontroler USB OTG, CAN bus, dan Ethernet terintegrasi pada board ChipKIT Uno32.

82. **Atmel AVR (8-bit / ATmega328P, ATtiny85)**
    * **Tipe:** 8-bit RISC Harvard Architecture | **Vendor Utama:** Atmel / Microchip
    * **Deskripsi:** 32 register kerja langsung terhubung ke ALU (1 instruksi per siklus clock hingga 20 MHz). Otak legendaris Arduino Uno R3 dan Arduino Nano.

83. **Atmel AVR32 (UC3 / AP7)**
    * **Tipe:** 32-bit RISC Embedded | **Vendor Utama:** Atmel
    * **Deskripsi:** Menggabungkan densitas kode tinggi dengan instruksi SIMD/DSP dan efisiensi energi untuk gateway jaringan embedded (NGW100).

84. **Intel 8051 (MCS-51 Architecture)**
    * **Tipe:** 8-bit CISC Harvard Microcontroller | **Vendor Utama:** Intel, NXP, Silicon Labs, STC
    * **Deskripsi:** Arsitektur MCU paling banyak dilisensikan dalam sejarah peradaban silikon; memiliki instruksi pemrosesan boolean level-bit langsung ke register internal (STC89C52, kontroler keyboard USB).

85. **Espressif Xtensa LX6 (Tensilica)**
    * **Tipe:** 32-bit Dual-Core Harvard | **Vendor Utama:** Cadence Tensilica, Espressif
    * **Deskripsi:** Dual-core 32-bit hingga 240 MHz yang mengintegrasikan stack Wi-Fi 802.11 b/g/n dan Bluetooth 4.2 BLE pada silikon board ESP32 NodeMCU.

86. **Espressif Xtensa LX7**
    * **Tipe:** 32-bit Dual-Core Vector | **Vendor Utama:** Espressif
    * **Deskripsi:** Dilengkapi instruksi Processor Instruction Extension (PIE) untuk komputasi neural network dan DSP audio lokal pada board ESP32-S3 (AI camera & speech recognition).

87. **Renesas RL78**
    * **Tipe:** 16-bit CISC Ultra-Low Power | **Vendor Utama:** Renesas Electronics
    * **Deskripsi:** Konsumsi arus operasional hanya 46 μA/MHz dan standby di bawah 0.5 μA pada smart meter listrik PLN, alat tensimeter digital, dan sensor IoT.

88. **Renesas RX (RXv1 – RXv3)**
    * **Tipe:** 32-bit CISC High-Efficiency | **Vendor Utama:** Renesas Electronics
    * **Deskripsi:** Pipeline 5-tahap dengan throughput instruksi mendekati 1 siklus per instruksi, dilengkapi FPU ganda pada panel HMI elevator industri (board RX65N).

89. **Renesas V850 / RH850**
    * **Tipe:** 32-bit RISC Automotive Standard | **Vendor Utama:** Renesas Electronics
    * **Deskripsi:** Arsitektur otomotif bersertifikasi keselamatan ASIL-D dengan antarmuka CAN-FD dan FlexRay pada ECU airbag dan power steering elektronik (EPS).

90. **Hitachi/Renesas SuperH SH-2**
    * **Tipe:** 32-bit RISC Dual-Core Pioneer | **Vendor Utama:** Hitachi
    * **Deskripsi:** Arsitektur instruksi fixed 16-bit hemat bus memori dengan kemampuan clustering multiprosesor tanpa lem eksternal (glueless) pada konsol Sega Saturn dan mesin arcade ST-V.

91. **Hitachi/Renesas SuperH SH-4**
    * **Tipe:** 32-bit RISC dengan 128-bit Vector FPU | **Vendor Utama:** Hitachi
    * **Deskripsi:** Dilengkapi FPU vektor 128-bit yang mampu melakukan transformasi matriks 3D 1.4 GFLOPS pada konsol Sega Dreamcast dan sistem arcade Sega NAOMI.

92. **Infineon TriCore (AURIX)**
    * **Tipe:** Unified 32-bit RISC + DSP + MCU | **Vendor Utama:** Infineon Technologies
    * **Deskripsi:** Menggabungkan logika kontrol MCU, kecepatan DSP, dan prosesor RISC dalam arsitektur lockstep aman bersertifikasi ASIL-D pada ECU mesin bahan bakar (Bosch ECU) dan inverter mobil listrik.

---

### KATEGORI 9: MIKROPROSESOR RETRO & VINTAGE LEGENDARIS (93–102)

93. **MOS Technology 6502**
    * **Tipe:** 8-bit NMOS Classic | **Vendor Utama:** MOS Technology
    * **Deskripsi:** Arsitektur legendaris berbiaya sangat murah ($25 saat mikroprosesor lain dijual $200+) dengan register minimalis. Menggerakkan Apple II, Commodore 64 (6510), dan Nintendo NES / Famicom (Ricoh 2A03).

94. **Zilog Z80**
    * **Tipe:** 8-bit CISC Extended | **Vendor Utama:** Zilog
    * **Deskripsi:** Kompatibel penuh dengan Intel 8080 namun melipatgandakan register internal (set register bayangan A', F', B', C') dan DRAM refresh hardware. Menggerakkan ZX Spectrum, komputer MSX, dan Sega Master System.

95. **Motorola 6800**
    * **Tipe:** 8-bit Mikroprosesor Pertama Motorola | **Vendor Utama:** Motorola
    * **Deskripsi:** Menggunakan dua akumulator 8-bit (A dan B) dan register indeks 16-bit dengan pengalamatan memori linier 64 KB pada mesin arcade klasik dan pinball Williams.

96. **Motorola 6809**
    * **Tipe:** 8-bit CISC Tercanggih | **Vendor Utama:** Motorola
    * **Deskripsi:** Arsitektur 8-bit paling ortogonal: mendukung perkalian hardware 8x8, dua stack pointer, dua register indeks, dan posisi kode independen (Tandy Color Computer, Vectrex).

97. **Motorola 68000 (m68k)**
    * **Tipe:** CISC 16/32-bit Legendaris | **Vendor Utama:** Motorola
    * **Deskripsi:** Register internal 32-bit penuh (D0–D7, A0–A7), bus eksternal 16-bit, pengalamatan memori datar 16 MB. Menggerakkan Apple Macintosh 128K, Commodore Amiga 500, Atari ST, Sega Genesis, dan arcade Capcom CPS-1/CPS-2 (Street Fighter II).

98. **Motorola 68020 / 68030**
    * **Tipe:** CISC 32-bit Murni | **Vendor Utama:** Motorola
    * **Deskripsi:** Bus alamat dan data eksternal 32-bit penuh, cache instruksi on-chip (68020), dan cache data terintegrasi serta MMU paging on-chip (68030) pada Commodore Amiga 1200/3000 dan Sun-3.

99. **Motorola 68040 / 68060**
    * **Tipe:** Superscalar CISC 32-bit | **Vendor Utama:** Motorola
    * **Deskripsi:** Arsitektur superscalar m68k dengan FPU hardware terintegrasi dan instruksi pipeline multi-eksekusi pada NeXTstation (mesin tempat WWW pertama diciptakan Tim Berners-Lee) dan Amiga 4000.

100. **Intel 4004**
     * **Tipe:** 4-bit Monolithic Microprocessor Pertama | **Vendor Utama:** Intel
     * **Deskripsi:** Mikroprosesor komersial monolithic satu chip pertama dalam sejarah peradaban manusia (1971), memproses data dalam format BCD 4-bit pada kalkulator meja Busicom 141-PF.

101. **Intel 8008**
     * **Tipe:** 8-bit Microprocessor Pertama Intel | **Vendor Utama:** Intel
     * **Deskripsi:** Arsitektur 8-bit awal dengan stack internal 7 level dan bus multiplexed pada terminal komputer Datapoint 2200 dan komputer rakitan awal Micral N.

102. **Intel 8080 / 8085**
     * **Tipe:** 8-bit Foundation PC Era | **Vendor Utama:** Intel
     * **Deskripsi:** Menggandakan kinerja 8008 dengan stack pointer 16-bit berbasis RAM eksternal, melahirkan era komputasi personal, OS CP/M, komputer Altair 8800, dan arcade Space Invaders asli.

---

### KATEGORI 10: DIGITAL SIGNAL PROCESSORS / DSP (103–110)

103. **Texas Instruments TMS320C2000 (C28x Core)**
     * **Tipe:** 32-bit Real-Time DSP | **Vendor Utama:** Texas Instruments
     * **Deskripsi:** Menggabungkan kecepatan DSP dengan fleksibilitas mikrokontroler; respons interrupt di bawah 10 siklus clock pada inverter solar panel dan stasiun charging mobil listrik (EV charger).

104. **Texas Instruments TMS320C6000 (C66x / C67x Cores)**
     * **Tipe:** VLIW High-Performance DSP | **Vendor Utama:** Texas Instruments
     * **Deskripsi:** Arsitektur VelociTI VLIW: mengeksekusi hingga 8 instruksi 32-bit per siklus clock secara serentak pada pemrosesan radar militer, sonar, dan base station seluler 4G/5G.

105. **Analog Devices SHARC (ADSP-21xxx Series)**
     * **Tipe:** Super Harvard Architecture DSP (SHARC) | **Vendor Utama:** Analog Devices
     * **Deskripsi:** Memori ganda independen (Program Memory dan Data Memory) untuk komputasi audio floating-point 32/40-bit tanpa overhead latensi pada studio mixer rekaman profesional dan DSP bioskop Dolby/DTS.

106. **Analog Devices Blackfin (BF5xx / BF7xx)**
     * **Tipe:** 16/32-bit Convergent Processor | **Vendor Utama:** Analog Devices
     * **Deskripsi:** Menggabungkan sistem operasi RTOS (uClinux) dan pemrosesan sinyal dual-MAC 16-bit dalam satu core pada telepon VoIP enterprise dan kamera inspeksi pabrik.

107. **Motorola / NXP DSP56000 (DSP56k)**
     * **Tipe:** 24-bit Fixed-Point DSP | **Vendor Utama:** Motorola / NXP
     * **Deskripsi:** Lebar bus dan register 24-bit unik yang menghasilkan 144 dB dynamic range audio murni tanpa saturasi pada komputer NeXT, synthesizer Atari Falcon, dan studio effect Lexicon.

108. **Qualcomm Hexagon DSP (QDSP6)**
     * **Tipe:** Multi-Threaded VLIW DSP | **Vendor Utama:** Qualcomm
     * **Deskripsi:** Arsitektur VLIW dengan hardware multithreading interleaved (HMT) dan vektor HVX 1024-bit pada SoC Snapdragon smartphone untuk komputasi AI kamera dan modem 5G.

109. **CEVA-TeakLite / SensPro**
     * **Tipe:** Low-Power Embedded DSP IP Core | **Vendor Utama:** CEVA Inc.
     * **Deskripsi:** Arsitektur silikon berdaya sangat rendah untuk memproses data sensor hub dan active noise cancellation (ANC) pada chip Bluetooth TWS earphone dan smartwatch.

110. **Cadence Tensilica HiFi DSP (HiFi 3 / HiFi 4 / HiFi 5)**
     * **Tipe:** Audio & Voice AI DSP Core | **Vendor Utama:** Cadence Design Systems
     * **Deskripsi:** Set instruksi komputasi audio VLIW dan SIMD terpadu untuk inferensi model suara berbasis AI pada smart speaker Google Nest, Amazon Echo, dan Smart TV Samsung/LG.

---

### KATEGORI 11: GPU & AKSELERATOR KOMPUTASI PARALEL (111–116)

111. **NVIDIA CUDA PTX (Parallel Thread Execution)**
     * **Tipe:** Virtual Intermediate ISA SIMT | **Vendor Utama:** NVIDIA
     * **Deskripsi:** ISA perantara komputasi paralel masif yang dieksekusi melalui koordinasi thread dalam *warp* (dari arsitektur Kepler hingga Blackwell `sm_120`) pada kartu workstation RTX dan GPU datacenter H100/B200.

112. **AMD GCN (Graphics Core Next)**
     * **Tipe:** SIMD Wavefront-64 GPU Architecture | **Vendor Utama:** AMD
     * **Deskripsi:** Arsitektur GPU pertama AMD berbasis Compute Units independen dengan thread pool *Wavefront-64* pada GPU Radeon HD 7000/RX 500 serta chip APU konsol PlayStation 4 dan Xbox One.

113. **AMD RDNA (RDNA 1, 2, 3 - Radeon DNA)**
     * **Tipe:** Low-Latency Wavefront-32 GPU Architecture | **Vendor Utama:** AMD
     * **Deskripsi:** Dirancang khusus untuk efisiensi latensi gaming dengan thread model *Wavefront-32*, Infinity Cache, dan hardware ray-tracing pada Radeon RX 6000/7000, PlayStation 5, dan Xbox Series X.

114. **AMD CDNA (CDNA 1, 2, 3 - Compute DNA)**
     * **Tipe:** Datacenter & HPC Accelerator Architecture | **Vendor Utama:** AMD
     * **Deskripsi:** Arsitektur komputasi murni tanpa rasterizer grafis; dilengkapi Matrix Core masif untuk akselerasi pelatihan AI FP8/FP16/BF16 pada akselerator AMD Instinct MI250X dan MI300A/X (superkomputer Frontier).

115. **Intel Xe / Xe-HPC (Ponte Vecchio)**
     * **Tipe:** Scalable Parallel GPU Compute Architecture | **Vendor Utama:** Intel
     * **Deskripsi:** Arsitektur multi-tile berbasis packaging 3D Foveros dengan unit matriks Intel XMX (Xe Matrix Extensions) dan memori HBM2e pada superkomputer Aurora.

116. **Google TPU (Tensor Processing Unit / V1 hingga V5p / Trillium)**
     * **Tipe:** Dedicated ASIC Systolic Array | **Vendor Utama:** Google
     * **Deskripsi:** Arsitektur *Systolic Array*; data mengalir langsung antar-sel komputasi Matrix Multiply Unit (MXU) tanpa perlu membaca-tulis register berkali-kali pada cluster pelatihan AI Google Gemini dan Search.

---

### KATEGORI 12: ARSITEKTUR KHUSUS, MANY-CORE & EMERGING MODERN (117–120)

117. **LoongArch (Loongson Architecture)**
     * **Tipe:** Sovereign 32/64-bit General-Purpose RISC | **Vendor Utama:** Loongson Technology
     * **Deskripsi:** Arsitektur instruksi mandiri Tiongkok (LA464/LA664) yang dirancang bebas dari lisensi asing; dilengkapi ekstensi translasi biner hardware (LBT) untuk mengeksekusi instruksi x86 dan ARM langsung pada PC Loongson 3A6000.

118. **Kalray MPPA (Massively Parallel Processor Array / Coolidge)**
     * **Tipe:** Manycore VLIW Asynchronous Architecture | **Vendor Utama:** Kalray
     * **Deskripsi:** Mengintegrasikan 80 hingga 160 core VLIW 64-bit independen melalui Network-on-Chip (NoC) deterministik tanpa bottleneck bus memori untuk SmartNIC datacenter dan sistem kendali mobil otonom.

119. **XMOS xCORE (Logical Core Multithreading)**
     * **Tipe:** Deterministic Multithreaded 32-bit | **Vendor Utama:** XMOS
     * **Deskripsi:** Mengeksekusi 8–16 logical core perangkat keras secara deterministik tanpa jitter (100 ps); mampu mengemulasikan peripheral hardware (I2S, USB, CAN) murni via software pada DAC audio USB audiophile.

120. **Tilera / Mellanox TILE-Gx (Tile Network Architecture)**
     * **Tipe:** Mesh Network-on-Chip Manycore RISC | **Vendor Utama:** Tilera, Mellanox / NVIDIA
     * **Deskripsi:** Kisi matriks (grid) 36 hingga 72 core RISC 64-bit 3-way VLIW yang saling terhubung melalui router paket on-chip iMesh pada SmartNIC dan MikroTik Cloud Core Router lawas (CCR1036, CCR1072).

---

### KATEGORI 13: ARSITEKTUR SILIKON SWITCHING & ROUTING JARINGAN (121–128)

121. **Broadcom StrataXGS (Tomahawk & Trident Series)**
     * **Tipe:** High-Density Datacenter Switch ASIC | **Vendor Utama:** Broadcom
     * **Deskripsi:** Arsitektur silikon switching standar emas datacenter global (Tomahawk 1–5 hingga throughput 51.2 Tbps per chip); menggerakkan switch spine/leaf Arista, Cisco Nexus, dan whitebox Facebook/Meta.

122. **Broadcom StrataDNX (Jericho & Qumran Series)**
     * **Tipe:** Deep-Buffer Carrier Routing ASIC | **Vendor Utama:** Broadcom
     * **Deskripsi:** Arsitektur silikon router carrier-grade dengan buffer paket memori HBM besar untuk menangani tabel routing internet global (BGP full-table) pada router Cisco, ZTE, dan Nokia.

123. **Marvell Prestera (Alleycat, Bobcat, Falcon, Aldrin)**
     * **Tipe:** Enterprise Packet Processor & Switch ASIC | **Vendor Utama:** Marvell
     * **Deskripsi:** Arsitektur pipeline switching terintegrasi yang menjadi otak pemrosesan data plane pada seri switch MikroTik CRS (CRS326, CRS328), UniFi Switch, dan Cisco SMB switch.

124. **Marvell OCTEON TX / TX2 (ARMv8-A Multi-Core NPU)**
     * **Tipe:** High-End Multi-Core ARM Network Processor | **Vendor Utama:** Marvell (Cavium)
     * **Deskripsi:** Evolusi arsitektur Cavium dari MIPS ke 24–36 custom core ARMv8-A 64-bit dengan coprocessor akselerasi paket IPsec/SSL, DPI, dan virtual switching pada SmartNIC dan security appliance.

125. **Realtek RTL83xx Managed Switch Architecture**
     * **Tipe:** Embedded L2/L3 Gigabit Switch Controller | **Vendor Utama:** Realtek
     * **Deskripsi:** Arsitektur SoC switching ekonomis berdaya rendah dengan CPU kontrol terintegrasi pada jutaan switch manageable D-Link (DGS-1210), TP-Link Easy Smart, dan router SOHO.

126. **Intel Tofino (Barefoot P4 Architecture)**
     * **Tipe:** Fully User-Programmable P4 Switch ASIC | **Vendor Utama:** Intel / Barefoot Networks
     * **Deskripsi:** Arsitektur pipeline pemrosesan paket jaringan (Protocol-Independent Switch Architecture / PISA) yang 100% perilakunya dapat diprogram bebas oleh engineer menggunakan bahasa pemrograman P4.

127. **Juniper Trio Chipset (Junos Packet Engine)**
     * **Tipe:** Carrier-Grade Network Processing Unit (NPU) | **Vendor Utama:** Juniper Networks
     * **Deskripsi:** Arsitektur silikon kustom multi-core berpemilik pada line card router Juniper MX Series (MX240, MX480, MX960) untuk lookup rute BGP kecepatan kawat dan subscriber management broadband.

128. **Juniper Express Silicon (Express 4 & Express 5)**
     * **Tipe:** Cloud-Scale Routing ASIC | **Vendor Utama:** Juniper Networks
     * **Deskripsi:** Arsitektur ASIC routing skala hyperscale dengan throughput hingga 28.8 Tbps per silikon pada router core backbone Juniper PTX Series (PTX10000).

---

### KATEGORI 14: ARSITEKTUR SUBSYSTEM WI-FI & GATEWAY SOC JARINGAN (129–131)

129. **Qualcomm Networking Pro / IPQ Family Architecture**
     * **Tipe:** Wi-Fi 6/7 Gateway & Network Processing Subsystem | **Vendor Utama:** Qualcomm
     * **Deskripsi:** Arsitektur hybrid multi-core ARM Cortex-A53/A73 yang dipadukan dengan NPU hardware packet-offload 12-stream Wi-Fi pada UniFi U6-Pro, MikroTik hAP ax3, dan router flagship Linksys.

130. **MediaTek Filogic (Filogic 820 / 830 / 880 Series)**
     * **Tipe:** Wi-Fi 6/7 High-Efficiency Gateway SoC | **Vendor Utama:** MediaTek
     * **Deskripsi:** Arsitektur ARM multi-core 64-bit yang dipadukan dengan mesin akselerasi paket perangkat keras Packet Processing Engine (PPE) pada TP-Link Archer AX, D-Link Eagle Pro AI, dan board Banana Pi open-source.

131. **Broadcom BCM49xx / BCM67xx Wi-Fi Core Architecture**
     * **Tipe:** Quad-Core 64-bit Gateway Platform | **Vendor Utama:** Broadcom
     * **Deskripsi:** Arsitektur quad-core ARMv8-A Cortex-B53 64-bit dengan engine switching terintegrasi yang mendominasi router gaming flagship ASUS ROG, Netgear Nighthawk, dan TP-Link Archer BE series.

---

### KATEGORI 15: ARSITEKTUR VIDEO SURVEILLANCE, AI ISP & NPU KAMERA (132–135)

132. **HiSilicon DaVinci NPU & VPU Video Architecture**
     * **Tipe:** Neural Processing & Video Codec Engine | **Vendor Utama:** HiSilicon (Huawei)
     * **Deskripsi:** Arsitektur komputasi matriks kubus 3D (3D Cube Computing) yang dipadukan dengan decoder video multi-channel H.264/H.265; mendominasi 70% pasar global DVR/NVR CCTV Hikvision dan Dahua.

133. **Novatek NT98xx Video Codec Architecture**
     * **Tipe:** Surveillance SoC with Integrated AI ISP | **Vendor Utama:** Novatek
     * **Deskripsi:** Arsitektur dual/quad-core ARM Cortex-A53 dengan akselerator inferensi AI 1.5–3.0 TOPS yang menjadi substitusi utama chip HiSilicon pada NVR Hikvision K/M Series dan Dahua WizSense.

134. **SigmaStar Smart Video Architecture**
     * **Tipe:** Edge Video & Audio Codec SoC | **Vendor Utama:** SigmaStar
     * **Deskripsi:** Arsitektur prosesor dual-core ARM Cortex-A7 hemat daya dengan engine Smart Codec H.265+ untuk DVR analog 4–16 channel (Hikvision Turbo HD dan Dahua Cooper Series).

135. **Ambarella CVflow Deep Learning Architecture**
     * **Tipe:** Computer Vision Dedicated AI Silicon | **Vendor Utama:** Ambarella
     * **Deskripsi:** Arsitektur pipeline neural network khusus pemrosesan visual beresolusi 4K/8K dengan konsumsi daya sangat rendah pada NVR enterprise kamera keamanan Axis, Bosch, dan Hanwha Vision.

---

### KATEGORI 16: ARSITEKTUR MOBILE APPLICATION PROCESSOR, BASEBAND & NPU (136–140)

136. **Qualcomm Snapdragon Kryo CPU Architecture**
     * **Tipe:** Custom/Semi-Custom ARM Application Core | **Vendor Utama:** Qualcomm
     * **Deskripsi:** Implementasi mikroarsitektur core ARM 64-bit berkecepatan tinggi pada smartphone flagship Android, modul router 5G CPE, dan board drone Snapdragon Flight.

137. **Apple Silicon Neural Engine (ANE)**
     * **Tipe:** Dedicated Matrix Co-Processor Architecture | **Vendor Utama:** Apple
     * **Deskripsi:** Arsitektur co-processor dedicated 16-core terintegrasi pada chip Apple seri A dan M untuk mengeksekusi operasi tensor inferensi AI lokal on-device hingga 38 TOPS.

138. **Samsung Exynos (Mongoose CPU Architecture / M1–M5)**
     * **Tipe:** Custom 64-bit ARMv8-A Microarchitecture | **Vendor Utama:** Samsung Electronics
     * **Deskripsi:** Desain mikroarsitektur core mandiri out-of-order dari Samsung dengan instruction window lebar dan branch target buffer masif untuk smartphone Galaxy seri S dan modem seluler.

139. **Tensilica ConnX BBE (Baseband DSP Architecture)**
     * **Tipe:** Vector DSP for Wireless Communications | **Vendor Utama:** Cadence Tensilica
     * **Deskripsi:** Arsitektur DSP vektor VLIW khusus pemrosesan lapisan fisik (PHY layer) modem radio seluler 4G/5G, radar otomotif MIMO, dan satelit IoT.

140. **Ceva-XC Vector DSP Architecture**
     * **Tipe:** Multi-Gigabit Cellular Baseband DSP | **Vendor Utama:** CEVA Inc.
     * **Deskripsi:** Arsitektur DSP vektor SIMD/VLIW skala besar untuk menangani komputasi demodulasi sinyal radio seluler 5G Advanced pada baseband transceiver BTS operator.

---

### KATEGORI 17: ARSITEKTUR KOMPUTASI VINTAGE & MILESTONE TAMBAHAN (141–148)

141. **DEC PDP-11 (Unibus Architecture)**
     * **Tipe:** 16-bit CISC Milestone | **Vendor Utama:** Digital Equipment Corporation (DEC)
     * **Deskripsi:** Arsitektur komputer legendaris tempat bahasa pemrograman C dan sistem operasi UNIX pertama kali diciptakan oleh Dennis Ritchie dan Ken Thompson di Bell Labs.

142. **DEC VAX (Virtual Address eXtension)**
     * **Tipe:** 32-bit Orthogonal CISC | **Vendor Utama:** Digital Equipment Corporation (DEC)
     * **Deskripsi:** Arsitektur CISC paling ortogonal dalam sejarah komputasi dengan ratusan mode pengalamatan kompleks; mendominasi komputasi universitas dan riset era 1980-an (VAX-11/780).

143. **Intel i960 (80960)**
     * **Tipe:** 32-bit Embedded RISC Superscalar | **Vendor Utama:** Intel
     * **Deskripsi:** Mikroprosesor embedded 32-bit populer era 1990-an dengan register window besar; menjadi standar pengontrol RAID controller SCSI server dan printer laser berkecepatan tinggi.

144. **Intel i860 (80860)**
     * **Tipe:** VLIW / RISC Graphics Coprocessor | **Vendor Utama:** Intel
     * **Deskripsi:** Prosesor 64-bit pertama Intel dengan integer dan floating-point pipeline paralel independen; digunakan pada akselerator workstation grafis 3D dan superkomputer Intel Paragon.

145. **Motorola ColdFire**
     * **Tipe:** Variable-Length RISC m68k Successor | **Vendor Utama:** Motorola / NXP / Freescale
     * **Deskripsi:** Evolusi arsitektur 68000 yang disederhanakan menjadi pipeline RISC hemat silikon untuk mikrokontroler kontrol otomasi industri dan instrumen medis.

146. **NEC V20 / V30**
     * **Tipe:** Enhanced x86 / 8080 Dual-Mode CISC | **Vendor Utama:** NEC
     * **Deskripsi:** Kloning Intel 8086/8088 yang direkayasa ulang dengan arsitektur mikro lebih cepat (menghemat siklus clock hingga 30%) dan memiliki mode hardware untuk mengeksekusi instruksi Intel 8080.

147. **AMD Am29000 (29k Family)**
     * **Tipe:** 32-bit RISC Embedded Pioneer | **Vendor Utama:** AMD
     * **Deskripsi:** Arsitektur RISC awal AMD dengan register file masif (192 register) berbasis arsitektur Berkeley RISC; digunakan pada kontroler printer PostScript dan avionik militer.

148. **Western Design Center 65C816**
     * **Tipe:** 16-bit 6502 Upward-Compatible | **Vendor Utama:** WDC
     * **Deskripsi:** Evolusi 16-bit dari MOS 6502 dengan pengalamatan memori 24-bit (16 MB) yang menggerakkan konsol legendaris Super Nintendo Entertainment System (SNES / Ricoh 5A22) dan Apple IIGS.

---

### KATEGORI 18: ARSITEKTUR HETEROGEN & AI MESH MODERN (149–150)

149. **Cell Broadband Engine (Cell B.E.)**
     * **Tipe:** Heterogeneous Multicore Architecture | **Vendor Utama:** Sony, Toshiba, IBM (STI Alliance)
     * **Deskripsi:** Menggabungkan satu core PowerPC 64-bit (Power Processing Element / PPE) dengan 8 unit prosesor vektor SIMD co-processing independen (Synergistic Processing Elements / SPE); otak konsol Sony PlayStation 3 dan superkomputer IBM Roadrunner (superkomputer pertama yang menembus 1 PFLOPS).

150. **Tenstorrent Wormhole / Blackhole (Tensix Cores)**
     * **Tipe:** AI RISC-V Mesh Architecture | **Vendor Utama:** Tenstorrent (Jim Keller)
     * **Deskripsi:** Arsitektur mutakhir yang menggabungkan cluster core RISC-V dengan array prosesor tensor (Tensix Cores) yang saling terhubung di atas kisi Network-on-Chip (NoC) dua dimensi untuk pelatihan dan inferensi model AI terdistribusi tanpa bottleneck bus PCI.

---

## MATRIKS REKAPITULASI BRAND JARINGAN & SURVEILLANCE

| Brand Perangkat | Arsitektur Utama yang Digunakan | Chipset / Vendor Silikon Teridentifikasi |
| :--- | :--- | :--- |
| **MikroTik** | MIPS32 (`mipsbe`), ARMv7 (`arm`), ARMv8 (`arm64`), Tilera (`tile`), x86-64 | Qualcomm Atheros, Marvell, Annapurna Labs, Tilera TILE-Gx, Intel |
| **Cisco Systems** | x86-64, MIPS64, ARMv8, Custom NPU (QFP, Silicon One, UADP) | Intel Xeon/Atom, Cisco QFP, Cavium OCTEON, Broadcom |
| **Ubiquiti UniFi** | ARMv8-A (64-bit), ARMv7-A, MIPS64, MIPS32 | Annapurna Labs Alpine, Qualcomm IPQ, MediaTek MT7621, Cavium OCTEON |
| **TP-Link** | MIPS32, ARMv7-A, ARMv8-A (64-bit) | MediaTek (MT7621/Filogic), Qualcomm Networking Pro, Broadcom BCM49xx |
| **D-Link** | MIPS32, ARMv7-A, ARMv8-A (64-bit) | Realtek RTL819x/RTL83xx, MediaTek Filogic, Broadcom |
| **DVR / NVR** | ARMv7-A, ARMv8-A, MIPS32 + Hardware VPU/NPU | HiSilicon DaVinci, Novatek, Rockchip, SigmaStar, Ambarella |
| **Nokia** | x86-64, ARMv8-A, Custom NPU (FP4/FP5, Quillion, ReefShark) | Nokia FP5 Silicon, Nokia Quillion, ReefShark, Intel Xeon |
| **ZTE** | x86-64, ARMv8-A, ARMv7-A (Cortex-A9), Custom NPU | ZTE Sanechips (ZXIC) ZX2791xx, Broadcom StrataDNX |
| **Huawei ("Hawaii")**| ARMv8-A (Kunpeng 920), ARMv7-A, Custom NPU (Solar, Tiangang) | HiSilicon (Solar 5/6 NPU, Gigahome, Balong, Tiangang 5G) |
