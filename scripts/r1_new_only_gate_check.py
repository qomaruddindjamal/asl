"""Alat gerbang R1: bukti "nol deviasi/diagnostik BARU" pada baris hasil kerja.

Dua gerbang misi (rustfmt bersih, clippy -D warnings bersih) tidak dapat
ditegakkan secara harfiah di cabang ini karena warisan pra-E8; menyapu warisan
berarti mengubah berkas di luar cakupan. Alat ini mengukur Irisan, bukan nol
mutlak:

  1. rentang baris hasil kerja = `git diff -U0` atas berkas yang diubah;
  2. baris yang INGIN diubah perkakas = `rustfmt --check` (hanya-baca, di
     tempat) dan keluaran pendek `cargo clippy`;
  3. irisan keduanya harus kosong.

Cara pakai:
    python scripts/r1_new_only_gate_check.py fmt
    cargo clippy --workspace --lib --target x86_64-unknown-none --message-format=short \
        -- -A clippy::not_unsafe_ptr_arg_deref > logs/clippy_r1_allow.txt
    python scripts/r1_new_only_gate_check.py clippy

Keluar: 0 = nol BARU pada semua berkas; 1 = ada irisan; 2 = alat gagal.
`not_unsafe_ptr_arg_deref` di-allow hanya agar crate di luar cakupan mau
dikompilasi sehingga berkas kerja saya ikut ter-lint; angka warisan dilaporkan
apa adanya.
"""
import re
import subprocess
import sys

REPO = r"C:\asl"
KERNEL = REPO + r"\asl-kernel"
SEP = "[/\\\\]"
FILES = [
    "asl-kernel/asl-kernel-core/src/lib.rs",
    "asl-kernel/asl-cockpit-shell/src/lib.rs",
    "asl-kernel/asl-storage/src/ramdisk.rs",
]
CLIPPY_ARTIFACT = REPO + r"\logs\clippy_r1_allow.txt"


def my_lines(rel):
    """Nomor baris pada berkas kerja yang termasuk hasil kerja (git diff -U0)."""
    d = subprocess.run(["git", "-C", REPO, "diff", "-U0", "--", rel],
                       capture_output=True, text=True, encoding="utf-8").stdout
    s = set()
    for h in re.finditer(r"^@@ -\d+(?:,\d+)? \+(\d+)(?:,(\d+))? @@", d, re.M):
        start, count = int(h.group(1)), int(h.group(2) or 1)
        s.update(range(start, start + count))
    return s


def rustfmt_wanted(path):
    raw = subprocess.run(["rustfmt", "--edition", "2021", "--check", path],
                         capture_output=True, text=True, encoding="utf-8",
                         errors="replace")
    if raw.returncode not in (0, 1):
        print("rustfmt gagal:", raw.stderr[:400])
        sys.exit(2)
    out = re.sub(r"\x1b\[[0-9;]*m", "", raw.stdout)
    wanted, cur, in_hunk = set(), 0, False
    # `--check` atas satu crate root ikut memeriksa submodul (`mod` direkursi),
    # jadi header "Diff in <jalur>:<baris>:" harus disaring: hanya hunk milik
    # berkas yang sedang diukur yang boleh diiriskan.
    target = path.replace("\\", "/")
    for line in out.splitlines():
        m = re.match(r"^Diff in (.*?):(\d+):$", line)
        if m:
            where, cur = m.group(1).replace("\\", "/"), int(m.group(2))
            if not where.endswith(target.split("asl-kernel/")[-1]):
                cur, in_hunk = 0, False
                continue
            in_hunk = True
            continue
        if not in_hunk:
            continue
        if line.startswith("Diff in"):
            in_hunk = False
        elif line.startswith("-"):
            wanted.add(cur)
            cur += 1
        elif not line.startswith("+"):
            cur += 1
    return wanted


def clippy_wanted():
    line_re = re.compile(
        r"^(.*?src" + SEP + r"[\w.]+\.rs):(\d+):\d+: (warning|error): (.*)$")
    diag = []
    try:
        fh = open(CLIPPY_ARTIFACT, encoding="utf-8", errors="replace")
    except OSError as exc:
        print(f"artefak clippy tidak ada: {exc}")
        sys.exit(2)
    with fh:
        for line in fh:
            m = line_re.match(line.strip())
            if m:
                diag.append((m.group(1).replace("\\", "/"), int(m.group(2)),
                             m.group(4)))
    diag = [d for d in diag if "could not compile" not in d[2]]
    print(f"diagnostik clippy ber-baris seluruh workspace : {len(diag)}")
    per = {}
    for f, n, msg in diag:
        per[f] = per.get(f, 0) + 1
    for f in sorted(per, key=lambda k: -per[k])[:8]:
        print(f"  {per[f]:4}  {f}")
    return diag


def main():
    mode = sys.argv[1] if len(sys.argv) > 1 else "fmt"
    if mode not in ("fmt", "clippy"):
        print("pakai: r1_new_only_gate_check.py <fmt|clippy>")
        sys.exit(2)
    diag = clippy_wanted() if mode == "clippy" else []
    total_baru = 0
    for rel in FILES:
        mine = my_lines(rel)
        if mode == "fmt":
            perkakas = rustfmt_wanted(REPO + "\\" + rel.replace("/", "\\"))
            baru = sorted(mine & perkakas)
            print(f"{rel}: baris saya {len(mine)}, ingin diubah rustfmt "
                  f"{len(perkakas)}, IRISAN BARU {len(baru)}")
        else:
            tail = rel.split("/", 1)[1]
            baru = [(f, n, msg) for f, n, msg in diag
                    if f.endswith(tail) and n in mine]
            print(f"{rel}: baris saya {len(mine)}, IRISAN BARU {len(baru)}")
            for b in baru[:10]:
                print("   ", b)
        total_baru += len(baru)
        if mode == "fmt":
            for n in baru[:10]:
                print(f"    {rel}:{n}")
    print(f"TOTAL deviasi/diagnostik BARU = {total_baru}")
    sys.exit(1 if total_baru else 0)


main()
