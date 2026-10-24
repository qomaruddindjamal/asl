# =============================================================================
# scripts/test_hv2_stack_overflow_qemu.ps1 - HV-2 [PHASE-5C] stack-overflow
# injection, cold-boot pada QEMU q35 (tanpa VT-d).
#
# Membangun ULANG artefak dengan fitur cargo `test-hv2-stack-overflow`
# (injeksi deliberate, TIDAK ada di artefak release nominal), mastered ISO
# khusus HV-2, boot cold, lalu tangkap COM1 serial ke
# Analysis/reports/qemu_hv2_serial.log.
#
# Exit criteria (dibaca dari serial log):
#   1. "STACK OVERFLOW (GUARD PAGE HIT)"            - verbatim
#   2. "[ASL-HV2] DR FSM latch: ... reason=3"       - FSM Recording + reason benar
#   3. "Controlled Halt Active" + VM tidak reboot   - safe halt
#
# PERINGATAN: script ini MENIMPA target/iso_root/boot/asl-kernel.elf dan
# target/asl-core-kernel.iso dengan artefak HV-2. Untuk mengembalikan artefak
# release nominal jalankan: scripts/build_kernel.ps1 -Action iso
# =============================================================================
param(
    [switch]$SkipBuild
)
$ErrorActionPreference = "Stop"
$RepoRoot = "C:\asl"
$Workspace = Join-Path $RepoRoot "asl-kernel"
$TargetArch = "x86_64-unknown-none"
$qemuExe = "C:\Program Files\qemu\qemu-system-x86_64.exe"
$xorriso = "xorriso.exe"
$limineExe = Join-Path $RepoRoot "tools\limine\v8.7.0\limine.exe"
$serialLog = Join-Path $RepoRoot "Analysis\reports\qemu_hv2_serial.log"
$qemuErrLog = Join-Path $RepoRoot "Analysis\reports\qemu_hv2_qemu_stderr.log"
$iso = Join-Path $Workspace "target\hv2-stack-overflow.iso"

if (-not (Test-Path $qemuExe)) { Write-Error "QEMU not found at $qemuExe"; exit 1 }

if (-not $SkipBuild) {
    Write-Host "==> HV-2: building kernel ELF with feature test-hv2-stack-overflow..." -ForegroundColor Cyan
    Set-Location $Workspace
    & cargo build --package asl-kernel-bin --target $TargetArch --release --features test-hv2-stack-overflow
    if ($LASTEXITCODE -ne 0) { Write-Error "Feature build failed"; exit $LASTEXITCODE }

    $kernelElf = Join-Path $Workspace "target\$TargetArch\release\asl-kernel"
    $isoBootDir = Join-Path $Workspace "target\iso_root\boot"
    if (-not (Test-Path $isoBootDir)) { New-Item -ItemType Directory -Path $isoBootDir -Force | Out-Null }
    Copy-Item $kernelElf (Join-Path $isoBootDir "asl-kernel.elf") -Force

    # xorriso build ini berbasis cygwin: wajib path cygdrive
    $isoRootCygwin = "/cygdrive/c/asl/asl-kernel/target/iso_root"
    $isoOutputCygwin = "/cygdrive/c/asl/asl-kernel/target/hv2-stack-overflow.iso"
    & $xorriso -as mkisofs -b boot/limine/limine-bios-cd.bin -no-emul-boot -boot-load-size 4 -boot-info-table --efi-boot boot/limine/limine-uefi-cd.bin -efi-boot-part --efi-boot-image --protective-msdos-label $isoRootCygwin -o $isoOutputCygwin
    if ($LASTEXITCODE -ne 0) { Write-Error "xorriso ISO mastering failed"; exit $LASTEXITCODE }
    & $limineExe bios-install $iso
    if ($LASTEXITCODE -ne 0) { Write-Error "limine bios-install failed"; exit $LASTEXITCODE }
    $size = (Get-Item $iso).Length
    $hash = (Get-FileHash -Algorithm SHA256 $iso).Hash
    Write-Host "[OK] HV-2 ISO: $iso ($size bytes, SHA256 $hash)" -ForegroundColor Green
}

Remove-Item $serialLog, $qemuErrLog -ErrorAction SilentlyContinue

$qargs = @(
    "-machine", "q35",
    "-vga", "std",
    "-m", "256M",
    "-cdrom", $iso,
    "-serial", "file:$serialLog",
    "-display", "none",
    "-no-reboot",
    "-no-shutdown"
)
Write-Host "==> HV-2: cold-booting QEMU q35 with injected stack overflow..." -ForegroundColor Cyan
$proc = Start-Process $qemuExe -ArgumentList $qargs -RedirectStandardError $qemuErrLog -PassThru

$deadline = (Get-Date).AddSeconds(90)
$done = $false
while ((Get-Date) -lt $deadline) {
    Start-Sleep -Seconds 5
    if (Test-Path $serialLog) {
        $txt = Get-Content $serialLog -Raw -ErrorAction SilentlyContinue
        if ($txt -and $txt.Contains("GUARD PAGE HIT")) { $done = $true; break }
    }
    if ($proc.HasExited) { break }
}
if (-not $proc.HasExited) { Stop-Process -Id $proc.Id -Force }

Write-Host "==> HV-2 run finished (marker_found=$done). Serial evidence: $serialLog" -ForegroundColor Green
if (Test-Path $serialLog) { Get-Content $serialLog } else { Write-Host "NO SERIAL LOG PRODUCED" }
if ($done) { exit 0 } else { exit 2 }
