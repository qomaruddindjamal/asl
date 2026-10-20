# =============================================================================
# scripts/ci_local.ps1 - PHASE-5D CI minimal (local runner, no hosted service)
#
# Empat langkah, berhenti pada kegagalan pertama (fail-closed = pipeline
# MERAH => tidak boleh merge, lihat kebijakan di bawah):
#   [1/4] cargo test --workspace --lib
#   [2/4] cargo build --package asl-kernel-bin --target x86_64-unknown-none
#         --release  (WAJIB 0 error DAN 0 warning)
#   [3/4] QEMU q35 cold-boot smoke: ISO release HARUS mencapai M6 + shell
#   [4/4] Ringkasan + bukti (ISO size/SHA256, footprint ELF, commit, hasil)
#
# Kebijakan merge (dinyatakan eksplisit, bukan tooling hosted):
#   - Pipeline MERAH = DILARANG merge/push ke branch utama.
#   - Artefak wajib: Analysis/reports/ci_<YYYY-MM-DD>.log (di-commit).
#
#usage:
#   powershell -ExecutionPolicy Bypass -File scripts\ci_local.ps1
#   optional: -SkipBuildISO  ( reuse ISO yang sudah ada di target\ )
# =============================================================================
param(
    [switch]$SkipBuildISO
)
$ErrorActionPreference = "Continue"
$RepoRoot = "C:\asl"
$Workspace = Join-Path $RepoRoot "asl-kernel"
$TargetArch = "x86_64-unknown-none"
$HostTarget = "x86_64-pc-windows-msvc"
$QemuExe = "C:\Program Files\qemu\qemu-system-x86_64.exe"
$Iso = Join-Path $Workspace "target\asl-core-kernel.iso"
$Elf = Join-Path $Workspace "target\$TargetArch\release\asl-kernel"
$Date = Get-Date -Format "yyyy-MM-dd"
$Log = Join-Path $RepoRoot "Analysis\reports\ci_$Date.log"
$VmSerial = Join-Path $RepoRoot "Analysis\reports\qemu_ci_smoke_serial.log"
$VmStderr = Join-Path $RepoRoot "Analysis\reports\qemu_ci_smoke_qemu_stderr.log"

Set-Location $Workspace
"ASL CI LOCAL - $Date $(Get-Date -Format 'HH:mm:ss')" | Out-File -FilePath $Log -Encoding ascii
# Catat langkah ke log teks-ASCII (bukan UTF-16 Tee-Object) agar di-commit
# sebagai bukti yang bisa dibaca `git diff`.
function Log($m) {
    Write-Host $m
    Add-Content -Path $Log -Value $m -Encoding ascii
}

$commit = "unknown"
$dirty = ""
& git -C $RepoRoot rev-parse --short HEAD 2>$null | ForEach-Object { $commit = $_ }
& git -C $RepoRoot status --porcelain 2>$null | ForEach-Object { $dirty += "dirty" }
if ($dirty) { $commit += "+dirty" }
Log "commit: $commit"
Log "toolchain: $((rustc -V)) / $((cargo -V))"

$results = @()
$failStep = ""

# ---------------------------------------------------------------- [1/4] tests
Log ""
Log "=== [1/4] cargo test --workspace --lib ==="
& cargo test --workspace --target $HostTarget --lib 2>&1 | Out-File -FilePath "$Log.step1.tmp" -Encoding ascii
Copy-Item "$Log.step1.tmp" "$Log.step1.log" -Force
$testsPassed = 0
$testsFailed = 0
foreach ($line in (Get-Content "$Log.step1.log")) {
    if ($line -match "test result: ok\. (\d+) passed") { $testsPassed += [int]$Matches[1] }
    if ($line -match "test result: FAILED\. \d+ passed; (\d+) failed") { $testsFailed += [int]$Matches[1] }
}
if ($LASTEXITCODE -eq 0 -and $testsFailed -eq 0 -and $testsPassed -gt 0) {
    Log "[1/4] PASS - $testsPassed host tests passed, 0 failed"
    $results += "[1/4] tests: PASS ($testsPassed passed / 0 failed)"
} else {
    Log "[1/4] FAIL - exit=$LASTEXITCODE passed=$testsPassed failed=$testsFailed"
    Log (Get-Content "$Log.step1.log" | Select-String -Pattern "^error|FAILED|panicked" | Select-Object -First 20 | Out-String)
    $results += "[1/4] tests: FAIL (exit=$LASTEXITCODE passed=$testsPassed failed=$testsFailed)"
    $failStep = "1/4"
}
Remove-Item "$Log.step1.tmp" -ErrorAction SilentlyContinue

# --------------------------------------------------------------- [2/4] build
if (-not $failStep) {
    Log ""
    Log "=== [2/4] cargo build --package asl-kernel-bin --target $TargetArch --release (0 error / 0 warning) ==="
    & cargo build --package asl-kernel-bin --target $TargetArch --release 2>&1 | Out-File -FilePath "$Log.step2.log" -Encoding ascii
    $warnLines = @(Get-Content "$Log.step2.log" | Select-String -Pattern "^warning" )
    $errLines = @(Get-Content "$Log.step2.log" | Select-String -Pattern "^error")
    $elfSize = if (Test-Path $Elf) { (Get-Item $Elf).Length } else { 0 }
    if ($LASTEXITCODE -eq 0 -and $warnLines.Count -eq 0 -and $errLines.Count -eq 0 -and $elfSize -gt 0) {
        Log "[2/4] PASS - 0 error, 0 warning, ELF $elfSize bytes"
        $results += "[2/4] bare-metal build: PASS (0 error / 0 warning, ELF $elfSize B)"
    } else {
        Log "[2/4] FAIL - exit=$LASTEXITCODE errors=$($errLines.Count) warnings=$($warnLines.Count) elf=$elfSize"
        Log (($warnLines + $errLines | Select-Object -First 20 | ForEach-Object { $_.ToString() }) -join "`n")
        $results += "[2/4] bare-metal build: FAIL (exit=$LASTEXITCODE warnings=$($warnLines.Count))"
        $failStep = "2/4"
    }
} else {
    Log ""
    Log "=== [2/4] SKIPPED (langkah 1 gagal - fail-closed) ==="
    $results += "[2/4] bare-metal build: SKIPPED"
}

# ----------------------------------------------------------- [3/4] QEMU smoke
if (-not $failStep) {
    Log ""
    Log "=== [3/4] QEMU q35 cold-boot smoke (harus capai M6 + shell) ==="
    if (-not $SkipBuildISO) {
        & powershell -ExecutionPolicy Bypass -File (Join-Path $RepoRoot "scripts\build_kernel.ps1") -Action iso 2>&1 |
            Out-File -FilePath "$Log.step3_iso.log" -Encoding ascii
        if (-not (Test-Path $Iso)) { Log "[3/4] FAIL - ISO tidak terbentuk"; $failStep = "3/4" }
    }
    if (-not $failStep) {
        Remove-Item $VmSerial, $VmStderr -ErrorAction SilentlyContinue
        $isoSize = (Get-Item $Iso).Length
        $isoHash = (Get-FileHash -Algorithm SHA256 $Iso).Hash
        Log "ISO: $isoSize bytes SHA256=$isoHash"
        $qargs = @("-machine", "q35", "-vga", "std", "-m", "256M", "-cdrom", $Iso,
            "-serial", "file:$VmSerial", "-display", "none", "-no-reboot", "-no-shutdown")
        $proc = Start-Process $QemuExe -ArgumentList $qargs -RedirectStandardError $VmStderr -PassThru
        $deadline = (Get-Date).AddSeconds(120)
        $reached = $false
        $crash = $false
        while ((Get-Date) -lt $deadline) {
            Start-Sleep -Seconds 3
            if (Test-Path $VmSerial) {
                $txt = Get-Content $VmSerial -Raw -ErrorAction SilentlyContinue
                if ($txt) {
                    if ($txt.Contains("ASL-OS>")) { $reached = $true; break }
                    if ($txt.Contains("ASL EXCEPTION") -or $txt.Contains("ASL-CORE-PANIC") -or $txt.Contains("FATAL")) { $crash = $true; break }
                }
            }
            if ($proc.HasExited) { break }
        }
        if (-not $proc.HasExited) { Stop-Process -Id $proc.Id -Force }
        $serialLines = if (Test-Path $VmSerial) { @(Get-Content $VmSerial).Count } else { 0 }
        $stderrLines = if (Test-Path $VmStderr) { @(Get-Content $VmStderr).Count } else { 0 }
        $m6 = $false
        if (Test-Path $VmSerial) { $m6 = (Get-Content $VmSerial -Raw).Contains("M6: Kernel Cold Boot Complete") }
        if ($reached -and $m6 -and -not $crash) {
            Log "[3/4] PASS - M6 tercapai + prompt shell (serial $serialLines baris, stderr QEMU $stderrLines baris)"
            $results += "[3/4] QEMU smoke: PASS (M6 reached, serial $serialLines lines)"
        } else {
            Log "[3/4] FAIL - reached=$reached m6=$m6 crash=$crash serial=$serialLines"
            $results += "[3/4] QEMU smoke: FAIL (reached=$reached m6=$m6 crash=$crash)"
            $failStep = "3/4"
        }
    } else {
        Log "[3/4] FAIL - ISO absent"
        $results += "[3/4] QEMU smoke: FAIL (ISO absent)"
    }
} else {
    Log ""
    Log "=== [3/4] SKIPPED (langkah sebelumnya gagal - fail-closed) ==="
    $results += "[3/4] QEMU smoke: SKIPPED"
}

# ------------------------------------------------------------- [4/4] summary
Log ""
Log "=== [4/4] RINGKASAN PIPELINE ==="
foreach ($r in $results) { Log "  $r" }
$elfSizeFinal = if (Test-Path $Elf) { (Get-Item $Elf).Length } else { 0 }
$isoSizeFinal = if (Test-Path $Iso) { (Get-Item $Iso).Length } else { 0 }
Log "  footprint: ELF=$elfSizeFinal B | ISO=$isoSizeFinal B | commit=$commit"
if ($failStep) {
    Log "  KEPUTUSAN: Pipeline MERAH (gagal di langkah $failStep). KEBIJAKAN: TIDAK BOLEH merge ke branch utama sampai langkah tersebut hijau."
    Log "  artefak: $Log"
    exit 1
}
Log "  KEPUTUSAN: Pipeline HIJAU - 4/4 langkah lulus. Merge diizinkan sesuai aturan fase (G6: hanya setelah gerbang hijau)."
Log "  artefak: $Log"
exit 0
