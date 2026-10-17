# DOKUMEN MASTER INVENTARISASI KEKURANGAN, HASIL R&D & ROADMAP ARSITEKTUR KERNEL ASL
## "The Universal Sovereign OS: Desktop, Mobile, Tablet, Modem, Router, Carrier Fiber, Micro-Hypervisor & Avionics Remote Desktop"
**Versi Dokumen:** `2.0.0-RND`  
**Status Audit:** Active Roadmap & Architecture Synthesis  
**Otoritas Desain:** Standar DO-178C DAL-A / ARINC 653 IMA / 150 Silicon Architectures  

---

## BAGIAN 1: REKAPITULASI HASIL VERIFIKASI FAKTUAL (STATUS v1.0.1)
1. **Verifikasi Hardware Asli (PASS):** Telemetri memori terbukti dinamis membaca alokasi fisik tanpa dummy data. Teruji pada live VirtualBox VM `"ASL"` dengan alokasi 64 MB dan 4 MB RAM tanpa recompile.
2. **Identitas CPUID Asli (PASS):** Berhasil mengekstraksi string brand CPU fisik host melalui instruksi assembly x86 `CPUID` leaves `0x80000002`..`0x80000004` (`Intel(R) Core(TM) i5-8500 CPU @ 3.00GHz`).
3. **Ketersediaan Suite QEMU Multi-Arch (PASS):** Seluruh emulator CPU target (`aarch64`, `arm`, `riscv`, `mips`, `ppc`, `sparc`, `s390x`, `loongarch64`) telah siap di host Windows (`C:\Program Files\qemu\`).
4. **Verifikasi Rilis v1.0.1 GitHub (PASS & LOCKED):** Aset biner ISO (`asl-core-kernel-v1.0.1.iso` - 3,954,688 B), ELF kernel (`asl-kernel-x86_64.elf` - 78,384 B), serial log, screenshot GUI, dan manifest kriptografis telah 100% tersinkronisasi di GitHub Release.

---

## BAGIAN 2: HASIL RISET DAN PENGEMBANGAN (R&D) STANDAR INDUSTRI & METODE KODE

Berdasarkan sintesis riset mendalam dari repositori open-source kelas dunia (Linux Kernel upstream, Redox OS, seL4, NetBSD, FreeBSD, SerenityOS, dan OSDev bare-metal standard), berikut adalah kaidah arsitektur dan penyesuaian kode yang **wajib diterapkan** ke dalam ASL Core Kernel:

### 1. Arsitektur Bus & Driver (Hardware Layer):
* **PCI / PCIe Enumeration (Port 0xCF8 / 0xCFC vs ECAM MMIO):**
  * *Metode:* Scanning 256 bus, 32 device, dan 8 function. Membaca Register 0x00 (Vendor/Device ID). Jika Vendor ID `0xFFFF`, device tidak ada.
  * *Penyelarasan ASL:* Crate `asl-driver-pci` harus `#![no_std]`, aman tanpa alokasi dinamis (array fixed-size descriptor), dan memetakan Base Address Registers (BAR0..BAR5) ke dalam ARINC 653 Spatial Vault.
* **PS/2 Keyboard & Controller (Ports 0x60 & 0x64):**
  * *Metode:* Polling loop non-blocking atau IRQ 1 via PIC 8259A. Scan-code Set 1 (Make/Break code) dipetakan ke ASCII buffer ring.
  * *Penyelarasan ASL:* Menggunakan zero-copy bounded ring buffer (DO-178C compliant), mencegah stack overflow pada spam ketikan.
* **USB 2.0 (EHCI) & USB 3.0/3.1 (xHCI):**
  * *Metode:* MMIO Register Operasional, Command Ring, Transfer Ring, dan Event Ring berbaris 64-byte aligned (sesuai DMA safety ASL).
  * *Penyelarasan ASL:* Mengintegrasikan DMA Bounce Buffer `asl-hal::cache::IsolatedBounceBuffer` untuk setiap transfer request block (TRB).
* **Storage (ATA PIO, AHCI SATA, NVMe):**
  * *Metode:* ATA PIO (Port `0x1F0`..`0x1F7`) untuk fallback ekstrem <4 MB RAM. AHCI via Memory Command List & FIS. NVMe via Submission/Completion Queue baris 4096-byte page boundary.
  * *Penyelarasan ASL:* Seluruh IO buffer wajib divalidasi checksum dan dilindungi oleh `IommuTranslationSpace`.

### 2. Partisi, Disk, & VFS (Storage Layer):
* **Format Tabel Partisi (MBR vs GPT):**
  * *Metode:* MBR membaca LBA 0 (tabel 4 partisi 16-byte). GPT membaca LBA 1 (Header 'EFI PART') dan LBA 2..33 (Partition Entries 128-byte).
  * *Penyelarasan ASL:* Crate `asl-parted` membaca disk stream mentah dan menghasilkan slice metadata partisi terisolasi.
* **Software RAID (0, 1, 5):**
  * *Metode:* RAID 0 (Striping blok), RAID 1 (Mirroring write ganda), RAID 5 (XOR Parity deterministik).
  * *Penyelarasan ASL:* Menghindari float atau floating allocations; parity dihitung menggunakan instruksi vektor bitwise murni.

### 3. Jaringan, XPON FTTH, & Modul Telekomunikasi:
* **XPON ONT Auto-Sensing (Dual-Mode GPON + EPON):**
  * *Metode Riset:* Sinyal optik hilir 1490nm dipindai polanya:
    * Frame GPON: Dideteksi dari sinkronisasi header GTC framing delimiter (`0xB6AB31E0`).
    * Frame EPON: Dideteksi dari MPCP Discovery Gate message opcode `0x000A` (IEEE 802.3ah).
  * *Penyelarasan ASL:* Modul `asl-ont-xpon` otomatis beralih (*auto-negotiate*) dalam waktu <50 milidetik setelah mendeteksi laser OLT.
* **SFP DDM (Digital Diagnostic Monitoring):**
  * *Metode:* Membaca chip I2C transceiver optik pada alamat I2C `0xA0` (ID data) dan `0xA2` (Diagnostik DDM). Register 102-103 = RX Optical Power, Register 104-105 = TX Optical Power. Rumus konversi: $P_{dBm} = 10 \cdot \log_{10}(P_{mW})$.
  * *Penyelarasan ASL:* Menggunakan fungsi integer fixed-point math (milli-dBm) tanpa FPU runtime, menjaga efisiensi di mode kernel.

### 4. Cockpit Shell & Lingkungan Pengembangan User:
* **Interactive Cockpit Shell (`asl-cockpit-shell`):**
  * *Metode:* Prompt interaktif baris perintah (`ASL-OS> `) yang merender output langsung ke `FbConsole` (Linear Framebuffer) dan serial COM1.
  * *Perintah Wajib v1.0.2:* `help`, `clear`, `meminfo`, `cpuinfo`, `lspci`, `uname`, `reboot`, `halt`.
* **Sistem Build & Workflow Developer:**
  * Setelah mendownload repo/ISO: Developer cukup menjalankan `make build`, `make iso`, `make test`, atau `make run-vm` tanpa perlu menghafal baris perintah panjang.

---

## BAGIAN 3: MATRIKS KEBUTUHAN SUBSYSTEM & DRIVER LENGKAP

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
* **Penyimpanan (Storage) & Partisi:**
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

### 5. Subsystem Unggulan: Remote Desktop Khas ASL & Micro-Hypervisor Bawaan Kernel
* **ASL Avionics Remote Desktop (ARD / `asl-remote-desktop`):**
  * *Filosofi:* Remote desktop bawaan kernel yang sangat responsif, hemat bandwidth, dan ultra-ringan (RAM footprint <64 KB).
  * *Mekanisme:* Menggunakan teknik **Dirty-Tile Framebuffer Streaming**:
    * Hanya mengirimkan blok tile piksel (16x16 atau 32x32) yang berubah (*differential rendering*), bukan kompresi video berat yang memakan CPU.
    * Enkripsi data bawaan berlatensi sub-milidetik berbasis chacha20-poly1305.
    * Protokol adaptif: dapat diakses langsung dari browser web via WebSocket / WebRTC atau client desktop native ASL.
* **ASL Micro-Hypervisor (`asl-micro-kvm`):**
  * *Filosofi:* Mesin virtualisasi hardware-assisted ultra-ringan (Type-1 Embedded Hypervisor) berbasis instruksi silikon:
    * Intel VT-x (`VMX`) dan AMD-V (`SVM`).
    * ARM Virtualization Extensions (EL2 Hypervisor mode).
  * *Karakteristik:*
    * Mampu menjalankan VM guest terisolasi dengan overhead RAM **kurang dari 1 MB**!
    * Memanfaatkan partisi deterministik ARINC 653 untuk menjamin alokasi CPU core VM guest tidak pernah mencuri waktu kernel utama (*Zero-Jitter VM Partitioning*).

---

## BAGIAN 4: ROADMAP INTEGRATIF 8 MILESTONE PENGEMBANGAN v1.0.2 - v2.0.0

```
┌────────────────────────────────────────────────────────────────────────────────────────┐
│ [MILESTONE 1] Input, Root Build & Bus Discovery (Makefile, PCI Scanner, PS/2 Keyboard) │
└──────────────────────────────────────────┬─────────────────────────────────────────────┘
                                           │
┌──────────────────────────────────────────▼─────────────────────────────────────────────┐
│ [MILESTONE 2] Cockpit Shell & Coreutils (Terminal CLI Prompt, mkdir, ls, meminfo, lspci)│
└──────────────────────────────────────────┬─────────────────────────────────────────────┘
                                           │
┌──────────────────────────────────────────▼─────────────────────────────────────────────┐
│ [MILESTONE 3] Storage Subsystem & Disk Management (ATA/AHCI, VFS FAT32/Ext4, Parted,RAID)│
└──────────────────────────────────────────┬─────────────────────────────────────────────┘
                                           │
┌──────────────────────────────────────────▼─────────────────────────────────────────────┐
│ [MILESTONE 4] Telecom & Optical Fiber XPON (Dual-Mode XPON Auto-Sense, SFP DDM dBm, OLT)│
└──────────────────────────────────────────┬─────────────────────────────────────────────┘
                                           │
┌──────────────────────────────────────────▼─────────────────────────────────────────────┐
│ [MILESTONE 5] Network Stack, SFP+ 10G & Ethernet (VirtIO, e1000, TCP/IP, SFP+ 10G)     │
└──────────────────────────────────────────┬─────────────────────────────────────────────┘
                                           │
┌──────────────────────────────────────────▼─────────────────────────────────────────────┐
│ [MILESTONE 6] Peripheral Modern (USB 2/3 xHCI, Thunderbolt, HDMI, GPU Intel/AMD/NVIDIA) │
└──────────────────────────────────────────┬─────────────────────────────────────────────┘
                                           │
┌──────────────────────────────────────────▼─────────────────────────────────────────────┐
│ [MILESTONE 7] Micro-Hypervisor (KVM Ultra-Ringan) & Avionics Remote Desktop (ARD)      │
└──────────────────────────────────────────┬─────────────────────────────────────────────┘
                                           │
┌──────────────────────────────────────────▼─────────────────────────────────────────────┐
│ [MILESTONE 8] Mobile, Tablet & Multi-Arch Fleet (Touch I2C, BLE, ARM64/MIPS/QEMU Suite)│
└────────────────────────────────────────────────────────────────────────────────────────┘
```

---

## BAGIAN 5: RINCIAN TARGET TEKNIS VERSI v1.0.2 (MILESTONE 1 & 2)

### Target 1.1: Root Makefile Universal (`/Makefile`)
- Menyediakan target:
  - `make build`: Kompilasi workspace Rust target `x86_64-unknown-none` dalam mode release.
  - `make test`: Menjalankan suite unit test host `x86_64-pc-windows-msvc`.
  - `make iso`: Mengemas kernel biner menjadi ISO bootable hybrid via `xorriso` dan Limine.
  - `make run-vm`: Menjalankan VirtualBox VM `"ASL"` secara instan.
  - `make clean`: Membersihkan file temporary build.

### Target 1.2: PCI Bus Enumerator (`asl-driver-pci`)
- Crate baru `asl-driver-pci` berstandar `#![no_std]`.
- Memindai bus I/O ports `0xCF8` (Config Address) dan `0xCFC` (Config Data).
- Struktur data terverifikasi:
  - Vendor ID, Device ID, Subsystem ID.
  - Class Code & Subclass (Network, Display, Storage, Bridge).
  - BAR0..BAR5 (Base Address Registers: Port I/O atau MMIO).
- Menyediakan API publik: `pci_enumerate() -> &[PciDeviceDescriptor]`.

### Target 1.3: PS/2 Keyboard Driver (`asl-driver-ps2-keyboard`)
- Crate baru `asl-driver-ps2-keyboard` berstandar `#![no_std]`.
- Membaca I/O Port `0x64` (Status Register) dan `0x60` (Data Register).
- Scan-code Set 1 decoder (Key Press & Release).
- Menerjemahkan scan-code ke karakter ASCII standar (termasuk tombol Shift, CapsLock, Enter, Backspace).

### Target 1.4: Interactive Cockpit Shell (`asl-cockpit-shell`)
- Prompt interaktif di layar: `ASL-OS> ` dengan kursor berkedip.
- Menangkap input keyboard pengguna melalui `asl-driver-ps2-keyboard`.
- Perintah bawaan tahap pertama:
  - `help`: Menampilkan daftar perintah yang tersedia.
  - `clear`: Membersihkan layar Linear Framebuffer.
  - `meminfo`: Menampilkan detail alokasi RAM dinamis dan jumlah free page.
  - `cpuinfo`: Menampilkan model brand string CPU fisik via CPUID.
  - `lspci`: Menampilkan daftar seluruh perangkat hardware yang terdeteksi di bus PCI.
  - `reboot` / `halt`: Menjalankan restart atau safe ACPI/APM shutdown.

---

## BAGIAN 6: FORMAL AUDIT CHECKLIST (CRITERIA FOR PASS & LOCK)
- [ ] Seluruh crate baru terdaftar di workspace root `Cargo.toml`.
- [ ] 100% lolos kompilasi `#![no_std]` untuk target bare-metal `x86_64-unknown-none`.
- [ ] Seluruh unit test lulus pada `cargo test --workspace --target x86_64-pc-windows-msvc`.
- [ ] Diuji secara live pada VM `"ASL"` dengan layar GUI interaktif.
- [ ] Audit kebersihan workspace `tools/Sub_Assist.py` menghasilkan `100% PASS`.
