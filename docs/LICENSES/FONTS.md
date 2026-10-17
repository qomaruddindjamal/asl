# FONTS & TYPOGRAPHY ATTRIBUTION: ASL OPERATING SYSTEM
======================================================

ASL Core Kernel dan ekosistem instalasi ASL mengadopsi standar tipografi terbuka, bebas royalti, dan mematuhi etika lisensi open source untuk seluruh platform komputasi avionik.

---

## 1. ASL Builtin Avionics Font (ASL Core Builtin)
* **Status:** Built-in Primary Microkernel Console Font
* **Ukuran Glyph:** 8x16 pixels / 16x32 scalable
* **Karakteristik:** Desain tebal (bold), proporsional, sudut tajam, dan kontras tinggi untuk keterbacaan instan di layar kokpit avionik dan konsol sistem bare-metal.
* **Lisensi:** Dedicated Public Domain / GPL-3.0-or-later for ASL Operating System.

---

## 2. Terminus Font & Spleen Monospace (External Boot / Installer Package)
* **Pembuat Terminus:** Dimitar Zhekov
* **Lisensi Terminus:** SIL Open Font License (OFL) v1.1
* **Pembuat Spleen:** Frederic Cambus
* **Lisensi Spleen:** BSD 2-Clause License
* **Karakteristik:** Font bitmap monospaced ultra-crisp yang dirancang untuk terminal, sistem operasi, dan antarmuka installer.
* **Penempatan:** Direktori instalasi boot `/boot/fonts/`.

---

ASL Core Kernel mempertahankan kemandirian penuh: jika modul font eksternal tidak ditemukan di media boot, mikrokernel secara deterministik menggunakan ASL Builtin Avionics Font.
