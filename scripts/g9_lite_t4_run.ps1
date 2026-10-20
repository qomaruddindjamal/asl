# =============================================================================
# scripts/g9_lite_t4_run.ps1 - PHASE-7 T4 (G9-lite VISIBLE-VERIFICATION)
#
# Diturunkan dari scripts/g9_acceptance_run.ps1 (PHASE-6B T3). Tiga perbedaan:
#   1. Nama berkas keluar ber-prefix g9_lite_t4_ dan bercap-waktu per-invokasi,
#      sehingga run ini tidak dapat menimpa artefak G9 2026-10-20 yang ter-commit.
#   2. Gerbang tambahan terukur untuk T4: serial memuat string versi BARU
#      "v1.1.0" dan NOL string lama "v1.0.2" (bukti banner benar-benar tampil
#      pada artefak yang di-boot, bukan hanya di sumber).
#   3. Catatan ISO dikoreksi: sejak PHASE-7 T1 ISO bersifat byte-reproducible
#      per-platform (tanggal image + GUID GPT di-pin), SHA256-nya tetap PER-TREE.
#
# Aturan G9 tetap mengikat: run WAJIB berjendela - "-display none" DILARANG,
# pemilik repo menyaksikan jendela QEMU sampai M6 + prompt shell.
#
#usage:
#   powershell -ExecutionPolicy Bypass -File scripts\g9_lite_t4_run.ps1
#   powershell -ExecutionPolicy Bypass -File scripts\g9_lite_t4_run.ps1 -StopQemu
#
# Keluar: 0 = M6 + shell + gerbang versi terpenuhi (jendela dibiarkan TERBUKA)
#         1 = timeout / crash / ISO tidak ada (fail-closed)
# =============================================================================
param(
    [switch]$StopQemu,
    [int]$TimeoutSeconds = 180
)
$ErrorActionPreference = "Continue"
$RepoRoot  = "C:\asl"
$Workspace = Join-Path $RepoRoot "asl-kernel"
$QemuExe   = "C:\Program Files\qemu\qemu-system-x86_64.exe"
$Iso       = Join-Path $Workspace "target\asl-core-kernel.iso"
$Stamp     = Get-Date -Format "yyyy-MM-dd_HH-mm-ss"
$Serial    = Join-Path $RepoRoot "Analysis\reports\g9_lite_t4_serial_$Stamp.log"
$Stderr    = Join-Path $RepoRoot "Analysis\reports\g9_lite_t4_qemu_stderr_$Stamp.log"
$Summary   = Join-Path $RepoRoot "Analysis\reports\g9_lite_t4_summary_$Stamp.log"
$PidFile   = Join-Path $RepoRoot "logs\g9_lite_t4_qemu_pid.txt"

function Sum($m) {
    Write-Host $m
    Add-Content -Path $Summary -Value $m -Encoding ascii
}

# ------------------------------------------------------------------ stop mode
if ($StopQemu) {
    if (Test-Path $PidFile) {
        $procId = Get-Content $PidFile -Raw
        $p = Get-Process -Id ([int]$procId.Trim()) -ErrorAction SilentlyContinue
        if ($p) { Stop-Process -Id $p.Id -Force; Write-Host "[OK] jendela QEMU (PID $($p.Id)) ditutup." }
        else { Write-Host "[i] PID $procId sudah tidak berjalan." }
        Remove-Item $PidFile -Force
    } else {
        Write-Host "[i] Tidak ada file PID G9-lite T4 di logs\ - tidak ada yang ditutup."
    }
    exit 0
}

"ASL G9-LITE ACCEPTANCE RUN T4 (VISIBLE) - $Stamp" | Out-File -FilePath $Summary -Encoding ascii

# ----------------------------------------------------------------- prereqs
if (-not (Test-Path $QemuExe)) { Sum "[FAIL] qemu tidak ditemukan: $QemuExe"; exit 1 }
if (-not (Test-Path $Iso))      { Sum "[FAIL] ISO tidak ditemukan: $Iso (jalankan build_kernel.ps1 -Action iso)"; exit 1 }

$isoSize = (Get-Item $Iso).Length
$isoHash = (Get-FileHash -Algorithm SHA256 $Iso).Hash
$qemuVer = (& $QemuExe --version | Select-Object -First 1)
Sum "QEMU   : $qemuVer"
Sum "ISO    : $Iso"
Sum "ISO    : $isoSize bytes  SHA256=$isoHash"
Sum "Catatan: sejak PHASE-7 T1 ISO byte-reproducible per-platform (tanggal image + GUID GPT di-pin), namun SHA256 tetap sidik jari PER-TREE - angka ini hanya sah untuk tree T4."

# ------------------------------------------------- launch dengan JENDELA TERBUKA
# SENGAJA tidak ada "-display none" / "-WindowStyle Hidden" (itu aturan G9).
$qemuArgs = @("-machine","q35","-vga","std","-m","256M","-cdrom",$Iso,
              "-serial","file:$Serial","-no-reboot","-no-shutdown")
Sum "CMDLINE: `"$QemuExe`" $($qemuArgs -join ' ')"
Sum "==> Jendela QEMU dibuka. Silakan lihat layarnya sampai prompt 'ASL-OS>' muncul."

$proc = Start-Process $QemuExe -ArgumentList $qemuArgs -RedirectStandardError $Stderr -PassThru
if (-not $proc) { Sum "[FAIL] Start-Process mengembalikan null"; exit 1 }
New-Item -ItemType Directory -Path (Join-Path $RepoRoot "logs") -Force | Out-Null
$proc.Id | Out-File -FilePath $PidFile -Encoding ascii
Sum "PID    : $($proc.Id)  (ditutup nanti dengan: g9_lite_t4_run.ps1 -StopQemu)"

# ----------------------------------------------------------------- polling
$deadline = (Get-Date).AddSeconds($TimeoutSeconds)
$m6 = $false; $shell = $false; $crash = $false
while ((Get-Date) -lt $deadline) {
    Start-Sleep -Seconds 3
    if (Test-Path $Serial) {
        $txt = Get-Content $Serial -Raw -ErrorAction SilentlyContinue
        if ($txt) {
            if ($txt.Contains("M6: Kernel Cold Boot Complete")) { $m6 = $true }
            if ($txt.Contains("ASL-OS>")) { $shell = $true; break }
            if ($txt.Contains("ASL EXCEPTION") -or $txt.Contains("ASL-CORE-PANIC") -or $txt.Contains("FATAL")) { $crash = $true; break }
        }
    }
    if ($proc.HasExited) { break }
}

$serialLines = if (Test-Path $Serial) { @(Get-Content $Serial).Count } else { 0 }
$stderrLines = if (Test-Path $Stderr) { @(Get-Content $Stderr).Count } else { 0 }
$verNew = 0; $verOld = 0
if (Test-Path $Serial) {
    $txt = Get-Content $Serial -Raw -ErrorAction SilentlyContinue
    if ($txt) {
        $verNew = ([regex]::Matches($txt, "v1\.1\.0")).Count
        $verOld = ([regex]::Matches($txt, "v1\.0\.2")).Count
    }
}
Sum "Hasil  : M6=$m6 shell=$shell crash=$crash  serial=$serialLines baris  stderr=$stderrLines baris"
Sum "Versi  : occurrence 'v1.1.0'=$verNew  occurrence 'v1.0.2'=$verOld  (gerbang T4: >=1 dan 0)"

if ($m6 -and $shell -and -not $crash -and $verNew -ge 1 -and $verOld -eq 0) {
    Sum "[PASS] G9-lite T4: cold-boot berjendela mencapai M6 + prompt ASL-OS>, string versi BARU tampil di serial, string lama NOL; jendela DIBIARKAN TERBUKA agar pemilik repo menyaksikan."
    Sum "Serial log: $Serial"
    Sum "Lanjut: setelah menyaksikan, tutup jendela dengan  powershell -File scripts\g9_lite_t4_run.ps1 -StopQemu"
    exit 0
}

Sum "[FAIL] G9-lite T4 tidak terpenuhi (timeout/crash/proses berhenti/gerbang versi gagal) - jendela ditutup, pipeline MERAH (G5)."
if (-not $proc.HasExited) { Stop-Process -Id $proc.Id -Force }
Remove-Item $PidFile -Force -ErrorAction SilentlyContinue
exit 1
