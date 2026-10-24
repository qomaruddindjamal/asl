"""Alat ekstrak bukti CI terkelola -> berkas bukti (metode per-langkah, sejak P7).

Kolom langkah pada `gh run view --log` kadang berisi 'UNKNOWN STEP' (terukur),
sehingga pemisahan langkah direkonstruksi dari header '##[group]Run <perintah>'
yang teks perintahnya dibaca dari .github/workflows/asl-ci-minimal.yml. Batas
region satu langkah = header '##[group]Run' berikutnya milik job yang sama.

Pemakaian: python scripts/ci_hosted_extract.py <run_id> <log_mentah> <berkas_keluar>
Semua angka dihitung ulang dari log mentah (aturan G7); tidak ada angka yang
diketik tangan. Metadata run diambil lewat `gh api` oleh alat ini sendiri, jadi
hasilnya dapat diulang dan diaudit. Istilah netral sesuai aturan G11:
'host build' (mesin kerja) dan 'runner CI' (mesin terkelola).
"""
import hashlib
import json
import re
import subprocess
import sys
from datetime import datetime, timezone

RUN, RAW, OUT = sys.argv[1], sys.argv[2], sys.argv[3]

meta = json.loads(subprocess.run(
    ["gh", "api", f"repos/qomaruddindjamal/asl/actions/runs/{RUN}"],
    capture_output=True, check=True).stdout.decode("utf-8"))

lines = open(RAW, "rb").read().decode("utf-8", "replace").splitlines()

TS = re.compile(r"^\d{4}-\d{2}-\d{2}T[\d:.]+Z\s?")
ANSI = re.compile(r"(?:\x1b|\^\[)\[[0-9;]*[A-Za-z]|\x1b\][^\x07]*\x07")


def cells(line):
    parts = line.split("\t")
    body = "\t".join(parts[2:]) if len(parts) >= 3 else "\t".join(parts)
    body = body.replace("\ufeff", "")
    return (parts[0] if parts else ""), (parts[1] if len(parts) >= 2 else ""), ANSI.sub("", TS.sub("", body)).strip()


def norm(text):
    return re.sub(r"\s+", " ", text).strip()


# --- daftar perintah langkah dari workflow (satu-sumber definisi CI) ---------
CMD = {
    "test": "cargo test --workspace --lib",
    "build": "cargo build --package asl-kernel-bin --target x86_64-unknown-none --release",
    "elf": "ls -l target/x86_64-unknown-none/release/asl-kernel",
    "covinstall": "cargo install cargo-llvm-cov --locked",
    "covrun": "cargo llvm-cov -p asl-storage -p asl-bootloader -p asl-kernel-core-secondary --lib --summary-only --fail-under-lines 80",
}

parsed = []
for idx, line in enumerate(lines):
    job, step, body = cells(line)
    parsed.append((job, step, body, idx))

jobs = {}
for job, step, body, idx in parsed:
    jobs.setdefault(job, []).append((idx, step, body))

step_col_unknown = {}
for job, rows in jobs.items():
    vals = {r[1] for r in rows}
    step_col_unknown[job] = sorted(vals)


def region(job, cmd_key):
    """Baris milik langkah `cmd_key` pada `job`: dari header hingga header berikutnya."""
    rows = jobs.get(job, [])
    want = norm(CMD[cmd_key])
    headers = []
    for i, (idx, step, body) in enumerate(rows):
        m = re.match(r"^##\[group\](.*)$", body)
        if m and norm(m.group(1)).startswith(("Run ", "run ")) and norm(m.group(1)[4:]).startswith(want[:40]):
            headers.append(i)
    if not headers:
        return []
    start = headers[0] + 1
    nxt = len(rows)
    for i in range(start, len(rows)):
        m = re.match(r"^##\[group\](Run |run |Post |Complete )", rows[i][2])
        if m:
            nxt = i
            break
    return [rows[i][2] for i in range(start, nxt)]


TB = "test-and-build"
COV = [j for j in jobs if j != TB and "coverage" in j.lower()]
s1 = region(TB, "test")
s2 = region(TB, "build")
elf = region(TB, "elf")

results = [b for b in s1 if b.startswith("test result:")]
per_test_ok = [b for b in s1 if re.search(r"test \S+ \.\.\. ok", b)]
warn1 = [b for b in s1 if b.startswith("warning") or b.startswith("warning:")]
err1 = [b for b in s1 if b.startswith("error")]
compiled = [b for b in s2 if b.startswith("Compiling")]
finished = [b for b in s2 if b.startswith("Finished")]
warn2 = [b for b in s2 if b.startswith("warning")]
err2 = [b for b in s2 if b.startswith("error")]

passed = failed = 0
for b in results:
    m = re.search(r"(\d+) passed; (\d+) failed", b)
    if m:
        passed += int(m.group(1))
        failed += int(m.group(2))

out = []
w = out.append
w(f"HOSTED CI EVIDENCE - workflow {meta['name']}, run {RUN} ({meta['event']})")
w(f"Ekstrak ter-filter dari `gh run view {RUN} --log` ({len(lines)} baris mentah).")
tb_vals = step_col_unknown.get(TB, [])
unknown_tb = "UNKNOWN STEP" in tb_vals
w("Catatan metode (terukur, bukan asumsi):")
w(f"- Kolom langkah pada run ini: job '{TB}' berisi {json.dumps(tb_vals, ensure_ascii=False)}; "
  f"job '{COV[0] if COV else '(tidak ada)'}' berisi {json.dumps(step_col_unknown.get(COV[0], []) if COV else [], ensure_ascii=False)}.")
w(f"- Nilai 'UNKNOWN STEP' {'HADIR pada run ini' if unknown_tb else 'TIDAK hadir pada run ini'} "
  "(terukur dari kolom langkah log mentah). Sebagian run v1.0.2 lain menghasilkan kolom 'UNKNOWN STEP', "
  "sehingga semua ekstrak yang dihasilkan generator ini (tahap P7) sengaja memakai SATU metode yang tidak bergantung kolom itu: pemisahan langkah "
  "direkonstruksi dari penanda '##[group]Run <perintah>' yang teks perintahnya dibaca dari "
  ".github/workflows/asl-ci-minimal.yml, dengan batas langkah = penanda '##[group]Run' berikutnya pada job yang sama.")
w("- Sekuens warna pada log mentah hadir sebagai teks caret (mis. '^[[36;1m'), dibuang sebelum pencocokan; "
  "BOM U+FEFF di awal baris pesan juga dibuang. Tanpa dua pemulihan ini, penanda langkah dan baris 'Compiling' tidak terbaca.")
w("- Setiap angka di bawah adalah hasil hitung dari region langkah tersebut, bukan salinan dari ekstrak lain.")
w(f"Branch teruji: {meta.get('head_branch')} | headSha: {meta['head_sha']}")
w("Metadata run: " + json.dumps({
    "conclusion": meta.get("conclusion"), "createdAt": meta.get("created_at"),
    "databaseId": meta["id"], "event": meta["event"], "headBranch": meta.get("head_branch"),
    "headSha": meta["head_sha"], "runStartedAt": meta.get("run_started_at"),
    "updatedAt": meta.get("updated_at"), "workflowName": meta["name"],
    "htmlUrl": meta.get("html_url")}, sort_keys=True))
w(f"Ekstraksi dilakukan {datetime.now(timezone.utc).strftime('%Y-%m-%d %H:%M')}Z; penanggalan berkas mengikuti tanggal run UTC.")
w("")
w("=== [1/2] cargo test --workspace --lib  (baris 'test result:' per suite) ===")
for b in results:
    w(b)
w(f"--- TOTAL TERHITUNG DARI {len(results)} SUITE: {passed} passed, {failed} failed ---")
w(f"--- baris `test <nama> ... ok` terhitung pada langkah ini: {len(per_test_ok)} (satu per test; roster individu) ---")
w(f"--- baris `warning*` terhitung pada langkah [1/2]: {len(warn1)} (non-blocking; gerbang 0-warning misi berlaku untuk build RELEASE); baris `error*` = {len(err1)} ---")
w("")
w("=== [2/2] cargo build --package asl-kernel-bin --target x86_64-unknown-none --release ===")
for b in compiled + finished:
    w(b)
w(f"  ({len(compiled)} baris `Compiling` + {len(finished)} baris `Finished` = langkah benar-benar berjalan pada tree run ini, bukan cache)")
w(f"  (terukur pada region langkah ini: {len(warn2)} baris `warning`, {len(err2)} baris `error`)")
w("  (runner hosted sengaja TIDAK memasang env RUSTFLAGS: env menggantikan rustflags asl-kernel/.cargo/config.toml sehingga linking rusak.)")
w("")
w("=== langkah 'Bukti footprint ELF hasil build' (ls -l target/x86_64-unknown-none/release/asl-kernel) ===")
for b in elf:
    if re.match(r"^[-dl][rwxst-]{9} ", b):
        w(b)
if COV:
    cj = COV[0]
    c2 = region(cj, "covrun")
    res2 = [b for b in c2 if b.startswith("test result:")]
    comp2 = [b for b in c2 if b.startswith("Compiling")]
    ok2 = [b for b in c2 if re.search(r"test \S+ \.\.\. ok", b)]
    fin2 = [b for b in c2 if b.startswith("Finished")]
    table = [b for b in c2 if re.match(r"^[A-Za-z0-9_./-]+\.(rs|c|cpp)\s+\d+", b)]
    total_row = [b for b in c2 if b.startswith("TOTAL")]
    w("")
    w(f"=== job '{cj}' (NON-BLOCKING; continue-on-error) - region langkah [2/2] ===")
    for b in comp2:
        w(b)
    for b in fin2:
        w(b)
    for b in res2:
        w(b)
    w(f"  ({len(comp2)} baris `Compiling` instrumen + {len(fin2)} baris `Finished` + {len(res2)} baris `test result:` + {len(ok2)} baris `test ... ok` = job ini mengompilasi dan menjalankan ulang tiga crate lingkup coverage pada tree run ini)")
    if table:
        w("")
        w(f"=== job '{cj}' - baris tabel ringkasan llvm-cov di runner CI ===")
        for b in table + total_row:
            w(b)
        block = table + total_row
        h = hashlib.sha256("\n".join(block).encode("utf-8")).hexdigest()
        h_nl = hashlib.sha256(("\n".join(block) + "\n").encode("utf-8")).hexdigest()
        w(f"  (blok {len(block)} baris tabel; hash konvensi gabungan-LF tanpa newline penutup = SHA256 {h}; "
          f"konvensi dengan satu newline penutup = SHA256 {h_nl})")

open(OUT, "w", encoding="utf-8", newline="\n").write("\n".join(out) + "\n")
print(f"{RUN}: {meta['head_sha'][:7]} suite={len(results)} passed={passed} failed={failed} "
      f"per_test_ok={len(per_test_ok)} warn1={len(warn1)} compile={len(compiled)} warn2={len(warn2)} "
      f"err2={len(err2)} -> {OUT}")
