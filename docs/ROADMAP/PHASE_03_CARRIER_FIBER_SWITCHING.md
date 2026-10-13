# FASE 3: AVIONICS BUS & FIBER SWITCHING (BULAN 13–18)
## Zero-Copy Packet Switching, Silikon Switch Datacenter & Infrastruktur FTTH PON

Dokumen ini mengatur sasaran rekayasa, deliverable arsitektural, dan protokol pengujian untuk **Fase 3**. Fase ini mengintegrasikan ASL Core Kernel dengan silikon switching jaringan berkecepatan tinggi (*Carrier Networks*) dan modem fiber optik FTTH (ONT & OLT).

---

## 1. SASARAN TEKNIS & ARSITEKTUR SASARAN
* **Arsitektur Utama:**
  1. `Broadcom StrataXGS (Tomahawk & Trident)` — Switch datacenter spine/leaf (hingga 51.2 Tbps).
  2. `Broadcom StrataDNX (Jericho & Qumran)` — Router carrier BGP deep-buffer.
  3. `Marvell Prestera (Alleycat / Bobcat)` — Switch packet processor (MikroTik CRS series, UniFi Switch).
  4. `Intel Tofino (Barefoot P4 Architecture)` — Fully user-programmable P4 switch ASIC.
  5. `GPON & XGS-PON Silikon (ONT & OLT)` — HiSilicon SD511x/Balong, ZTE Sanechips ZX2791xx, Nokia Quillion ASIC, Cortina Systems.

---

## 2. DELIVERABLES TEKNIS (BULAN 13 – BULAN 18)

### Bulan 13–14: ASL FastPath Network Engine & Abstraksi TCAM
* **Polled-Mode Driver (PMD) Framework:** Arsitektur transfer paket zero-copy bypass kernel mirip DPDK langsung dari ring buffer PCIe ke userspace.
* **TCAM Rule Ingestion:** Abstraksi pengisian tabel Access Control List (ACL), L2 Forwarding Database (FDB), dan L3 Routing Table langsung ke register hardware ASIC.
* **Link Aggregation & VLAN Offload:** Offload protokol 802.1Q VLAN dan LACP hardware wire-speed.

### Bulan 15–16: Driver Silikon Fiber Optik FTTH (Modem ONT)
* **BOSA & SerDes Transceiver Handshake:** Pengaturan sinyal analog/optik transceiver burst-mode pada panjang gelombang 1310nm/1490nm.
* **GEM Framing & T-CONT Engine:** Penanganan enkapsulasi GPON Encapsulation Method (GEM) dan buffer Transmission Containers (T-CONT) pada chipset ZTE Sanechips ZX279128S dan HiSilicon SD5116.
* **Hardware NAT (HNAT) Acceleration:** Routing gigabit wire-speed tanpa pembebanan CPU pada modem pelanggan.

### Bulan 17–18: Driver OLT Carrier & Algoritma DBA
* **Dynamic Bandwidth Allocation (DBA) Controller:** Perhitungan alokasi slot transmisi TDMA dalam interval mikrodetik pada kartu GPON Huawei MA5800 dan ZTE C320/C600.
* **Integrasi Nokia Quillion:** Driver manajemen konsumsi daya rendah dan agregasi multi-PON (GPON + XGS-PON 10G di satu port).
* **Kriteria Kelulusan Fase 3:** Throughput pengalihan paket mencapai line-rate tanpa packet loss, integrasi registrasi OMCI antara ONT dan OLT berjalan stabil.
