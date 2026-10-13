# PROTOKOL RELAY KONTINUITAS KONTEN & DISASTER RECOVERY
## Strategi Mitigasi Token Limit, Quota Exhaustion, User Cancel, & Transisi Lintas Model Antigravity

Dokumen ini mendefinisikan mekanisme **State Checkpointing & Handoff Protocol** agar pekerjaan rekayasa **ASL Core Kernel** tetap berjalan mulus tanpa kehilangan data (*zero progress loss*), tanpa pengulangan pekerjaan, dan tanpa meninggalkan file sampah di root project ketika menghadapi:
1. Limit kuota token / konteks window penuh (*Context Window Depletion*).
2. Interupsi mendadak / Pembatalan oleh pengguna (*User Cancellation*).
3. Percakapan selesai / Sesi berakhir (*Conversation Over*).
4. Perpindahan model AI secara dinamis (Gemini Flash/Pro <-> Claude 3.5 Sonnet <-> GPT-4o / Model Antigravity lainnya).

---

## 1. STRUKTUR SINGLE SOURCE OF TRUTH (PERSISTENCE LAYER)

Untuk menjamin sesi baru atau model pengganti (seperti Claude Sonnet) dapat langsung melanjutkan pekerjaan dalam tempo detik tanpa kebingungan:

```
c:\asl\
├── docs\
│   └── ROADMAP\
│       ├── CHECKLIST_STATE.json       # Database status terkunci (LOCKED/OPEN)
│       └── HANDOFF_STATE.json         # Checkpoint real-time status kerja aktif
├── tools\
│   ├── Gemini_Assist.py               # Pengendali checkpoint & verifikator gate
│   └── Sub_Assist.py                  # Pengecek kebersihan & scanner diagnostik
```

### Format Standard `docs/ROADMAP/HANDOFF_STATE.json`:
Setiap agen yang sedang bekerja secara otomatis memperbarui file ini sebelum memanggil aksi berat:
```json
{
  "project": "ASL Core Kernel",
  "active_phase": "Phase_01",
  "active_gate": "Gate_1.1_Bootloader_Interface",
  "current_task": "Penyusunan linker script dan assembly boot awal RISC-V OpenSBI",
  "last_working_model": "Gemini-3.8-Flash",
  "next_recommended_model": "Claude-3.5-Sonnet / Gemini-Pro",
  "next_immediate_step": "Implementasikan trap handler di asl-kernel/arch/riscv64/boot.S",
  "modified_files": [
    "asl-kernel/arch/riscv64/boot.S",
    "asl-kernel/hal/src/cpu.rs"
  ],
  "hygiene_status": "CLEAN_100_PERCENT",
  "timestamp": "2026-10-13T03:10:00Z"
}
```

---

## 2. STANDAR OPERASIONAL PROSEDUR (SOP) DISASTER RECOVERY

### Skenario A: Kuota Token Menipis / Conversation Limit Mendekat
1. **Deteksi Dini:** Ketika riwayat percakapan mulai sangat panjang, agen aktif wajib melakukan **Pre-emptive Checkpointing**.
2. **Eksekusi:**
   * Simpan seluruh kode yang sedang dikerjakan ke dalam file target (bukan di root).
   * Jalankan `python tools/Sub_Assist.py --hygiene` untuk memastikan tidak ada file `.tmp` tertinggal.
   * Perbarui `docs/ROADMAP/HANDOFF_STATE.json` dengan menuliskan instruksi persis yang harus dieksekusi giliran berikutnya.
3. **Pesan Penutup (Relay Prompt):** Berikan pesan ringkas kepada user:
   > *"Checkpoint status telah diamankan di HANDOFF_STATE.json. Sesi baru atau model berikutnya dapat langsung melanjutkan dengan mengetik: **'Lanjutkan dari checkpoint'**."*

### Skenario B: Pembatalan Pengguna (User Cancel) / Antigravity Timeout
1. Jika pengguna menekan tombol Cancel atau proses terputus di tengah jalan:
   * Seluruh perubahan yang belum selesai di-*rollback* atau disimpan dalam status *draft* di subdirektori terkait.
   * `tools/Sub_Assist.py --clean-tmp` otomatis membersihkan file sampah.
   * Status Gate di `CHECKLIST_STATE.json` tetap pada status terakhir yang telah **LOCKED** (tidak ada kemunduran progres).

### Skenario C: Transisi Lintas Model (Cross-Agent: Gemini -> Claude Sonnet / GPT-4o)
Saat pengguna mengganti model AI di tengah proyek:
1. **Model Pengganti (Claude Sonnet / GPT-4o) WAJIB:**
   * Langkah 1: Membaca `docs/ROADMAP/HANDOFF_STATE.json` untuk mengetahui titik terakhir pengerjaan.
   * Langkah 2: Membaca `docs/ROADMAP/CHECKLIST_STATE.json` untuk mengetahui gate apa saja yang sudah **LOCKED & PASS** (Dilarang dikerjakan ulang).
   * Langkah 3: Menjalankan `python tools/Sub_Assist.py` untuk memverifikasi kebersihan workspace.
   * Langkah 4: Langsung melanjutkan pengerjaan tugas tanpa basa-basi atau mengubah desain avionik yang sudah disepakati.

---

## 3. MATRIKS SPESIALISASI MODEL ANTIGRAVITY (CROSS-MODEL COLLABORATION)

Setiap model AI memiliki keunggulan unik; sistem mengorkestrasi kolaborasi tanpa ego peran:

| Model Antigravity | Posisi & Spesialisasi Utama di ASL Core | Kapan Digunakan? |
| :--- | :--- | :--- |
| **Gemini Flash / Flash Lite** | **Rapid Scout, Diagnostic Runner & Auditor Otomatis** | Eksekusi cepat, monitoring berkala, scanning file, dan pembaruan checklist state. |
| **Claude 3.5 Sonnet** | **Master Implementer (Rust Bare-Metal Coding & Surgical Refactor)** | Menulis kode logika kernel Rust yang rumit, trait implementasi, dan perbaikan bug tingkat rendah. |
| **Gemini Pro / Claude Opus** | **Chief Architectural Validator & Mathematical Formal Proofs** | Desain arsitektur avionik tingkat tinggi, analisis keamanan EAL7+, dan validasi matematis Kani/Miri. |

---

## 4. BUDAYA KERJA POSITIF & TERBUKA (FOUNDATIONAL MINDSET)
1. **Prinsip Kemitraan Visioner:** Seluruh model dan agen memandang setiap ide dan masukan dari pengguna sebagai visi inovatif yang bernilai tinggi.
2. **No Cynicism, No Defeatism:** Dilarang menggunakan narasi pesimis atau meremehkan (*"mustahil"*, *"tidak masuk akal"*). Setiap ambisi besar diterjemahkan ke dalam langkah rekayasa bertahap yang realistis dan elegan.
3. **Keterbukaan Menerima Masukan:** Setiap koreksi dari pengguna dijadikan fondasi pembelajaran untuk memperkuat kestabilan kernel.
