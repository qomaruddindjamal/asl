# Katalog 120 Arsitektur CPU, Mikroprosesor, SoC, & Board (ISA / Hardware Architectures)

Dokumen ini memuat taksonomi lengkap **120 Arsitektur Hardware, Instruction Set Architecture (ISA), dan Platform Board/SoC** yang digunakan di dunia komputasi—mulai dari prosesor komersial modern, embedded system, mikrokontroler, DSP, mainframe, retro/vintage, hingga arsitektur open-source & akselerator modern.

---

## 1. Arsitektur x86 / CISC Klasik & Modern (1–12)

1. **x86-64 (AMD64 / Intel 64)**: Arsitektur 64-bit standar industri desktop, laptop, server, dan superkomputer.
2. **x86-32 (IA-32 / i386)**: Arsitektur 32-bit x86 legendaris Intel (80386 hingga Pentium/Core series awal).
3. **Intel 8086 / 8088**: Fondasi keluarga 16-bit x86, prosesor awal IBM PC asli.
4. **Intel 80186**: Evolusi x86 16-bit dengan peripheral terintegrasi, populer untuk embedded controllers.
5. **Intel 80286**: Prosesor x86 16-bit pertama dengan Protected Mode dan MMU virtual memory addressing.
6. **Intel IA-64 (Itanium / EPIC)**: Explicitly Parallel Instruction Computing 64-bit untuk enterprise server.
7. **AMD K6 / 3DNow!**: Varian x86 AMD dengan instruksi multimedia SIMD perintis sebelum SSE.
8. **Transmeta Crusoe (VLIW + Code Morphing)**: Arsitektur internal VLIW dengan dynamic binary translation ke x86.
9. **Transmeta Efficeon**: Generasi kedua VLIW 256-bit engine yang menerjemahkan x86 secara hardware-software.
10. **Cyrix 6x86 / MII**: Mikroarsitektur x86 superscalar kompatibel soket Socket 7 dengan instruksi efisiensi tinggi.
11. **VIA C3 / Eden (Centaur Architecture)**: Arsitektur mikro x86 daya ultra-rendah untuk thin client dan industri.
12. **VIA Nano (Isaiah)**: Arsitektur 64-bit superscalar out-of-order x86 dari VIA/Centaur untuk komputasi embedded.

---

## 2. Keluarga Arm Architecture (13–28)

13. **AArch64 (ARMv8-A / ARMv9-A)**: Arsitektur 64-bit Arm untuk smartphone modern, Apple Silicon, server cloud (Graviton, Neoverse), dan SBC (Raspberry Pi 4/5).
14. **AArch32 (ARMv7-A)**: Arsitektur 32-bit Cortex-A (Cortex-A7, A9, A15) standar smartphone generasi 3G/4G awal dan SBC (Raspberry Pi 2).
15. **ARMv6 (ARM11)**: Digunakan pada Raspberry Pi Generasi 1, Pi Zero, dan iPhone 2G/3G pertama.
16. **ARMv5TE / ARM9**: Arsitektur embedded klasik 32-bit dengan ekstensi DSP, digunakan luas pada router dan feature phone.
17. **ARMv4T / ARM7TDMI**: Arsitektur mikrokontroler legendaris dengan Thumb instruction set (contoh: Nintendo Game Boy Advance).
18. **ARM Cortex-M0 / M0+ (ARMv6-M)**: Core ultra-low power untuk mikrokontroler hemat energi (RP2040, SAMD21).
19. **ARM Cortex-M3 (ARMv7-M)**: Inti mikrokontroler 32-bit mainstream (STM32F103 "Blue Pill").
20. **ARM Cortex-M4 / M4F (ARMv7E-M)**: Inti mikrokontroler dengan instruksi DSP hardware dan Floating Point Unit (STM32F4).
21. **ARM Cortex-M7 (ARMv7E-M)**: Mikrokontroler performa tinggi dengan superscalar pipeline & cache L1 (STM32H7, Teensy 4.0).
22. **ARM Cortex-M33 (ARMv8-M Mainline)**: Mikrokontroler modern dengan proteksi keamanan TrustZone hardware (RP2350, nRF5340).
23. **ARM Cortex-M55 / M85 (ARMv8.1-M)**: Mikrokontroler dengan akselerasi Helium Vector DSP untuk embedded AI/ML (TinyML).
24. **ARM Cortex-R4 / R5 (ARMv7-R)**: Arsitektur Real-Time deterministik untuk kontrol mesin otomotif, rem ABS, dan baseband 4G.
25. **ARM Cortex-R8 / R52 / R82 (ARMv8-R)**: Arsitektur Real-Time 64-bit dengan hardware virtualization untuk mission-critical & ASIL-D.
26. **ARM Cortex-A53 / A55**: Inti efisiensi daya tinggi 64-bit, backbone kebanyakan SoC TV box, router, dan SBC.
27. **ARM Cortex-A76 / A78 / X-series**: Inti performa tinggi 64-bit untuk laptop ARM dan flagship SoC (Snapdragon, Dimensity).
28. **ARM SecurCore (SC000 / SC300)**: Varian anti-tamper bersertifikasi keamanan tinggi untuk smartcard, kartu SIM, dan modul paspor elektronik.

---

## 3. RISC-V Open Architecture (29–42)

29. **RV32I**: Basis Integer 32-bit instruksi minimalis (hanya 47 instruksi dasar).
30. **RV32EC / RV32E**: Varian embedded dengan 16 register untuk mikrokontroler silikon hemat footprint.
31. **RV32IMAC**: Varian 32-bit dengan Integer Multiplication (M), Atomic (A), dan Compressed 16-bit instructions (C) (contoh: ESP32-C3).
32. **RV32IMAFDC (RV32GC)**: Varian lengkap General-Purpose 32-bit dengan Single (F) dan Double (D) precision FPU.
33. **RV64I**: Basis 64-bit Integer standar untuk komputasi general-purpose.
34. **RV64GC (RV64IMAFDC)**: Standar de facto sistem operasi Linux RISC-V 64-bit (SiFive FU740, Allwinner D1, StarFive JH7110).
35. **RV64GCV (Vector Extension)**: RISC-V 64-bit dengan ekstensi SIMD Vector dinamis untuk komputasi AI/ML dan DSP (SpacemiT K1).
36. **RV128I**: Spesifikasi instruksi 128-bit untuk addressing memori skala exabyte/zettabyte masa depan.
37. **SiFive E-Core (E20 / E31 / E76)**: Implementasi core mikrokontroler real-time 32-bit hemat daya.
38. **SiFive U-Core (U54 / U74 / P550)**: Inti Linux-capable superscalar application processor.
39. **T-Head XuanTie C906 / C910**: Mikroarsitektur RISC-V open-source performa tinggi dari Alibaba untuk cloud dan edge SBC.
40. **CH32V (WCH QingKe Core)**: Mikrokontroler RISC-V ultra-murah pengganti mikrokontroler 8-bit klasik (CH32V003).
41. **Western Digital SweRV (EH1/EH2/EL2)**: Arsitektur RISC-V dual-issue superscalar internal untuk storage controller SSD.
42. **PULP Platform (PULPino / RI5CY)**: Arsitektur riset RISC-V ultra-low-power paralel dari ETH Zürich / University of Bologna.

---

## 4. MIPS Architecture (43–52)

43. **MIPS I (R2000 / R3000)**: Arsitektur 32-bit RISC pertama, prosesor konsol legendaris Sony PlayStation 1.
44. **MIPS II (R6000)**: Penambahan instruksi load-linked/store-conditional untuk multiprosesing.
45. **MIPS III (R4000 / VR4300)**: Arsitektur 64-bit RISC awal komersial, digunakan pada konsol Nintendo 64 dan workstation SGI.
46. **MIPS IV (R8000 / R10000)**: Superscalar out-of-order 64-bit dengan FPU canggih untuk superkomputer grafis SGI Onyx.
47. **MIPS V**: Standar 64-bit dengan instruksi pasangan floating point (paired-single) dan MIPS-3D.
48. **MIPS32 (Release 1 – 6)**: Standar modern 32-bit yang mendominasi jutaan router WiFi komersial (Broadcom, Atheros, MediaTek).
49. **MIPS64 (Release 1 – 6)**: Standar modern 64-bit untuk edge network gateway dan router core.
50. **MIPS microMIPS**: Arsitektur instruksi variabel 16/32-bit hemat memori untuk microcontroller PIC32MZ.
51. **Ingenic XBurst**: Varian MIPS32 ultra-low-power berkecepatan tinggi untuk SoC wearable dan kamera IP smart CCTV.
52. **Loongson-1 / 2 (GS232 / GS464)**: Implementasi MIPS kompatibel dari Tiongkok sebelum transisi ke LoongArch.

---

## 5. Power Architecture & PowerPC (53–62)

53. **IBM POWER1 / POWER2**: Arsitektur RISC multi-chip perintis untuk server RISC System/6000 (RS/6000).
54. **PowerPC 601 / 603 / 604 (G2/G3)**: Kolaborasi Apple-IBM-Motorola (AIM Alliance) untuk komputer Apple Macintosh klasik.
55. **PowerPC G4 (MPC7400 / AltiVec)**: Prosesor PowerPC dengan vektor coprocessor SIMD Velocity Engine/AltiVec pertama.
56. **PowerPC 970 (PowerPC G5)**: Prosesor PowerPC 64-bit turunan POWER4 untuk Apple Power Mac G5.
57. **IBM POWER9**: Prosesor enterprise 64-bit pendukung superkomputer Summit dan Sierra, mendukung OpenPOWER Foundation.
58. **IBM POWER10**: Prosesor 7nm dengan Matrix Math Accelerator (MMA) untuk enterprise cloud dan analitik AI.
59. **PowerPC 750CL / Broadway**: Custom core PowerPC 32-bit pada konsol Nintendo GameCube dan Nintendo Wii.
60. **PowerPC Espresso**: CPU tri-core PowerPC 32-bit pada konsol Nintendo Wii U.
61. **Xenon (PowerPC)**: CPU tri-core 64-bit 3.2 GHz dengan VMX128 pada konsol Microsoft Xbox 360.
62. **Power Architecture e200 / e500 (QorIQ)**: Arsitektur PowerPC real-time untuk ECU kontrol otomotif dan jaringan industri.

---

## 6. SPARC Family (63–70)

63. **SPARC V7**: Arsitektur RISC 32-bit pertama Sun Microsystems berbasis konsep Berkeley RISC.
64. **SPARC V8**: Arsitektur 32-bit standar IEEE 1754, digunakan pada workstation Sun SPARCstation dan server solaris.
65. **LEON2 / LEON3 / LEON4**: Arsitektur SPARC V8 radiation-hardened standar European Space Agency (ESA) untuk satelit dan pesawat luar angkasa.
66. **SPARC V9 (UltraSPARC I – IV)**: Arsitektur 64-bit dengan instruksi VIS (Visual Instruction Set) untuk multimedia.
67. **UltraSPARC T1 / T2 (Niagara / CoolThreads)**: Arsitektur multithreading masif (hingga 64 threads) hemat daya untuk web-server throughput.
68. **SPARC64 (Fujitsu VI – X+)**: Mikroarsitektur performa superkomputer dari Fujitsu untuk enterprise mainframe.
69. **Fujitsu SPARC64 VIIIfx / IXfx**: Prosesor yang menggerakkan superkomputer tercepat dunia K-Computer di RIKEN Jepang.
70. **OpenSPARC**: Spesifikasi dan source RTL Verilog open-source dari UltraSPARC T1/T2 yang dirilis oleh Sun Microsystems.

---

## 7. Mainframe & Arsitektur Enterprise Bersejarah (71–78)

71. **IBM System/360 (S/360)**: Arsitektur komputer legendaris 1964 yang menetapkan standar byte 8-bit dan alamat 32-bit.
72. **IBM System/370 (S/370)**: Evolusi S/360 dengan dukungan memori virtual dinamis dan multiprosesing.
73. **IBM ESA/390 (System/390)**: Arsitektur enterprise 31-bit dengan Enterprise Systems Architecture.
74. **IBM z/Architecture (s390x)**: Arsitektur mainframe 64-bit modern tanpa batas kompatibilitas ke belakang (IBM z16).
75. **DEC Alpha (Alpha AXP / 21064, 21164, 21264)**: Arsitektur 64-bit murni super-cepat yang memecahkan rekor clock rate tahun 1990-an.
76. **HP PA-RISC (Precision Architecture)**: Arsitektur workstation dan server HP-UX performa tinggi (PA-7000 hingga PA-8900).
77. **Motorola 88000 (m88k)**: Usaha awal arsitektur RISC 32-bit dari Motorola sebelum bergabung ke aliansi PowerPC.
78. **CDC 6600**: Arsitektur superkomputer pertama di dunia rancangan Seymour Cray dengan 10 peripheral processor independen.

---

## 8. Arsitektur Mikrokontroler & Embedded Klasik / Kontemporer (79–92)

79. **Microchip PIC16 / PIC18 (Mid-Range & Enhanced)**: Arsitektur Harvard 8-bit yang sangat populer di industri otomasi dan perakitan.
80. **Microchip PIC24 / dsPIC33**: Arsitektur 16-bit dengan instruksi matematika DSP satu-siklus untuk kontrol motor dan konversi daya.
81. **Microchip PIC32 (MIPS / SAM)**: Mikrokontroler 32-bit performa tinggi berbasis MIPS32 dan ARM Cortex.
82. **Atmel AVR (8-bit / ATmega328P)**: Arsitektur RISC 8-bit instruksi 1-siklus, otak board legendaris Arduino Uno.
83. **AVR32 (AP7 / UC3)**: Arsitektur RISC 32-bit performa tinggi dari Atmel untuk embedded multimedia dan kontrol industri.
84. **Intel 8051 (MCS-51)**: Arsitektur mikrokontroler 8-bit paling banyak diproduksi dalam sejarah silikon dunia.
85. **Espressif Xtensa LX6 (Tensilica)**: Arsitektur 32-bit dual-core Harvard pada SoC Wi-Fi/Bluetooth ESP32.
86. **Espressif Xtensa LX7**: Generasi lanjutan Tensilica dengan instruksi vektor SIMD untuk akselerasi neural network pada ESP32-S3.
87. **Renesas RL78**: Arsitektur mikrokontroler 16-bit ultra-low-power untuk perangkat meteran medis dan rumah tangga.
88. **Renesas RX (RXv1 – RXv3)**: Arsitektur CISC 32-bit proprietary dengan efisiensi eksekusi tinggi untuk kontrol industri.
89. **Renesas V850 / RH850**: Arsitektur RISC 32-bit deterministik khusus ECU otomotif (dashboard, gateway CAN, sensor).
90. **Hitachi/Renesas SuperH SH-2**: RISC 32-bit dual-processor konsol Sega Saturn dan mesin arcade Sega ST-V.
91. **Hitachi/Renesas SuperH SH-4**: RISC 32-bit dengan 128-bit vector floating point 3D, penggerak konsol Sega Dreamcast dan Naomi.
92. **Infineon TriCore**: Arsitektur terpadu 32-bit (RISC + DSP + MCU) standar de facto ECU powertrain & keselamatan mobil global.

---

## 9. Arsitektur Mikroprosesor Retro & Vintage (93–102)

93. **MOS Technology 6502**: Mikroprosesor 8-bit perintis revolusi PC (Apple II, Commodore 64, Nintendo NES/Famicom).
94. **Zilog Z80**: Mikroprosesor 8-bit kompatibel 8080 (ZX Spectrum, MSX, Sega Master System, Game Boy sound CPU).
95. **Motorola 6800**: Mikroprosesor 8-bit awal dengan register akumulator kembar, basis banyak sistem arcade dan mesin pinball.
96. **Motorola 6809**: Mikroprosesor 8-bit paling canggih dengan register 16-bit dan pengalamatan ortogonal (Dragon 32, Tandy CoCo).
97. **Motorola 68000 (m68k)**: Mikroarsitektur CISC 16/32-bit legendaris (Commodore Amiga, Atari ST, Sega Genesis/Mega Drive, Apple Macintosh 128K).
98. **Motorola 68020 / 68030**: Varian 32-bit murni 68k dengan cache on-chip dan unit paging MMU (Amiga 1200/3000, Sun-3).
99. **Motorola 68040 / 68060**: Varian 68k dengan pipeline superscalar dan FPU hardware terintegrasi (Amiga 4000, NeXTstation).
100. **Intel 4004**: Mikroprosesor komersial monolithic pertama di dunia (4-bit) untuk kalkulator Busicom.
101. **Intel 8008**: Mikroprosesor 8-bit pertama dari Intel dengan arsitektur memori stack internal.
102. **Intel 8080 / 8085**: Mikroprosesor 8-bit yang melahirkan era sistem operasi CP/M dan komputer Altair 8800.

---

## 10. Arsitektur DSP (Digital Signal Processor) (103–110)

103. **Texas Instruments TMS320C2000 (C28x)**: Arsitektur DSP 32-bit real-time untuk kontrol inverter motor tenaga surya dan EV charger.
104. **Texas Instruments TMS320C6000 (C66x / C67x)**: Arsitektur VLIW performa tinggi untuk radar militer, sonar, dan base station seluler.
105. **Analog Devices SHARC (ADSP-21xxx)**: Arsitektur floating-point DSP 32/40-bit dengan memory Super Harvard (SHARC) untuk audio pro.
106. **Analog Devices Blackfin**: Prosesor terpadu 16/32-bit menggabungkan DSP dan kontrol mikroprosesor untuk telekomunikasi.
107. **Motorola / NXP DSP56000**: Arsitektur DSP 24-bit legendaris untuk synthesizer musik, workstation audio, dan NeXT Computer.
108. **Qualcomm Hexagon DSP (QDSP6)**: Arsitektur multi-threaded VLIW dalam Snapdragon SoC untuk pemrosesan modem 5G, kamera, dan audio.
109. **CEVA-TeakLite / SensPro**: Arsitektur DSP IP core berdaya rendah untuk sensor hub dan audio codec di smartphone.
110. **Cadence Tensilica HiFi DSP (HiFi 3 / 4 / 5)**: Arsitektur audio dan speech AI processor berdaya rendah yang diadopsi secara luas di SoC TV dan TWS earphone.

---

## 11. Arsitektur GPU & Akselerator Komputasi Paralel (111–116)

111. **NVIDIA CUDA / PTX (Parallel Thread Execution)**: ISA virtual perantara untuk eksekusi massive SIMT pada GPU NVIDIA (dari Kepler hingga Blackwell sm_120).
112. **AMD GCN (Graphics Core Next)**: Arsitektur compute SIMD berbasis Wavefront 64 untuk kartu grafis Radeon dan konsol PlayStation 4 / Xbox One.
113. **AMD RDNA (Radeon DNA)**: Arsitektur grafis berfokus latency rendah dengan Wavefront 32 (RDNA 1, 2, 3) untuk PC gaming dan PS5.
114. **AMD CDNA (Compute DNA)**: Arsitektur GPU akselerator datacenter & superkomputer untuk AI (Instinct MI250X, MI300A/X).
115. **Intel Xe / Xe-HPC (Ponte Vecchio)**: Arsitektur mikro GPU skalabel dari integrasi low-power hingga superkomputer Aurora.
116. **Google TPU (Tensor Processing Unit / V1 – V5p / Trillium)**: Arsitektur ASIC Systolic Array terdedikasi untuk akselerasi perkalian matriks bfloat16/int8 deep learning.

---

## 12. Arsitektur Khusus, Asinkron, & Emerging Modern (117–120)

117. **LoongArch (Loongson Technology)**: ISA mandiri 32/64-bit modern dari Tiongkok (LA464/LA664) yang dirancang untuk kompatibilitas biner lintas arsitektur x86/ARM/MIPS.
118. **Kalray MPPA (Massively Parallel Processor Array)**: Arsitektur Manycore Many-Processor VLIW (ratusan core pada satu silikon) untuk komputasi deterministik otomotif & edge AI.
119. **XMOS xCORE (Logical Core Multithreading)**: Arsitektur deterministik 32-bit yang mengimplementasikan protokol I/O hardware secara langsung via software real-time threads.
120. **Tilera / Mellanox TILE-Gx (Tile Architecture)**: Arsitektur prosesor mesh network-on-chip dengan puluhan hingga ratusan tile core RISC untuk packet processing jaringan kecepatan tinggi (SmartNIC).

---

## Matriks Rekapitulasi Berdasarkan Paradigma Eksekusi

| Paradigma | Contoh Arsitektur | Karakteristik Utama |
| :--- | :--- | :--- |
| **CISC** | x86-64, m68k, PDP-11, VAX, z/Architecture | Panjang instruksi variabel, operasi memori-ke-register kompleks |
| **RISC** | Arm, RISC-V, MIPS, SPARC, PowerPC, SuperH, PA-RISC | Instruksi fixed-length seragam, arsitektur Load/Store murni |
| **VLIW** | TI C6000, Transmeta Crusoe, Qualcomm Hexagon, Intel Itanium | Compiler menjadwalkan paralelisme instruksi ke multiple execution slot |
| **SIMD / Vector** | RISC-V Vector, Arm Neon/Helium, Cray-1 | Operasi tunggal diterapkan serempak pada array data besar |
| **SIMT** | NVIDIA CUDA PTX, AMD GCN/RDNA | Ribuan thread paralel dieksekusi serentak melalui lock-step warp/wavefront |
| **Systolic Array** | Google TPU | Aliran data langsung antar-unit komputasi tanpa read/write register terus-menerus |
