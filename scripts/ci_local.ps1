# =============================================================================
# scripts/ci_local.ps1 - PHASE-5D CI minimal (local runner, no hosted service)
#
# Lima langkah, berhenti pada kegagalan pertama (fail-closed = pipeline
# MERAH => tidak boleh merge, lihat kebijakan di bawah):
#   [1/5] cargo test --workspace --lib
#   [2/5] cargo build --package asl-kernel-bin --target x86_64-unknown-none
#         --release  (WAJIB 0 error DAN 0 warning)
#   [3/5] QEMU q35 cold-boot smoke: ISO release HARUS mencapai M6 + shell
#   [4/5] Gerbang regresi memori boot 3 MB (scripts\test_ram_floor_3mb.ps1)
#   [5/5] Ringkasan + bukti (ISO size/SHA256, footprint ELF, commit, hasil)
#
# Kebijakan merge (dinyatakan eksplisit, bukan tooling hosted):
#   - Pipeline MERAH = DILARANG merge/push ke branch utama.
#   - Artefak wajib: Analysis/reports/ci_<YYYY-MM-DD>.log (di-commit).
#   - Artefak serial QEMU memakai nama bertimestamp supaya bukti yang sudah
#     ter-commit tidak pernah tertimpa oleh jalan berikutnya.
#   - Prompt shell dibaca dari satu-sumbernya (konstanta SHELL_PROMPT di
#     asl-cockpit-shell/src/lib.rs), bukan string yang disalin ke sini.
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
$Clock = Get-Date -Format "HH-mm-ss"
$Log = Join-Path $RepoRoot "Analysis\reports\ci_$Date.log"
$VmSerial = Join-Path $RepoRoot "Analysis\reports\qemu_ci_smoke_serial_${Date}_${Clock}.log"
$VmStderr = Join-Path $RepoRoot "Analysis\reports\qemu_ci_smoke_stderr_${Date}_${Clock}.log"
$PromptSource = Join-Path $Workspace "asl-cockpit-shell\src\lib.rs"

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
$deviationNote = ""

# Prompt shell: satu-sumber dari konstanta Rust, di-parse di sini.
$ShellPrompt = $null
Select-String -Path $PromptSource -Pattern 'pub const SHELL_PROMPT\s*:\s*&str\s*=\s*"([^"]*)"' |
    ForEach-Object { $ShellPrompt = $_.Matches[0].Groups[1].Value }
if (-not $ShellPrompt) {
    Log "KEBIJAKAN: prompt shell tidak terbaca dari $PromptSource - langkah QEMU tidak bisa dinilai."
    $ShellPrompt = "ASL-KERNEL> "
}
Log "prompt satu-sumber: '$ShellPrompt'"

# ---------------------------------------------------------------- [1/5] tests
Log ""
Log "=== [1/5] cargo test --workspace --lib ==="
& cargo test --workspace --target $HostTarget --lib 2>&1 | Out-File -FilePath "$Log.step1.tmp" -Encoding ascii
Copy-Item "$Log.step1.tmp" "$Log.step1.log" -Force
$testsPassed = 0
$testsFailed = 0
foreach ($line in (Get-Content "$Log.step1.log")) {
    if ($line -match "test result: ok\. (\d+) passed") { $testsPassed += [int]$Matches[1] }
    if ($line -match "test result: FAILED\. \d+ passed; (\d+) failed") { $testsFailed += [int]$Matches[1] }
}
if ($LASTEXITCODE -eq 0 -and $testsFailed -eq 0 -and $testsPassed -gt 0) {
    Log "[1/5] PASS - $testsPassed host tests passed, 0 failed"
    $results += "[1/5] tests: PASS ($testsPassed passed / 0 failed)"
} else {
    Log "[1/5] FAIL - exit=$LASTEXITCODE passed=$testsPassed failed=$testsFailed"
    Log (Get-Content "$Log.step1.log" | Select-String -Pattern "^error|FAILED|panicked" | Select-Object -First 20 | Out-String)
    $results += "[1/5] tests: FAIL (exit=$LASTEXITCODE passed=$testsPassed failed=$testsFailed)"
    $failStep = "1/5"
}
Remove-Item "$Log.step1.tmp" -ErrorAction SilentlyContinue

# --------------------------------------------------------------- [2/5] build
if (-not $failStep) {
    Log ""
    Log "=== [2/5] cargo build --package asl-kernel-bin --target $TargetArch --release (0 error / 0 warning) ==="
    & cargo build --package asl-kernel-bin --target $TargetArch --release 2>&1 | Out-File -FilePath "$Log.step2.log" -Encoding ascii
    $warnLines = @(Get-Content "$Log.step2.log" | Select-String -Pattern "^warning" )
    $errLines = @(Get-Content "$Log.step2.log" | Select-String -Pattern "^error")
    $elfSize = if (Test-Path $Elf) { (Get-Item $Elf).Length } else { 0 }
    if ($LASTEXITCODE -eq 0 -and $warnLines.Count -eq 0 -and $errLines.Count -eq 0 -and $elfSize -gt 0) {
        Log "[2/5] PASS - 0 error, 0 warning, ELF $elfSize bytes"
        $results += "[2/5] bare-metal build: PASS (0 error / 0 warning, ELF $elfSize B)"
    } else {
        Log "[2/5] FAIL - exit=$LASTEXITCODE errors=$($errLines.Count) warnings=$($warnLines.Count) elf=$elfSize"
        Log (($warnLines + $errLines | Select-Object -First 20 | ForEach-Object { $_.ToString() }) -join "`n")
        $results += "[2/5] bare-metal build: FAIL (exit=$LASTEXITCODE warnings=$($warnLines.Count))"
        $failStep = "2/5"
    }
} else {
    Log ""
    Log "=== [2/5] SKIPPED (langkah 1 gagal - fail-closed) ==="
    $results += "[2/5] bare-metal build: SKIPPED"
}

# ----------------------------------------------------------- [3/5] QEMU smoke
if (-not $failStep) {
    Log ""
    Log "=== [3/5] QEMU q35 cold-boot smoke (harus capai M6 + shell) ==="
    if (-not $SkipBuildISO) {
        & powershell -ExecutionPolicy Bypass -File (Join-Path $RepoRoot "scripts\build_kernel.ps1") -Action iso 2>&1 |
            Out-File -FilePath "$Log.step3_iso.log" -Encoding ascii
        if (-not (Test-Path $Iso)) { Log "[3/5] FAIL - ISO tidak terbentuk"; $failStep = "3/5" }
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
                    if ($txt.Contains($ShellPrompt)) { $reached = $true; break }
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
            Log "[3/5] PASS - M6 tercapai + prompt shell (serial $serialLines baris, stderr QEMU $stderrLines baris)"
            $results += "[3/5] QEMU smoke: PASS (M6 reached, serial $serialLines lines)"
        } else {
            Log "[3/5] FAIL - reached=$reached m6=$m6 crash=$crash serial=$serialLines"
            $results += "[3/5] QEMU smoke: FAIL (reached=$reached m6=$m6 crash=$crash)"
            $failStep = "3/5"
        }
    } else {
        Log "[3/5] FAIL - ISO absent"
        $results += "[3/5] QEMU smoke: FAIL (ISO absent)"
    }
} else {
    Log ""
    Log "=== [3/5] SKIPPED (langkah sebelumnya gagal - fail-closed) ==="
    $results += "[3/5] QEMU smoke: SKIPPED"
}

# ------------------------------------------------------- [4/5] gerbang memori
if (-not $failStep) {
    Log ""
    Log "=== [4/5] Gerbang regresi memori boot 3 MB (scripts\test_ram_floor_3mb.ps1) ==="
    $gateOut = & powershell -NoProfile -ExecutionPolicy Bypass -File `
        (Join-Path $RepoRoot "scripts\test_ram_floor_3mb.ps1") -TimeoutSec 60 2>&1 | Out-String
    $gateExit = $LASTEXITCODE
    ($gateOut -split "`r?`n") | ForEach-Object { if ($_.Trim()) { Log "  $($_.TrimEnd())" } }
    if ($gateExit -eq 0) {
        Log "[4/5] PASS - janji misi -m 3M dan ratchet dua-duanya boot"
        $results += "[4/5] gerbang memori: PASS (misi 3M + ratchet boot)"
    } elseif ($gateExit -eq 2) {
        Log "[4/5] DEVIASI TERCATAT - ratchet lulus, janji misi -m 3M MERAH."
        Log "        Status merah gerbang Bagian 5 TIDAK diubah menjadi hijau;"
        Log "        deviasi hidup dan wajib muncul di ACCOUNTABILITY_FAULT_LOG."
        $results += "[4/5] gerbang memori: DEVIASI TERCATAT (3M merah, ratchet lulus, exit=2)"
        $deviationNote = "Gerbang Bagian 5 'qemu -m 3M' MERAH pada tree ini (ratchet regresi lulus)."
    } else {
        Log "[4/5] FAIL - regresi footprint memori, boot tidak capai prompt pada ratchet (exit=$gateExit)"
        $results += "[4/5] gerbang memori: FAIL (exit=$gateExit)"
        $failStep = "4/5"
    }
} else {
    Log ""
    Log "=== [4/5] SKIPPED (langkah sebelumnya gagal - fail-closed) ==="
    $results += "[4/5] gerbang memori: SKIPPED"
}

# ------------------------------------------------------------- [5/5] summary
Log ""
Log "=== [5/5] RINGKASAN PIPELINE ==="
foreach ($r in $results) { Log "  $r" }
$elfSizeFinal = if (Test-Path $Elf) { (Get-Item $Elf).Length } else { 0 }
$isoSizeFinal = if (Test-Path $Iso) { (Get-Item $Iso).Length } else { 0 }
Log "  footprint: ELF=$elfSizeFinal B | ISO=$isoSizeFinal B | commit=$commit"
if ($failStep) {
    Log "  KEPUTUSAN: Pipeline MERAH (gagal di langkah $failStep). KEBIJAKAN: TIDAK BOLEH merge ke branch utama sampai langkah tersebut hijau."
    Log "  artefak: $Log"
    exit 1
}
Log "  KEPUTUSAN: Pipeline HIJAU - 5/5 langkah dieksekusi, tidak ada REGRESI. Merge diizinkan sesuai aturan fase (G6: hanya setelah gerbang hijau)."
if ($deviationNote) {
    Log "  DEVIASI TERBUKA: $deviationNote Keliru bila baris HIJAU di atas dibaca sebagai 'semua janji Bagian 5 lulus'."
    Log "  Rujukan deviasi: ACCOUNTABILITY_FAULT_LOG + laporan akhir misi v1.0.2 bagian keputusan otonom."
}
Log "  artefak: $Log"
exit 0
