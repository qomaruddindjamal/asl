# DOKUMEN MASTER INVENTARISASI KEKURANGAN & ROADMAP PEMBARUAN RND LENGKAP (2026)
## "The Universal Sovereign OS: Desktop, Mobile, Tablet, Modem, Router & Carrier Fiber"

Dokumen ini memuat inventarisasi lengkap kekurangan sistem serta **Roadmap Terintegrasi 7 Milestone** yang mencakup seluruh perangkat: Desktop, Laptop, HP/Smartphone, Tablet, Smartwatch, Router, Modem FTTH (XPON ONT), dan Carrier OLT.

---

## BAGIAN 1: REKAPITULASI HASIL VERIFIKASI LAPANGAN
1. **Verifikasi Hardware Asli (PASS):** Telemetri memori terbukti dinamis membaca kapasitas asli BIOS/chipset tanpa dummy data (diuji pada 64 MB dan 4 MB RAM).
2. **Identitas CPUID Asli (PASS):** Berhasil mengekstraksi string brand CPU fisik host (`Intel(R) Core(TM) i5-8500 CPU @ 3.00GHz`).
3. **Ketersediaan Suite QEMU Multi-Arch (PASS):** Seluruh emulator CPU (`aarch64`, `arm`, `riscv`, `mips`, `ppc`, `sparc`, `s390x`, `loongarch64`) telah terinstal dan terverifikasi di Windows host (`C:\Program Files\qemu\`).

---

## BAGIAN 2: MATRIKS KEBUTUHAN DRIVER PER KATEGORI PERANGKAT

### 1. Kategori Desktop, Laptop & Server (x86_64 / ARM64 / RISC-V)
* **Bus & Interconnect:**
  * `asl-driver-pci`: Scanner bus PCI/PCIe konfigurasi port `0xCF8`/`0xCFC` & PCIe ECAM MMIO.
  * `asl-driver-usb-xhci` & `ehci`: Kontroler USB 3.0 / USB 2.0 untuk flashdisk, keyboard, dan mouse.
  * `asl-driver-thunderbolt`: Intel Goshen/Titan Ridge & USB4 router tunneling (40/80 Gbps).
* **Display & Grafis:**
  * `asl-driver-hdmi-dp`: Display engine transmitter, audio TMDS clock, dan pembaca I2C EDID/DDC monitor.
  * Driver Akselerasi GPU Modern:
    * `asl-driver-gpu-intel`: Intel Iris Xe / Arc / UHD Graphics.
    * `asl-driver-gpu-amd`: AMD Radeon RDNA / GCN (AMDGPU kernel submission ring).
    * `asl-driver-gpu-nvidia`: NVIDIA GeForce / RTX (GSP firmware pushbuffer interface).
* **Penyimpanan (Storage):**
  * `asl-driver-ata-pio`: IDE hard disk legacy (<10 KB RAM).
  * `asl-driver-ahci`: Kontroler SATA SSD / HDD (FIS & Command List).
  * `asl-driver-nvme`: Kontroler NVMe PCIe SSD berkecepatan tinggi.
  * `asl-storage-raid`: Software RAID 0 (Striping), RAID 1 (Mirroring), RAID 5 (Parity).
  * `asl-parted`: Utilitas manajemen tabel partisi disk GPT dan MBR.

### 2. Kategori Handphone (Smartphone) & Tablet (Snapdragon, Exynos, MediaTek, Apple A/M)
* **Antarmuka Sentuh & Display Mobile:**
  * `asl-driver-touch-i2c`: Driver panel sentuh kapasitif multi-touch (Goodix GT911, Synaptics, FocalTech).
  * `asl-driver-dsi-display`: MIPI-DSI video controller untuk layar OLED / AMOLED handphone.
* **Manajemen Daya & Baterai:**
  * `asl-driver-pmic`: Qualcomm PMIC / MediaTek Power Management IC (pengukur baterai Fuel Gauge, Fast Charging, voltase thermal).
* **Konektivitas Nirkabel & Sensor:**
  * `asl-driver-wifi-mobile`: Driver Wi-Fi 6/7 mobile station mode.
  * `asl-driver-bluetooth-hci`: Bluetooth Low Energy (BLE) stack via UART HCI.
  * `asl-driver-sensor-hub`: Akselerometer, giroskop, kompas magnetik, dan proximity sensor via bus I2C/SPI.
* **Kamera & Audio:**
  * `asl-driver-mipi-csi`: Antarmuka sensor kamera HP.
  * `asl-driver-audio-codec`: I2S DAC audio amplifier & mikrofon ganda.

### 3. Kategori Modem Rumahan, XPON ONT & Router (MikroTik, OpenWrt, TP-Link, Huawei, ZTE)
* **Subsystem Jaringan LAN / WAN:**
  * `asl-driver-e1000`: Kartu Gigabit Ethernet Intel universal.
  * `asl-driver-rtl8139-8169`: Kartu LAN Realtek untuk router rumahan.
  * `asl-driver-virtio-net`: Driver jaringan virtual akselerasi VM.
  * `asl-driver-switch-realtek`: Driver switch terintegrasi L2/L3 (RTL8367 / RTL83xx).
* **Teknologi Fiber XPON (Dual-Mode GPON + EPON):**
  * `asl-ont-xpon`: Mesin auto-sensing deteksi otomatis sinyal OLT hilir:
    * Mode GPON (ITU-T G.984 GEM Framing + T-CONT).
    * Mode EPON (IEEE 802.3ah MPCP Gate/Report + LLID).
  * `asl-driver-sfp-ddm`: Driver pembaca register I2C modul transceiver laser SFP (`0xA0`/`0xA2`):
    * TX Optical Power (Daya tembak laser dalam dBm).
    * RX Optical Power (Sensitivitas redaman dalam dBm).
    * Arus bias laser diode & temperatur modul.
  * `asl-driver-hnat`: Hardware Network Address Translation offload (akselerasi NAT 1 Gbps tanpa beban CPU).

### 4. Kategori Carrier OLT & Central Office (Huawei MA5800, ZTE C320/C600, Nokia Quillion)
* **Optical Line Terminal Subsystem:**
  * `asl-olt-carrier`: Pengendali Dynamic Bandwidth Allocation (DBA) skala mikrodetik untuk ratusan ONT.
  * `asl-driver-olt-mac`: Antarmuka register fisik chip OLT (Broadcom BCM68620 / BCM68650).
  * `asl-driver-sfp-plus-10g`: Driver kartu fiber 10G/25G SFP+ (Intel 82599 / X520 / Mellanox ConnectX).

---

## BAGIAN 3: ROADMAP STRATEGIS 7 MILESTONE EKSEKUSI RND

```
[MILESTONE 1] Input, Root Build & Bus Discovery (Makefile, PCI Scanner, PS/2 Keyboard)
     │
[MILESTONE 2] Cockpit Shell & Coreutils (Terminal CLI Prompt, mkdir, ls, meminfo)
     │
[MILESTONE 3] Storage Subsystem & Disk Management (ATA/AHCI, VFS FAT32/Ext4, Parted, RAID)
     │
[MILESTONE 4] Telecom & Optical Fiber XPON (Dual-Mode XPON Auto-Sense, SFP DDM dBm, OLT)
     │
[MILESTONE 5] Network Stack, SFP+ 10G & Ethernet (VirtIO, e1000, TCP/IP, SFP+ 10G)
     │
[MILESTONE 6] Peripheral Modern (USB 2/3 xHCI, Thunderbolt, HDMI, GPU Intel/AMD/NVIDIA)
     │
[MILESTONE 7] Mobile, Tablet & Multi-Arch Fleet (Touch I2C, BLE, ARM64/MIPS/QEMU Suite)
```

---

### Rincian Target Tiap Milestone:

#### MILESTONE 1: PONDASI BUS, INPUT & ALAT BUILD ROOT
* **Target 1.1:** Tambahkan `Makefile` di root proyek untuk mempermudah developer (`make build`, `make iso`, `make run`).
* **Target 1.2:** Implementasikan `asl-driver-pci` (scanner otomatis perangkat hardware via I/O `0xCF8`/`0xCFC`).
* **Target 1.3:** Implementasikan `asl-driver-ps2-keyboard` (menangani input ketikan keyboard di port `0x60`).

#### MILESTONE 2: INTERACTIVE COCKPIT SHELL & PERINTAH DASAR
* **Target 2.1:** Implementasikan `asl-cockpit-shell` dengan prompt interaktif `ASL-OS> `.
* **Target 2.2:** Sediakan built-in commands awal: `help`, `clear`, `meminfo`, `cpuinfo`, `lspci`.
* **Target 2.3:** Siapkan struktur in-memory VFS awal untuk perintah direktori: `mkdir`, `ls`, `cat`.

#### MILESTONE 3: STORAGE SUBSYSTEM, PARTED & RAID ENGINE
* **Target 3.1:** Implementasikan `asl-driver-ata-pio` (IDE) dan `asl-driver-ahci` (SATA SSD/HDD).
* **Target 3.2:** Bangun utilitas partisi disk `asl-parted` (mengelola tabel partisi GPT/MBR di disk fisik).
* **Target 3.3:** Implementasikan mesin Software RAID `asl-storage-raid` (RAID 0, 1, 5).

#### MILESTONE 4: TELEKOMUNIKASI FIBER XPON SEJATI (GPON + EPON) & OLT
* **Target 4.1:** Perluas crate `asl-ont-gpon` menjadi dual-mode **XPON Auto-Sensing** (GPON GEM + EPON MPCP 802.3ah).
* **Target 4.2:** Implementasikan `asl-driver-sfp-ddm` untuk membaca daya laser optik fisik asli (TX/RX dBm, arus bias, suhu).
* **Target 4.3:** Integrasikan register abstraction kontrol OLT carrier multi-rate.

#### MILESTONE 5: NETWORKING, SFP+ 10G & TCP/IP STACK
* **Target 5.1:** Implementasikan `asl-driver-virtio-net` dan `asl-driver-e1000`.
* **Target 5.2:** Implementasikan driver kartu fiber SFP+ 10G (`Intel 82599 / ixgbe`).
* **Target 5.3:** Implementasikan stack protokol jaringan ringan `asl-net-stack` (ARP, IPv4, ICMP Ping).

#### MILESTONE 6: DRIVER PERIPHERAL MODERN (USB 3, THUNDERBOLT, HDMI, GPU)
* **Target 6.1:** Implementasikan kontroler USB 2.0 (EHCI) & USB 3.0/3.1 (xHCI) untuk flashdisk/keyboard USB.
* **Target 6.2:** Siapkan kerangka Thunderbolt PCIe tunneling controller.
* **Target 6.3:** Implementasikan pembaca HDMI I2C DDC/EDID dan display timing controller.
* **Target 6.4:** Integrasikan abstraction ring GPU untuk Intel Iris Xe, AMD Radeon, dan NVIDIA GeForce.

#### MILESTONE 7: MOBILE, TABLET & VERIFIKASI MULTI-ARSITEKTUR LENGKAP
* **Target 7.1:** Implementasikan driver layar sentuh I2C Touchscreen & Bluetooth Low Energy (BLE).
* **Target 7.2:** Otomatisasi pengujian kompilasi & boot pada `qemu-system-aarch64` (Handphone/Tablet Snapdragon & Cortex-A).
* **Target 7.3:** Otomatisasi boot pada `qemu-system-riscv64` dan `qemu-system-mips` (Smartwatch & Router).
* **Target 7.4:** Otomatisasi boot pada `qemu-system-sparc` dan `qemu-system-s390x` (Avionik Satelit & Mainframe).
