# Analisis Lengkap Arsitektur Hardware & Chipset Perangkat Jaringan & Surveillance

Dokumen ini memetakan arsitektur CPU/SoC, Instruction Set Architecture (ISA), vendor silikon, dan sistem operasi internal untuk:
1. MikroTik
2. Cisco Systems
3. Ubiquiti UniFi
4. TP-Link
5. D-Link
6. DVR / NVR (Hikvision, Dahua, dll.)
7. Nokia Networks
8. ZTE
9. Huawei ("Hawaii")

---

## 1. MIKROTIK (RouterBOARD & Cloud Core Router)

MikroTik menggunakan beragam arsitektur tergantung kelas perangkat dan generasinya di sistem operasi **RouterOS (v6 & v7)**:

| Seri / Perangkat | Arsitektur RouterOS | Instruction Set (ISA) | Vendor & Model SoC/CPU | Karakteristik & Contoh Seri |
| :--- | :--- | :--- | :--- | :--- |
| **RouterBOARD SOHO / Budget** | `mipsbe` (MIPS Big Endian) | **MIPS32 Release 2** | Qualcomm Atheros (AR9344, QCA9531, AR7241) | RB750r2 (hEX lite), RB951Ui-2HnD, BaseBox |
| **Router SOHO / Tower Modern** | `arm` (ARM 32-bit) | **ARMv7-A (32-bit)** | Qualcomm IPQ4018 / IPQ4019, Annapurna Labs Alpine AL214 | hEX S (RB760iGS), cAP ac, RB3011UiAS, RB4011 |
| **Router Generasi Baru (Wi-Fi 6)** | `arm64` (ARM 64-bit) | **ARMv8-A (AArch64)** | Qualcomm IPQ6010, IPQ5018, MediaTek MT7981/MT7986 | hEX (RB750Gr3 v2), hAP ax2, hAP ax3, L009UiGS, RB5009UG+S+IN |
| **Cloud Router Switch (CRS)** | `arm`, `mipsbe` | **ARMv7-A** / **MIPS32** | Marvell Prestera, Realtek Switch ASIC | CRS326, CRS328, CRS305 (CPU manajemen switch) |
| **Cloud Core Router (CCR Legacy)**| `tile` (Tilera) | **TILE-Gx (VLIW Manycore)** | Tilera TILE-Gx8009, TILE-Gx8016, TILE-Gx8036, TILE-Gx8072 | CCR1009, CCR1016, CCR1036 (36 core), CCR1072 (72 core) |
| **Cloud Core Router (CCR Modern)**| `arm64` (AArch64) | **ARMv8-A 64-bit** | Marvell Armada 7040 / 8040, Annapurna Labs AL73400 (16-core Cortex-A72) | CCR2004, CCR2116-12G-4S+, CCR2216 (16-core A72 @ 2GHz + Marvell Switch ASIC) |
| **Wireless Long Range / Low End** | `smips` | **MIPS32 (16-bit compressed)** | Microchip/Atheros MIPS | hAP mini, hAP lite (QCA9533) |
| **RouterBOARD Klasik** | `mipsle`, `ppc` | **MIPS Little Endian** / **PowerPC** | Freescale/NXP PowerPC MPC8544, AMD Geode x86 | RB1000, RB1100AHx2 (PowerPC), RB100 |
| **Virtual Appliance (CHR / PC)** | `x86_64` | **x86-64 (AMD64)** | Intel Xeon, Core i-series, AMD EPYC | Cloud Hosted Router (CHR) di Proxmox/VMware, x86 PC RouterOS |

---

## 2. CISCO SYSTEMS (Enterprise Router & Switch)

Cisco memisahkan arsitektur menjadi **Control Plane** (CPU sistem operasi IOS / IOS-XE) dan **Data Plane** (ASIC/NPU penerusan paket jalur cepat):

| Kategori Perangkat | Arsitektur CPU (Control Plane) | Komponen Data Plane (Forwarding ASIC) | Vendor / Jenis Prosesor | Contoh Perangkat |
| :--- | :--- | :--- | :--- | :--- |
| **Cisco ISR 1000 / 4000 Series** | **x86-64** (Intel multi-core) | Cisco **QuantumFlow Processor (QFP)** (NPU khusus Cisco) | Intel Atom C2000/C3000, Xeon D | ISR 4321, 4331, 4451 (Control plane menjalankan Linux + IOS-XE container) |
| **Cisco Catalyst 8000 (Edge/SD-WAN)**| **x86-64** | Cisco **QFP 3.0** / Silicon One | Intel Xeon-D series | C8200, C8300, C8500 Edge Platforms |
| **Cisco Catalyst 9000 Switches** | **x86-64** (sebelumnya ARMv8) | Cisco **UADP (Unified Access Data Plane 2.0/3.0)** ASIC | Intel multi-core x86-64 CPU | Catalyst 9200, 9300, 9400, 9500 |
| **Cisco ASR 1000 / 9000 Series** | **x86-64** (Route Processor) | Cisco **QuantumFlow Processor (QFP)** & Custom Network Processors | Intel multi-core Xeon | ASR 1001-X, ASR 1002-HX, ASR 9000 |
| **Cisco 8000 Series (Cloud/Carrier)**| **x86-64** | Cisco **Silicon One** (ASIC arsitektur terpadu hingga 51.2 Tbps) | Intel Xeon SP | Cisco 8100, 8200, 8800 (Telco Core) |
| **Cisco ISR Generasi Lama (Legacy)** | **MIPS64** / **PowerPC** | Hardware crypto engine terintegrasi | Cavium OCTEON (MIPS64), Freescale PowerPC MPC8500 | Cisco 1921, 2901, 3945, 7200 Series |
| **Cisco Meraki (Cloud-Managed)** | **ARMv7-A** / **ARMv8-A** (AArch64) | Marvell / Broadcom Switching ASIC | Qualcomm IPQ, Broadcom StrataGX, Annapurna Labs | Meraki MX64/MX68, MX84, MR access points |

---

## 3. UBIQUITI UNIFI & EDGEMAX

Ubiquiti bertransisi dari arsitektur MIPS murni di era EdgeRouter menuju arsitektur ARM 64-bit pada ekosistem UniFi OS:

| Seri / Lini Produk | Arsitektur CPU | ISA & Bit | Vendor & Model SoC | Contoh Perangkat |
| :--- | :--- | :--- | :--- | :--- |
| **UniFi Dream Machine (UDM / UCG)** | **ARMv8-A (AArch64)** | 64-bit ARM | Annapurna Labs Alpine AL324 (Quad-Core Cortex-A57 @ 1.7GHz), Rockchip RK3399 | UDM-Pro, UDM-SE, UniFi Cloud Gateway Ultra (UCG-Ultra), UXG-Pro |
| **UniFi Express & Access Points Wi-Fi 6/7** | **ARMv8-A** / **ARMv7-A** | 64-bit / 32-bit ARM | Qualcomm Networking Pro (IPQ5018, IPQ6018, IPQ8072), MediaTek Filogic | U6-Lite, U6-Pro, U7-Pro, UniFi Express (UX) |
| **UniFi Access Points (Legacy AC)** | **MIPS32** | 32-bit MIPS | Qualcomm Atheros (QCA9563, AR9342) | UAP-AC-Lite, UAP-AC-LR, UAP-AC-Pro |
| **UniFi Security Gateway (USG Legacy)** | **MIPS64** | 64-bit MIPS | Cavium OCTEON II (CN5020 Dual-Core MIPS64) | USG-3P, USG-PRO-4 |
| **EdgeRouter Series (EdgeMAX)** | **MIPS64** (mips64/octeon) | 64-bit MIPS | Cavium OCTEON II / III (CN6120, CN7130 Multi-Core MIPS) | EdgeRouter Lite (ERLite-3), ER-4, ER-6P, ER-8, ER-Infinity |
| **EdgeRouter X Series** | **MIPS32** (mipsel) | 32-bit MIPS LE | MediaTek MT7621AT (Dual-core 4-threads MIPS 1004Kc @ 880MHz) | ER-X, ER-X-SFP |

---

## 4. TP-LINK (Home, Archer, Deco, & Omada Enterprise)

TP-Link adalah pengguna volume terbesar SoC MediaTek dan Qualcomm di dunia networking:

| Kategori Produk | Arsitektur CPU | ISA | Vendor & Model SoC | Contoh Perangkat |
| :--- | :--- | :--- | :--- | :--- |
| **TP-Link Entry / Budget (WR series)** | **MIPS32** (mipsbe / mipsel) | 32-bit MIPS | Qualcomm Atheros (QCA9533), MediaTek (MT7628N, MT7620) | TL-WR840N, TL-WR841N, TL-WR940N |
| **Archer Mainstream (Gigabit AC/AX)** | **MIPS32** / **ARMv7-A** | 32-bit MIPS / ARM | MediaTek MT7621AT, Broadcom BCM47189, Qualcomm IPQ0505 | Archer C6, Archer A6, Archer AX10, Archer AX20 |
| **Archer High-End & Wi-Fi 7 (AX/BE)** | **ARMv8-A (AArch64)** | 64-bit ARM | Broadcom BCM4908 / BCM4912, Qualcomm IPQ8074, MediaTek MT7988 (Filogic 880) | Archer AX73, Archer AX90, Archer BE800, Archer BE900 |
| **Deco Mesh Wi-Fi Systems** | **ARMv7-A** / **ARMv8-A** | 32-bit & 64-bit ARM | Qualcomm Networking Pro, Realtek RTL8198D, Broadcom | Deco M4, Deco M5, Deco X20, Deco X50, Deco BE65 |
| **Omada Gateway (SDN Enterprise)** | **MIPS32** / **ARMv8-A** | MIPS / 64-bit ARM | MediaTek MT7621A (ER605), NXP / Rockchip ARM (ER7206, ER8411) | ER605 (MT7621 Dual MIPS), ER7206, ER8411 (Quad-Core ARM A72) |

---

## 5. D-LINK (Consumer & D-Link DGS/DIR Series)

D-Link menggunakan strategi multi-vendor silikon yang sangat mirip dengan TP-Link:

| Kategori Produk | Arsitektur CPU | ISA | Vendor & Model SoC | Contoh Perangkat |
| :--- | :--- | :--- | :--- | :--- |
| **DIR Series Entry SOHO** | **MIPS32** | 32-bit MIPS | MediaTek MT7628, Realtek RTL8196E / RTL8197D | DIR-612, DIR-615 |
| **DIR Gigabit & EXO Series (AC/AX)**| **MIPS32** / **ARMv7-A** | 32-bit MIPS / ARM | MediaTek MT7621AT, Realtek RTL8197F, Broadcom BCM4709 | DIR-825, DIR-842, DIR-882, DIR-X1560 |
| **Eagle Pro AI / Wi-Fi 6 Series** | **ARMv7-A** / **ARMv8-A** | 32-bit & 64-bit ARM | MediaTek Filogic 820/830, Realtek RTL8198D | DIR-X1860, M15 Mesh, R15 Router |
| **D-Link DGS Smart Switch Managed** | **MIPS32** / **ARMv7-A** | 32-bit | Realtek RTL8380 / RTL8382M, Broadcom BCM53344 | DGS-1210 series, DGS-1510 series |

---

## 6. DVR & NVR (SURVEILLANCE RECORDER)
*(Hikvision, Dahua, Uniview, Tiandy, XM/Xiongmai)*

Perangkat perekam kamera CCTV (DVR analog & NVR IP) memiliki arsitektur unik: kombinasi **CPU Core (ARM/MIPS)** untuk OS Linux + **Hardware ISP/Video Codec Engine** (H.264/H.265 NPU) untuk decode/encode multi-channel:

| Vendor Silikon / Chipset | Arsitektur CPU Core | Unit Akselerasi Khusus | Contoh Merk & Seri DVR/NVR |
| :--- | :--- | :--- | :--- |
| **HiSilicon (Huawei)** *(Mendominasi 70%+ pasar sebelum 2020)* | **ARM Cortex-A7 / Cortex-A9 / Cortex-A17** (32-bit), **Cortex-A53** (64-bit) | VPU H.264/H.265 hardware decoder + DaVinci NPU AI | Seri Hi3520D, Hi3521A, Hi3531D, Hi3535, Hi3536 (Hikvision DS-7600/7700 series, Dahua NVR4000/5000) |
| **Novatek (Taiwan)** *(Pengganti utama HiSilicon)* | **ARM Cortex-A9** (NT9852x) / **ARM Cortex-A53 Dual/Quad** (NT9832x, NT9833x) | Hardware Multi-channel Video Codec + 1.5–3.0 TOPS NPU | Hikvision Seri K/M modern (DS-7608NI-K2/M2), Dahua WizSense NVR |
| **Rockchip (Tiongkok)** | **ARMv8-A 64-bit** (Quad Cortex-A55) | 4K/8K Video Decode + 2.0–6.0 TOPS NPU | Rockchip RK3568, RK3588 (NVR kelas AI pengenal wajah Hikvision/Uniview) |
| **SigmaStar (Eks-MStar)** | **ARM Cortex-A7** Dual Core / **Cortex-A53** | Smart Codec H.265+ ISP Engine | SigmaStar SSC338G, SSC8838G (DVR 4-16 channel budget Hikvision Turbo HD / Dahua Cooper) |
| **Ambarella (USA)** | **ARMv8-A (AArch64)** Cortex-A53 | Ambarella CVflow AI Vision Engine | NVR enterprise high-end Axis, Bosch, Hanwha Vision, Hikvision DeepinMind |
| **Allwinner / Xiongmai (XM)** | **ARM Cortex-A7** / **MIPS32** (Ingenic) | Codec H.264/H.265 entry level | DVR/NVR OEM tanpa merk, board Xiongmai (XM-530, XM-550) |

---

## 7. NOKIA NETWORKS (Telco Core, OLT GPON, IP Routing)

Nokia adalah raksasa infrastruktur operator telekomunikasi (Base Station 5G, GPON OLT, dan IP Core Routing):

| Kategori Perangkat | Arsitektur CPU (Control Plane) | Arsitektur Silikon Data Plane (Network Processor) | Keterangan & Contoh Perangkat |
| :--- | :--- | :--- | :--- |
| **IP Core & Edge Routers** *(Nokia 7750 SR)* | **x86-64** (Intel multi-core Xeon) | **Nokia FP-Series (FP4, FP5 Silicon)** *(Network Processing Unit kustom Nokia)* | Nokia 7750 Service Router (SR-1, SR-7s, SR-14s), FP5 beroperasi hingga 4.8 Tbps per chip dengan enkripsi line-rate ANYsec |
| **Optical OLT GPON / XGS-PON** | **x86-64** / **ARMv8-A** (AArch64) | Nokia **Quillion Chipset** (ASIC hemat energi generasi baru) | Nokia ISAM 7360 FX, Nokia Lightspan FX-4 / FX-8 / MF-14 (OLT fiber optic FTTH Telkom/Indosat) |
| **Base Station 5G (AirScale)** | **ARMv8-A** / **x86-64** | Nokia **ReefShark Chipset** (Arsitektur SoC hybrid ARM + Massive MIMO beamforming ASIC) | Nokia AirScale Radio Access (BTS 5G & Cloud RAN baseband unit) |
| **ONT / Modem Fiber Rumah** | **MIPS32** / **ARMv7-A** | Broadcom / Realtek PON SoC | Nokia G-240W-F, G-140W-ME (ONT Indihome standar) |

---

## 8. ZTE (Zhongxing Telecommunication Equipment)

ZTE memiliki divisi perancangan chip mandiri bernama **Sanechips (ZXIC)** untuk router backbone, BTS, dan OLT:

| Kategori Perangkat | Arsitektur CPU & ISA | Vendor Silikon / Chipset | Contoh Perangkat |
| :--- | :--- | :--- | :--- |
| **Core Router & Data Center Switch** | **x86-64** (Control) + ZTE Custom NPU | ZTE **Sanechips (ZXIC)** LT series / Broadcom StrataDNX | ZTE ZXR10 T8000, M6000-S Service Router (Telco Core) |
| **OLT GPON / XGS-PON Carrier** | **ARMv8-A** / **PowerPC** (legacy) | ZTE Sanechips ZX2791xx family / Broadcom | ZTE ZXA10 C300, ZXA10 C320, ZXA10 C600 Titan platform |
| **ONT GPON Rumahan (Modem Fiber)** | **ARMv7-A Cortex-A9** / **MIPS32** | ZTE Sanechips **ZX279128S / ZX279127** (Dual ARM Cortex-A9 @ 1GHz) | ZTE F609, F660, F670L (Modem standar sejuta umat Indihome / MNC Play) |
| **5G Wireless Baseband (BBU)** | **ARMv8-A 64-bit** | ZTE Sanechips 7nm 5G Baseband Processor | ZTE ZXSDR B8300, V9200 5G Baseband Unit |
| **5G CPE Router (Wi-Fi Portable)** | **ARMv8-A (AArch64)** | Qualcomm Snapdragon X55/X62 atau Unisoc Tanggula V510 | ZTE MC801A, ZTE Blade 5G CPE |

---

## 9. HUAWEI ("Hawaii") (Enterprise, Carrier, & Consumer Network)

Huawei mendesain arsitektur silikon sendiri melalui anak perusahaannya, **HiSilicon**, yang memproduksi lini chip Solar, Kunpeng, Balong, dan Gigahome:

| Kategori Perangkat | Arsitektur CPU & ISA | Forwarding Chipset / NPU | Contoh Seri Perangkat |
| :--- | :--- | :--- | :--- |
| **Backbone & Core Router** | **Huawei Kunpeng (ARMv8-A 64-bit)** / Intel x86 | Huawei **Solar Network Processor (Solar 5.0 / Solar 6.0 NPU)** | Huawei NetEngine 8000 (NE8000), NE40E, NE5000E Core Router |
| **Data Center Cloud Switches** | **Kunpeng 920** (ARMv8-A 64-bit) | Huawei **CloudEngine ASIC** + Ascend AI processing unit | Huawei CloudEngine 16800, CE8800, CE6800 Series |
| **OLT GPON / XGS-PON Carrier** | **ARMv8-A** (Kunpeng / Cortex-A53) | HiSilicon SD511x / Balong network processor | Huawei SmartAX MA5600T, MA5800 series (XGS-PON & GPON backbone) |
| **ONT / Modem Fiber Rumah** | **ARMv7-A** / **MIPS32** | HiSilicon **SD5116 / SD5117** (Dual ARM Cortex-A9) | Huawei HG8245A, HG8245H, HG8245H5, EchoLife EG8145V5 |
| **Enterprise Wi-Fi 6/7 Access Points** | **ARMv8-A (AArch64)** | HiSilicon **Gigahome Quad-Core** / Qualcomm Networking Pro | Huawei AirEngine 5760, 6760, 8760 series |
| **Consumer Wi-Fi 6 Router** | **ARMv8-A 64-bit** | HiSilicon **Gigahome 650 (Hi5651L / Hi5651T Quad-Core @ 1.4GHz)** | Huawei WiFi AX3 / AX3 Pro, Huawei Router WS5200 |
| **5G BTS Baseband Unit (BBU)** | **ARMv8-A (Kunpeng Architecture)** | Huawei **Tiangang Chipset** (SoC baseband 5G terintegrasi penuh pertama di dunia) | Huawei BBU5900, 5G Blade Site |
