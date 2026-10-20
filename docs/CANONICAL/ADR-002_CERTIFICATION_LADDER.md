# ADR-002: TANGGA SERTIFIKASI — DO-178C-ALIGNED, LADDER DAL-C → DAL-B → DAL-A

- **Status:** ACCEPTED (2026-10-20)
- **Decision Driver:** AMENDMENT-2 (FASE-4): klaim "DO-178C DAL-A" sebagai status *saat ini* tidak dapat dibuktikan dengan bukti yang ada di repo; prinsip proyek sendiri — EVIDENCE-FIRST dan G7 (jangan menebak angka/klaim) — menuntut pernyataan status mencerminkan bukti.
- **Relationship to ADR-001:** komplemen. ADR-001 memutuskan *bahasa*; ADR-002 memutuskan *klaim proses sertifikasi*. Tidak men-supersede konten teknis ADR-001; referensi "DAL-A" di dalam ADR-001 (mis. penurunan beban MC/DC) dibaca sebagai *aspirasi ujung tangga*, bukan status.

## 1. Keputusan
1. Proses rekayasa proyek dinyatakan **"DO-178C-aligned"** (selaras dengan praktik DO-178C: PSAC/SDP-style discipline, traceability dokumen→kode, negative verification testing, accountability fault log), **bukan** "DO-178C DAL-A certified" atau DAL-A *in effect*.
2. **Target sertifikasi pertama = DAL-C pada SATU arsitektur (x86_64-unknown-none)** — single-architecture, single-DAL certification increment (v1.1).
3. **DAL-A tetap aspirasi jangka panjang** di ujung tangga; seluruh dokumen yang semula menulis "Governing Standard: DO-178C DAL-A" kini menulis **"DO-178C-aligned, ladder DAL-C→A (lihat ADR-002)"**.

## 2. Tangga dan exit criteria per anak tangga
| Anak tangga | Target | Exit criteria (harus TERBUKTI, bukan diklaim) |
|---|---|---|
| **DAL-C** | rilis **v1.1**, x86_64 saja | (a) MC/DC coverage terukur pada semua logika keamanan-spesifik (bukan hanya structural/line) dengan tool terkuifikasi; (b) requirement→source→test traceability matrix lengkap untuk scope x86_64; (c) **DO-330 tool qualification untuk Gemini_Assist dan Sub_Assist (TQL-1/TQL-2 sesuai peran)** — SEBELUM output keduanya boleh dipakai sebagai bukti verifikasi; (d) hardware verification gate `REMEDIATION_PLAN.md §5` P1 selesai (QM-01 VT-d smoke, injeksi runtime STACK OVERFLOW end-to-end); (e) keterlibatan otoritas/conformity assessment body (EASA/NRA atau delegasi) formal, bukan self-assessment; (f) verifikasi oleh **tim verifikasi independen** (bukan agen penulis kode, bukan auditor yang sama dengan pengembang). |
| **DAL-B** | v1.2+ | Seluruh kriteria DAL-C + tambahan: formal methods di jalur kritis (`RND_07`), WCET terukur pada hardware riil (bukan anggaran), review proses sesuai table appendix-A DO-178C untuk DAL-B, arsitektur kedua hanya bila bukti BSP lengkap. |
| **DAL-A** | jangka panjang | Seluruh kriteria DAL-B + instrumentasi/coverage penuh DAL-A, independence review of certification basis, qualifying environment (konformitas 21 CFR-style independence), DMA-CACHE-01 penuh (barrier HW riil), ATOMIC-SPARC-01 tertutup di target terkait, dan dual-core lockstep/fault-coverage evidence avionik. |

## 3. Justifikasi berbasis bukti (kenapa klaim DAL-A saat ini tidak dapat dipertahankan)
1. **Tidak ada keterlibatan otoritas:** tidak ada satu pun artefak korespondensi/conformity assessment dengan EASA/NRA/DER di repo; register audit seluruhnya self/agent-audited.
2. **Tidak ada MC/DC coverage:** yang terbukti adalah host unit tests (160/160, `cargo test --workspace --lib`, 2026-10-20) — itu *functional testing*, bukan pemenuhan criteria coverage DAL-A; tidak ada coverage tool/angka MC/DC yang tercatat.
3. **Tool qualification belum ada (DO-330):** Gemini_Assist / Sub_Assist menghasilkan kode dan laporan yang masuk dokumen kanonik; tanpa TQL, outputnya tidak boleh menjadi bukti verifikasi — dan hari ini tidak ada TQL.
4. **Hardware verification gate terbuka:** DMA-ADDR-01 = MITIGATED (CODE) dengan VERIFIED-ON-HARDWARE PENDING; DMA-CACHE-01 PARTIAL; QM-01 belum dieksekusi; Jalur reset HW (DA-04) menunggu BSP.
5. Konsistensi ke depan: klaim berlebihan adalah temuan audit di standar ini sendiri (preseden: klaim hantu "340 tests" ditolak dan dicatat sebagai `FALSE_CLAIM_INSTRUCTION` di `ACCOUNTABILITY_FAULT_LOG.json`, 2026-10-20).

## 4. Yang TIDAK berubah
- Tidak ada perubahan kode `.rs`, `CHECKLIST_STATE.json`, atau status gate.
- "DAL-A" yang merupakan **nama literal artefak teknis** (mis. `compile_error!` guard `IdentityDmaSpace` di `asl-hal/src/dma_space.rs:8`, fitur cargo `dal-a`, komentar kontrak kode) TETAP apa adanya — ia mendeskripsikan *kebijakan build untuk masa DAL-A*, bukan klaim status proyek. Dokumen kanonik cukup menunjuk ADR ini.
- Proses keras yang sudah berjalan (G1–G7, append-only log, fault log, negative testing, evidence-first) tetap — justru menjadi fondasi anak tangga DAL-C.

## 5. Aturan penulisan status (menurunkan ADR ini jadi kebijakan dokumen)
- Header dokumen: `Governing Standard: DO-178C-aligned, ladder DAL-C→A (ADR-002)`.
- Dilarang menulis "certified", "compliant", atau "DAL-A" sebagai *cap status*; DAL-A hanya boleh muncul sebagai *target tangga* dengan referensi ADR-002.
- Setiap klaim verifikasi wajib menyebut kategori: **VERIFIED (host)** / **CODE-COMPLETE** / **VERIFIED-ON-VM** / **VERIFIED-ON-HARDWARE** / **PENDING**.
- **VERIFIED-ON-VM** (ditambahkan 2026-10-20, PHASE-5B): bukti teramati pada mesin emulator dengan device model perangkat keras nyata (QEMU q35 `-device intel-iommu`) memakai artefak build bare-metal yang sama (bukan binary host); contoh bukti: log serial cold-boot + respons register device model. Kategori ini **TIDAK setara dan TIDAK mengimplikasikan VERIFIED-ON-HARDWARE** — konfirmasi board fisis tetap wajib untuk klaim hardware-final (per ADR-002 anak tangga; QEMU memodelkan semantik perangkat, bukan waktu-nyata/timing silikon).
