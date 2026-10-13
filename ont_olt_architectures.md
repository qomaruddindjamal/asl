# Arsitektur Hardware & Chipset Sistem GPON / XGS-PON: ONT & OLT

Dokumen ini menguraikan arsitektur internal, Instruction Set Architecture (ISA), vendor silikon, dan blok diagram fungsional dari:
1. **Modem ONT / ONU (Optical Network Terminal / Unit)** — Sisi Pelanggan (CPE)
2. **Perangkat OLT (Optical Line Terminal)** — Sisi Sentral Operator / ISP (CO)

---

## 1. ANATOMI ARSITEKTUR MODEM ONT (CUSTOMER PREMISES EQUIPMENT)

Modem ONT mengonversi sinyal fotonik (cahaya) pada kabel fiber optik (panjang gelombang 1310nm/1490nm untuk GPON) menjadi sinyal elektrik (Ethernet, Wi-Fi, dan POTS Telepon RJ11).

### Diagram Blok Hardware ONT
```
                                        +-------------------------------------------------+
                                        |                   ONT SoC                       |
 [Kabel Fiber Optik]                    |                                                 |
        |                               |   +-------------------+   +-----------------+   |
        v                               |   | CPU Core          |   | PON MAC & TC    |   |
  +------------+   Analog / SerDes      |   | (ARMv7 / ARMv8    |   | Layer Hardware  |   |
  | BOSA Optic | ---------------------> |   |  atau MIPS32)     |   | Engine (AES/FEC)|   |
  | Transceiver|                        |   +---------+---------+   +--------+--------+   |
  +------------+                        |             |                      |            |
                                        |   +---------v----------------------v--------+   |
                                        |   | Packet Forwarding Engine (PPE / HNAT)   |   |
                                        |   | Hardware IPv4/IPv6 Wire-Speed Switch    |   |
                                        |   +----+---------------+---------------+----+   |
                                        +--------|---------------|---------------|--------+
                                                 |               |               |
                                                 v               v               v
                                            [Gigabit LAN]   [Wi-Fi Radio]   [SLIC / RJ11]
                                            (RTL8211/PHY)    (2.4G / 5GHz)    (VoIP FXS)
```

### Komponen Kunci Arsitektur ONT:
1. **BOSA (Bi-Directional Optical Sub-Assembly):** Komponen fotonik yang memisahkan sinyal downstream (1490nm) via fotodioda APD/TIA dan memancarkan sinyal upstream (1310nm) via laser DFB.
2. **PON MAC & Transmission Convergence (TC) Layer Engine:** Blok silikon khusus yang menangani enkripsi AES-128 downstream, alokasi bandwidth dinamis (DBA via T-CONT), dan enkapsulasi GEM (GPON Encapsulation Method).
3. **Hardware Packet Processor (PPE / HNAT):** Mesin switching paket berbasis perangkat keras yang melakukan routing NAT kecepatan penuh (1 Gbps) tanpa membebani CPU utama.
4. **SLIC (Subscriber Line Interface Circuit):** Chip konversi VoIP digital ke sinyal analog tegangan tinggi (48V) untuk port telepon RJ11 (POTS).

---

## 2. DETAIL CHIPSET & ARSITEKTUR MODEM ONT POPULER

| Merek & Model ONT | Chipset / SoC Utama | Arsitektur CPU & ISA | Kecepatan CPU | Blok Akselerator Hardware | Penggunaan di Indonesia |
| :--- | :--- | :--- | :--- | :--- | :--- |
| **ZTE F609 (v1–v3)** | ZTE Sanechips ZX279127 | **ARMv7-A** (Dual-Core Cortex-A9) | 800 MHz | Hardware NAT + PON TC Engine | Standar Indihome / Telkom |
| **ZTE F609 (v4/v5) / F670L** | ZTE Sanechips ZX279128S | **ARMv7-A** (Dual-Core Cortex-A9) | 1.0 GHz | Hardware QoS, Wi-Fi Offload Engine | Indihome, MNC Play, MyRepublic |
| **Huawei HG8245A / HG8245H**| HiSilicon SD5116 | **ARMv7-A** (Dual-Core Cortex-A9) | 800 MHz | HiSilicon FastPath IPv4/IPv6 Engine | Indihome, First Media, XL Home |
| **Huawei HG8245H5 / EG8145V5**| HiSilicon SD5117P | **ARMv7-A** (Dual-Core Cortex-A9) | 1.0 GHz | Dedicated Crypto & Gigabit Switch Engine | Standar Indihome Wi-Fi Dual Band |
| **Huawei OptiXstar (Wi-Fi 6)**| HiSilicon SD5182 / Gigahome | **ARMv8-A** (Dual/Quad Cortex-A53 64-bit)| 1.2–1.4 GHz | Wi-Fi 6 NPU + XGS-PON 10G MAC | Telkomsel One / Indihome Wi-Fi 6 |
| **Nokia G-240W-F / G-140W-ME**| Broadcom BCM68380 / BCM6846 | **MIPS32** (Dual MIPS 74Kc) atau **ARM Cortex-A7** | 600 MHz – 1.0 GHz | Broadcom Packet Processing Engine (Runner) | Indihome (Regional Nokia) |
| **Fiberhome AN5506-04-FG** | MediaTek (Econet) EN7512 / EN7528 | **MIPS32** (Dual MIPS 1004Kc 4-threads) | 900 MHz | MediaTek Hardware NAT + Gigabit Switch | Indihome, Iconnet (PLN) |
| **ZTE F6600P (Wi-Fi 6 AX3000)**| ZTE Sanechips ZX279132 | **ARMv8-A** (Quad-Core Cortex-A53 64-bit)| 1.2 GHz | Multi-gigabit PPE + Wi-Fi 6 OFDMA engine | MyRepublic, Indihome High-Speed |

---

## 3. ANATOMI ARSITEKTUR PERANGKAT OLT (OPTICAL LINE TERMINAL)

OLT berada di kantor sentral (Central Office / POP ISP) dan berfungsi sebagai agregator optik yang melayani ribuan ONT pelanggan melalui arsitektur modular bertingkat (*chassis-based* atau *pizza-box 1U*).

### Arsitektur Terdistribusi OLT:
```
 +-----------------------------------------------------------------------------------+
 |                               OLT CHASSIS SYSTEM                                  |
 |                                                                                   |
 |  [CONTROL / SWITCHING CARD] (Contoh: Huawei SCUN / ZTE SCTM / Nokia NT Card)      |
 |  +-----------------------------------------------------------------------------+  |
 |  | Control Plane CPU (Intel x86-64 / ARMv8 / PowerPC)                          |  |
 |  | -> Menjalankan OS CLI, SNMP, BGP/OSPF, Registrasi ONT, Manajemen Bandwidth |  |
 |  +-----------------------------------------------------------------------------+  |
 |  | Central Switching Fabric ASIC (Broadcom StrataXGS / Marvell / Custom)        |  |
 |  | -> Kapasitas 480 Gbps hingga 3.2 Tbps Non-Blocking Forwarding Matrix         |  |
 |  +--------------------------------------+--------------------------------------+  |
 |                                         | High-Speed Backplane Bus (10G/40G/100G) |
 |                                         v                                         |
 |  [GPON / XGS-PON SERVICE LINE CARDS] (Contoh: GPFD / GTGO / FGLT)                 |
 |  +-----------------------------------------------------------------------------+  |
 |  | Dedicated Linecard Packet Processor (Microsemi / Broadcom / Sanechips)      |  |
 |  | -> Mengatur Dynamic Bandwidth Allocation (DBA) siklus mikrodetik            |  |
 |  +-----------------------------------------------------------------------------+  |
 |  | PON MAC Controllers (Misal: 1 chip menangani 8–16 port GPON)                |  |
 |  +------+--------+--------+--------+--------+--------+--------+--------+-------+  |
 |         |        |        |        |        |        |        |        |          |
 |         v        v        v        v        v        v        v        v          |
 |       [SFP B+ / C+ / C++ Transceiver Optik] (1 port melayani hingga 1:64 / 1:128) |
 +-----------------------------------------------------------------------------------+
```

---

## 4. DETAIL ARSITEKTUR OLT CARRIER POPULER (HUAWEI, ZTE, NOKIA, FIBERHOME, VSOL)

### A. Huawei SmartAX MA5600T & MA5800 Series
* **Klasifikasi:** OLT Modular Chassis Carrier-Grade (Standar utama Telkom Indihome).
* **Control Board (SCUN / MPU):**
  * *CPU:* **ARMv8-A 64-bit** (Kunpeng / Dual Cortex-A53) pada seri modern MA5800; **PowerPC / MIPS** pada MA5600T lawas.
  * *Switching Fabric:* Huawei Custom ASIC Switching Matrix (MA5800 bertipe Distributed Architecture hingga 200 Gbps per slot).
* **Service Board (GPFD / GPHF 16-Port GPON):**
  * *PON MAC Processor:* **HiSilicon SD5113 / HiSilicon Balong NP** (menangani 16 port GPON, 2048 ONT per board pada rasio 1:128).
  * *Fungsi Khusus:* Per-port hardware encryption, line-rate HQoS (Hierarchical Quality of Service).

### B. ZTE ZXA10 C300, C320, & C600 Titan
* **Klasifikasi:** OLT Chassis Modular Carrier-Grade (Banyak dipakai di Indihome & ISP Swasta).
* **Control & Uplink Board (SCXM / SCTM / SMXA):**
  * *CPU:* **ARMv8-A 64-bit** (pada seri C600 Titan); **Freescale PowerPC / MIPS64** (pada seri C300/C320).
  * *Switching ASIC:* ZTE Sanechips ZXIC LT-Series Switching Fabric (C600 Titan mendukung arsitektur Router-level hingga 14 Tbps switching matrix).
* **Service Board (GTGO / GTGH 8/16-Port GPON):**
  * *Chipset:* ZTE **Sanechips ZX2791xx PON Engine** (ASIC pemrosesan MAC optik terpadu).

### C. Nokia 7360 ISAM FX & Lightspan Series
* **Klasifikasi:** OLT Modular High-End Carrier (Penyedia FTTH di Eropa & regional Telkom).
* **Control Unit (NT Card):**
  * *CPU:* **x86-64 multi-core Intel Atom / Xeon** dipadukan dengan kontroler embedded ARM.
* **PON Architecture (Quillion Silicon):**
  * Menggunakan **Nokia Quillion Chipset** — prosesor ASIC optik proprietary Nokia dengan penghematan daya hingga 50% yang mampu melayani GPON, XGS-PON, dan 25G-PON secara serentak di kartu yang sama (Multi-PON).

### D. Fiberhome AN5116-06B & AN5516-01
* **Klasifikasi:** OLT Carrier Telco Tiongkok (Digunakan di Iconnet PLN & ISP daerah).
* **Arsitektur:** Menggunakan kombinasi CPU kontrol **PowerPC / MIPS64** dengan kartu layanan berbasis PON Processor dari **Broadcom (StrataXGS PON)** dan FPGA Xilinx/Altera untuk pemrosesan framing GEM.

### E. V-SOL & HSGQ (Pizza-Box 1U OLT SOHO/ISP RT-RW Net)
* **Klasifikasi:** OLT Standalone 4-Port / 8-Port / 16-Port 1U.
* **Arsitektur CPU:** **MIPS32** (MediaTek / Realtek RTL838x) atau **ARM Cortex-A9**.
* **PON Controller ASIC:** Menggunakan chip turnkey terintegrasi dari **Cortina Systems (Cortina CS8032 / CS8022)** atau **Realtek RTL9601 / Broadcom BCM6800 series**.

---

## 5. TABEL PERBANDINGAN ARSITEKTUR: ONT VS OLT

| Parameter Arsitektur | Modem ONT (Sisi Pelanggan) | OLT (Sisi Sentral Operator) |
| :--- | :--- | :--- |
| **Skala Pemrosesan** | 1 Stream Pengguna (1 – 4 Port LAN) | Ratusan Stream Agregat (Ribuan ONT Pelanggan) |
| **Arsitektur CPU** | SoC Terpadu: **ARMv7-A (Cortex-A9)** / **ARMv8-A (A53)** / **MIPS32** | Terdistribusi: **x86-64** / **ARMv8 Enterprise** + Kartu Layanan Terpisah |
| **Data Plane / Forwarding**| Hardware NAT / PPE Engine hemat daya (on-SoC) | Central Non-Blocking Switching Fabric ASIC (Tbps capacity) |
| **Laser Transmitter** | Burst-Mode Transmitter (DFB Laser 1310nm) | Continuous-Mode Transmitter (DFB/EML Laser 1490nm) |
| **Laser Receiver** | Continuous-Mode Receiver (1490nm) | Burst-Mode Receiver (1310nm, respons mikrodetik) |
| **Manajemen Bandwidth** | Menjalankan T-CONT buffer & antrean lokal | Menghitung algoritma DBA (Dynamic Bandwidth Allocation) secara terpusat |
| **Sistem Operasi** | Embedded Linux (BusyBox + proprietary daemons) | Carrier Modular OS (Huawei VRP, ZTE ROS, Nokia TiMOS) |
