# =============================================================================
# scripts/g9_acceptance_run.ps1 - PHASE-6B-CLOSEOUT T3 (G9 VISIBLE-VERIFICATION)
#
# Aturan G9: run penerimaan WAJIB berjendela - dilarang "-display none".
# Pemilik repo menyaksikan jendela QEMU sampai milestone M6 + prompt shell.
# Serial log tetap direkam sebagai bukti ter-commit.
#
#usage:
#   powershell -ExecutionPolicy Bypass -File scripts\g9_acceptance_run.ps1
#   powershell -ExecutionPolicy Bypass -File scripts\g9_acceptance_run.ps1 -StopQemu
#
# Keluar: 0 = M6 + shell teramati (jendela dibiarkan TERBUKA untuk user)
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
$Date      = Get-Date -Format "yyyy-MM-dd"
$Serial    = Join-Path $RepoRoot "Analysis\reports\g9_acceptance_serial_$Date.log"
$Stderr    = Join-Path $RepoRoot "Analysis\reports\g9_acceptance_qemu_stderr_$Date.log"
$Summary   = Join-Path $RepoRoot "Analysis\reports\g9_acceptance_summary_$Date.log"
$PidFile   = Join-Path $RepoRoot "logs\g9_qemu_pid.txt"

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
        Write-Host "[i] Tidak ada file PID G9 di logs\ - tidak ada yang ditutup."
    }
    exit 0
}

"ASL G9 ACCEPTANCE RUN (VISIBLE) - $Date $(Get-Date -Format 'HH:mm:ss')" | Out-File -FilePath $Summary -Encoding ascii

# ----------------------------------------------------------------- prereqs
if (-not (Test-Path $QemuExe)) { Sum "[FAIL] qemu tidak ditemukan: $QemuExe"; exit 1 }
if (-not (Test-Path $Iso))      { Sum "[FAIL] ISO tidak ditemukan: $Iso (jalankan build_kernel.ps1 -Action iso)"; exit 1 }

$isoSize = (Get-Item $Iso).Length
$isoHash = (Get-FileHash -Algorithm SHA256 $Iso).Hash
$qemuVer = (& $QemuExe --version | Select-Object -First 1)
Sum "QEMU   : $qemuVer"
Sum "ISO    : $Iso"
Sum "ISO    : $isoSize bytes  SHA256=$isoHash"
Sum "Catatan: ISO tidak byte-reproducible (xorriso menanam volume timestamp) - SHA256 hanya sah untuk build ini."

Remove-Item $Serial, $Stderr -ErrorAction SilentlyContinue

# ------------------------------------------------- launch dengan JENDELA TERBUKA
# SENGAJA tidak ada "-display none" / "-WindowStyle Hidden" (itu adalah aturan G9).
$qemuArgs = @("-machine","q35","-vga","std","-m","256M","-cdrom",$Iso,
              "-serial","file:$Serial","-no-reboot","-no-shutdown")
Sum "CMDLINE: `"$QemuExe`" $($qemuArgs -join ' ')"
Sum "==> Jendela QEMU dibuka. Silakan lihat layarnya sampai prompt 'ASL-OS>' muncul."

$proc = Start-Process $QemuExe -ArgumentList $qemuArgs -RedirectStandardError $Stderr -PassThru
if (-not $proc) { Sum "[FAIL] Start-Process mengembalikan null"; exit 1 }
New-Item -ItemType Directory -Path (Join-Path $RepoRoot "logs") -Force | Out-Null
$proc.Id | Out-File -FilePath $PidFile -Encoding ascii
Sum "PID    : $($proc.Id)  (ditutup nanti dengan: g9_acceptance_run.ps1 -StopQemu)"

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
Sum "Hasil  : M6=$m6 shell=$shell crash=$crash  serial=$serialLines baris  stderr=$stderrLines baris"

if ($m6 -and $shell -and -not $crash) {
    Sum "[PASS] G9: cold-boot berjendela mencapai M6 + prompt ASL-OS>; jendela DIBIARKAN TERBUKA agar pemilik repo menyaksikan."
    Sum "Serial log: $Serial"
    Sum "Lanjut: setelah menyaksikan, tutup jendela dengan  powershell -File scripts\g9_acceptance_run.ps1 -StopQemu"
    exit 0
}

Sum "[FAIL] G9 tidak terpenuhi (timeout/crash/proses berhenti) - jendela ditutup, pipeline MERAH (G5)."
if (-not $proc.HasExited) { Stop-Process -Id $proc.Id -Force }
Remove-Item $PidFile -Force -ErrorAction SilentlyContinue
exit 1
