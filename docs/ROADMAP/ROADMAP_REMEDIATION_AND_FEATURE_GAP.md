# DOKUMEN MASTER INVENTARISASI KEKURANGAN & ROADMAP PEMBARUAN RND (2026)
## "The Avionics OS Evolution: From Microkernel Core to Sovereign Multi-Arch OS"

---

## BAGIAN 1: REKAPITULASI HASIL VERIFIKASI RND TERAKHIR
Berdasarkan hasil pengujian langsung di mesin bare-metal/VirtualBox:
1. **Verifikasi Memori Dinamis (PASS):** Berhasil membaca memori hardware asli secara presisi tanpa dummy data (diuji pada 64 MB dan 4 MB RAM via Limine Handover & CMOS reader).
2. **Identitas CPU Asli (PASS):** Berhasil mengekstraksi string brand CPU fisik host (`Intel(R) Core(TM) i5-8500 CPU @ 3.00GHz`) via instruksi assembly `CPUID`.
3. **Ketersediaan Emulator Multi-Arch (PASS):** Suite QEMU lengkap untuk seluruh arsitektur dunia telah terverifikasi tersedia di PC Windows Anda (`C:\Program Files\qemu\`).

---

## BAGIAN 2: DAFTAR LENGKAP INVENTARISASI KEKURANGAN KERNEL

Berikut adalah daftar seluruh komponen, driver, paket, utilitas, dan toolchain yang **belum ada** di dalam kernel saat ini:

### A. Driver Perangkat Keras (Hardware Drivers)
1. **Bus & Device Discovery:**
   * `asl-driver-pci`: Scanner bus PCI/PCIe untuk enumerasi otomatis perangkat terpasang (NIC, Storage, GPU, Sound).
2. **Interrupt Controller:**
   * `asl-driver-pic-ioapic` (x86), `asl-driver-gic` (ARM Cortex-A), `asl-driver-plic` (RISC-V).
3. **Driver Input Pengguna:**
   * `asl-driver-ps2-keyboard`: Driver keyboard port `0x60`/`0x64` untuk menerima ketikan langsung di layar.
   * `asl-driver-usb-hid`: Driver keyboard & mouse USB.
4. **Driver Kartu Jaringan (NIC Ethernet & Paravirtual):**
   * `asl-driver-virtio-net`: Driver jaringan virtual tercepat untuk VM (VirtualBox/QEMU).
   * `asl-driver-e1000`: Driver kartu jaringan Intel Gigabit Ethernet universal.
   * `asl-driver-rtl8139-8169`: Driver kartu LAN Realtek untuk router & desktop.
5. **Driver Fiber Optik & Transceiver (FTTH / SFP):**
   * `asl-driver-sfp-ddm`: Driver pembaca register I2C modul SFP (`0xA0`/`0xA2`) untuk kekuatan laser dBm optik & temperatur.
   * `asl-ont-xpon`: Modul dual-mode auto-sensing (kombinasi GPON GEM + EPON MPCP 802.3ah).
   * Driver transceiver kartu OLT chipset (Broadcom BCM68620/BCM68650).
6. **Driver Media Penyimpanan (Storage):**
   * `asl-driver-ata-pio`: Driver hard disk IDE klasik hemat memori (<10 KB RAM).
   * `asl-driver-ahci`: Driver kontroler SATA SSD/HDD.
   * `asl-driver-nvme`: Driver SSD PCIe berkecepatan tinggi.
7. **Driver Mobile, Wearable & Wireless:**
   * Driver touchscreen bus I2C (Goodix/Synaptics) untuk HP/tablet.
   * Stack Bluetooth Low Energy (HCI BLE) untuk smartwatch/kacamata pintar.
   * Subsystem Wi-Fi (802.11ax/be).

### B. Perangkat Lunak Manajemen Penyimpanan (RAID & Partisi)
1. **Software RAID Engine (`asl-storage-raid`):**
   * Belum ada dukungan RAID 0 (Striping), RAID 1 (Mirroring), dan RAID 5 (Parity).
2. **Utilitas Partisi Disk (`asl-parted` / `asl-fdisk`):**
   * Belum ada software CLI untuk membaca/mengubah tabel partisi GPT (GUID Partition Table) dan MBR langsung dari dalam OS.

### C. Sistem Berkas (VFS) & Perintah Dasar (Coreutils)
1. **Virtual File System (`asl-vfs`):**
   * Belum ada lapisan VFS untuk menangani mount point (`/`, `/bin`, `/etc`, `/dev`).
   * Belum ada implementasi filesystem FAT32 dan Ext4/VaultFS.
2. **Utilitas Perintah Shell:**
   * Belum ada perintah manajemen direktori dan file: `mkdir`, `rm`, `ls`, `cd`, `cat`, `touch`, `cp`.

### D. Lingkungan Pengembang & Build System
1. **Root Build Wrapper (`Makefile`):**
   * Belum ada file `Makefile` di root repositori untuk mempermudah developer luar mengeksekusi `make build`, `make iso`, dan `make run`.
2. **Interactive Terminal Shell (`asl-cockpit-shell`):**
   * Belum ada antarmuka baris perintah interaktif (prompt `ASL-OS> `) di layar framebuffer/serial.

---

## BAGIAN 3: ROADMAP STRATEGIS PEMBARUAN RND

Roadmap ini disusun terstruktur menjadi 6 Milestone Eksekusi:

```
[MILESTONE 1] Input & Bus Discovery (PCI + PS/2 Keyboard + Makefile)
     │
[MILESTONE 2] Interactive Cockpit Shell & Coreutils (Shell, mkdir, ls)
     │
[MILESTONE 3] Storage Subsystem & Disk Management (ATA/AHCI, VFS, Parted, RAID)
     │
[MILESTONE 4] Telecom & Optical Fiber XPON (XPON Auto-Sense, SFP DDM dBm)
     │
[MILESTONE 5] Network Stack & Ethernet Drivers (VirtIO-Net, e1000, TCP/IP)
     │
[MILESTONE 6] Multi-Arch Fleet QEMU Emulation (ARM, RISC-V, MIPS, SPARC, LoongArch)
```

---

### Rincian Rencana Aksi Roadmap:

#### MILESTONE 1: PONDASI BUS, INPUT INTERAKTIF & ROOT BUILD
* **Target 1.1:** Tambahkan `Makefile` di root folder untuk simplifikasi build/run (`make build`, `make iso`, `make run`).
* **Target 1.2:** Implementasikan crate `asl-driver-pci` untuk memindai perangkat hardware secara otomatis.
* **Target 1.3:** Implementasikan crate `asl-driver-ps2-keyboard` untuk membaca penekanan tombol keyboard pengguna.

#### MILESTONE 2: COCKPIT SHELL & PERINTAH DASAR (COREUTILS)
* **Target 2.1:** Implementasikan crate `asl-cockpit-shell` dengan prompt interaktif `ASL-OS> `.
* **Target 2.2:** Sediakan built-in commands awal: `help`, `clear`, `meminfo`, `cpuinfo`, `lspci`.
* **Target 2.3:** Siapkan struktur in-memory VFS awal untuk perintah `mkdir`, `ls`, dan `cat`.

#### MILESTONE 3: STORAGE SUBSYSTEM, PARTED & RAID ENGINE
* **Target 3.1:** Implementasikan `asl-driver-ata-pio` dan `asl-driver-ahci` (SATA).
* **Target 3.2:** Bangun utilitas CLI `asl-parted` untuk mengelola tabel partisi GPT di disk fisik.
* **Target 3.3:** Implementasikan modul `asl-storage-raid` (RAID 0, 1, 5).

#### MILESTONE 4: TELEKOMUNIKASI FIBER XPON SEJATI (GPON + EPON)
* **Target 4.1:** Perluas crate `asl-ont-gpon` menjadi dual-mode **XPON Auto-Sensing** (GPON GEM + EPON MPCP 802.3ah).
* **Target 4.2:** Implementasikan driver sensor laser I2C DDM untuk mengukur kekuatan pancar/terima optik asli (TX/RX dBm).

#### MILESTONE 5: NETWORKING & ETHERNET CONTROLLERS
* **Target 5.1:** Implementasikan `asl-driver-virtio-net` dan `asl-driver-e1000`.
* **Target 5.2:** Implementasikan stack protokol jaringan ringan `asl-net-stack` (ARP, IPv4, ICMP Ping).

#### MILESTONE 6: VERIFIKASI MULTI-ARSITEKTUR VIA QEMU SUITE WINDOWS
* **Target 6.1:** Otomatisasi pengujian kompilasi & boot pada `qemu-system-aarch64` (Handphone/Tablet).
* **Target 6.2:** Otomatisasi boot pada `qemu-system-riscv64` dan `qemu-system-mips` (Smartwatch/Router).
* **Target 6.3:** Otomatisasi boot pada `qemu-system-sparc` (Avionik Antariksa).
