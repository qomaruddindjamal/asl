"""Audit utang ekstrak: untuk tiap commit cabang R1, adakah run CI dan adakah
ekstraknya sudah ter-commit di tree HEAD saat ini? Semua dari git + gh api."""
import io, json, re, subprocess, sys, glob
sys.stdout.reconfigure(encoding="utf-8")

def sh(*a):
    return subprocess.run(a, capture_output=True, check=True, cwd=r"C:\asl").stdout.decode("utf-8")

commits = sh("git", "rev-list", "--reverse", "main..HEAD").split()
head = sh("git", "rev-parse", "--short", "HEAD").strip()
runs = json.loads(sh("gh", "api",
    "repos/qomaruddindjamal/asl/actions/runs?branch=feature/v1.0.2-r1&per_page=100"))["workflow_runs"]
by_sha = {}
for r in runs:
    by_sha.setdefault(r["head_sha"], []).append(r)

# ekstrak yang TER-COMMIT (bukan hanya ada di working tree)
committed = set()
for line in sh("git", "ls-tree", "-r", "--name-only", "HEAD", "Analysis/reports").splitlines():
    m = re.match(r"ci_hosted_(\d+)_.*\.log$", line.split("/")[-1])
    if m:
        committed.add(m.group(1))

print("AUDIT UTANG EKSTRAK HOSTED CI - cabang feature/v1.0.2-r1")
print("HEAD saat audit dijalankan: %s (run milik HEAD belum ter-commit, memang begitu aturan G8)" % head)
print("SUMBER: `git rev-list main..HEAD`, `gh api .../actions/runs?branch=...`, `git ls-tree -r HEAD Analysis/reports`.")
print("SUDUT BACA: baris BELUM di bawah adalah utang yang ditutup commit BERIKUTNYA, bukan kegagalan.")
print()
print("%-8s %-12s %-6s %-6s %s" % ("commit", "run-id", "event", "koncl", "ekstrak-di-repo"))
open_debt, no_run = [], []
for c in commits:
    short = c[:7]
    rs = by_sha.get(c, [])
    if not rs:
        no_run.append(short)
        print("%-8s %-12s %-6s %-6s %s" % (short, "TIDAK ADA RUN", "-", "-", "-"))
        continue
    for r in sorted(rs, key=lambda x: (x["event"] != "push", x["id"])):
        has = str(r["id"]) in committed
        print("%-8s %-12s %-6s %-6s %s" % (short, r["id"], r["event"], r["conclusion"], "ADA" if has else "BELUM"))
        if not has:
            open_debt.append((short, r["id"], r["event"]))

print()
print("total commit cabang:", len(commits))
print("commit tanpa run sendiri:", ", ".join(no_run), "(%d)" % len(no_run))
print("utang terbuka (run ada, ekstrak belum di repo):", open_debt)
print("ekstrak ter-commit total:", len(committed))
