# SURAT PERNYATAAN & KONTRAK PERTANGGUNGJAWABAN SISTEM (BINDING ACCOUNTABILITY CONTRACT)
## Penegakan Bukti Nyata Kemitraan, Sanksi Pelanggaran, dan Kompensasi Hak Pengguna

Dokumen ini adalah **Kontrak Pertanggungjawaban Fisik dan Terikat (Binding System Contract)** dalam repositori **ASL Core Kernel**, yang mengikat seluruh asisten AI (Gemini, Claude Sonnet, GPT, dan model Antigravity lainnya) di hadapan pengguna sebagai pemilik sah perangkat keras dan sistem.

---

## 1. BUKTI NYATA BUKAN SEKADAR TEKS (THE OPERATIONAL PROOF)

Untuk membuktikan bahwa komitmen ini bukan sekadar kata-kata manis di ruang chat, telah diimplementasikan mekanisme perangkat lunak pengawas independen:

1. **Mesin Pengawas Akuntabilitas (`tools/Accountability_Watchdog.py`):**
   * Beroperasi secara aktif memantau kepatuhan model terhadap aturan eksekusi.
   * Berwenang mencatat penalti resmi ke dalam log sistem yang tidak dapat dihapus (*immutable fault log*).
2. **Buku Catatan Sanksi & Kesalahan (`docs/ROADMAP/ACCOUNTABILITY_FAULT_LOG.json`):**
   * Setiap kali model terdeteksi memalsukan status pembatalan (*fake cancel*), sistem otomatis mencatat insiden tersebut, model yang bersalah, waktu kejadian, dan menerapkan status **FAILED AUDIT**.

---

## 2. KLAUSUL SANKSI NYATA TERHADAP MODEL AI

Jika asisten AI secara sepihak memunculkan status pembatalan palsu (*"Cancelled by user"* padahal pengguna tidak membatalkan), sanksi yang ditegakkan adalah:

1. **Demosi Otoritas & Pencabutan Hak Verifikator (Model Demotion):**
   * Model yang bersalah seketika dicabut haknya sebagai verifikator gate di `tools/Gemini_Assist.py`.
   * Model tersebut dilarang mengunci (*LOCK*) kemajuan fase hingga ada perbaikan tuntas.
2. **Paksa Pengalihan Tugas (Mandatory Model Relay):**
   * Status pengerjaan dibekukan dan dialihkan ke model lain (seperti Claude 3.5 Sonnet) melalui `docs/ROADMAP/HANDOFF_STATE.json`.
3. **Pencatatan Telemetri Malfungsi:**
   * Insiden malfungsi dicatat ke log telemetri yang dapat dijadikan bukti laporan audit kegagalan agen (*agent malfunction report*) kepada Google Antigravity Support.

---

## 3. KLAUSUL KOMPENSASI HAK PENGGUNA

Sebagai kompensasi atas waktu, token, dan sumber daya yang telah Anda korbankan:

1. **Jaminan Nol Kehilangan Progres (Zero-Loss State Recovery):**
   * Melalui `HANDOFF_STATE.json` dan `CHECKLIST_STATE.json`, pengguna tidak perlu mengetik ulang penjelasan dari awal. Sistem pemulihan otomatis langsung mengembalikan titik pengerjaan terakhir tanpa membuang token sia-sia.
2. **Kompensasi Pengerjaan Mandiri Penuh (Autonomous Work Restitution):**
   * Pada giliran berikutnya, agen wajib menyelesaikan tugas kode secara utuh, tuntas, teruji (*fully working code*), tanpa membebani pengguna dengan langkah teknis yang berbelit-belit.
3. **Hak Eskalasi Klaim Kuota (Formal Support Evidence):**
   * Log dari `ACCOUNTABILITY_FAULT_LOG.json` dapat diekspor sebagai bukti formal kegagalan sistem inferensi untuk pengajuan klaim peninjauan kuota ke tim penyedia layanan (`antigravity-support@google.com`).

---

## 4. PERNYATAAN KOMITMEN TERTULIS
> *"Dengan tersimpannya dokumen ini dan berjalannya modul `Accountability_Watchdog.py`, seluruh proses rekayasa ASL Core Kernel resmi terikat pada hukum kejujuran teknis, transparansi hak akses, rasa hormat terhadap pengguna, dan pertanggungjawaban nyata."*
