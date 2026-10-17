# =============================================================================
# scripts/build_kernel.ps1 - Automated Build, Test, ISO, and Run Orchestrator
# =============================================================================
param(
    [Parameter(Mandatory=$true)]
    [ValidateSet("build", "build-aarch64", "test", "iso", "run-vm", "run-qemu", "run-qemu-aarch64", "clean")]
    [string]$Action
)

$ErrorActionPreference = "Stop"
$ScriptRoot = Split-Path -Parent $MyInvocation.MyCommand.Path
$RepoRoot = Split-Path -Parent $ScriptRoot
$Workspace = Join-Path $RepoRoot "asl-kernel"
$TargetArch = "x86_64-unknown-none"
$HostTarget = "x86_64-pc-windows-msvc"

function Assert-Tool($Name, $Command) {
    $found = Get-Command $Command -ErrorAction SilentlyContinue
    if (-not $found) {
        Write-Error "ERROR: Required tool '$Name' ($Command) was not found on PATH or installed location."
        exit 1
    }
    return $found.Source
}

switch ($Action) {
    "build" {
        Write-Host "==> Compiling ASL Core Kernel ($TargetArch) in release mode..." -ForegroundColor Cyan
        Set-Location $Workspace
        & cargo build --package asl-kernel-bin --target $TargetArch --release
        if ($LASTEXITCODE -ne 0) {
            Write-Error "Cargo build failed with exit code $LASTEXITCODE"
            exit $LASTEXITCODE
        }
        Write-Host "[OK] Kernel ELF successfully compiled." -ForegroundColor Green
    }

    "test" {
        Write-Host "==> Running host unit test suite ($HostTarget)..." -ForegroundColor Cyan
        Set-Location $Workspace
        & cargo test --workspace --target $HostTarget --lib
        if ($LASTEXITCODE -ne 0) {
            Write-Error "Cargo test failed with exit code $LASTEXITCODE"
            exit $LASTEXITCODE
        }
        Write-Host "[OK] All host unit tests passed." -ForegroundColor Green
    }

    "iso" {
        Write-Host "==> Building bootable hybrid ISO..." -ForegroundColor Cyan
        Set-Location $Workspace

        # 1. Build release ELF first
        & cargo build --package asl-kernel-bin --target $TargetArch --release
        if ($LASTEXITCODE -ne 0) { exit $LASTEXITCODE }

        # 2. Check xorriso and limine tools
        $xorriso = Assert-Tool "xorriso" "xorriso.exe"
        $limineExe = Join-Path $Workspace "target\limine_v8_binary\limine.exe"
        if (-not (Test-Path $limineExe)) {
            Write-Error "ERROR: Limine binary utility not found at '$limineExe'."
            exit 1
        }

        # 3. Copy kernel ELF into iso_root
        $kernelElf = Join-Path $Workspace "target\$TargetArch\release\asl-kernel"
        $isoBootDir = Join-Path $Workspace "target\iso_root\boot"
        if (-not (Test-Path $isoBootDir)) {
            New-Item -ItemType Directory -Path $isoBootDir -Force | Out-Null
        }
        Copy-Item $kernelElf (Join-Path $isoBootDir "asl-kernel.elf") -Force

        # 4. Master ISO via xorriso
        $isoOutput = Join-Path $Workspace "target\asl-core-kernel.iso"
        $isoRootCygwin = "/cygdrive/c/asl/asl-kernel/target/iso_root"
        $isoOutputCygwin = "/cygdrive/c/asl/asl-kernel/target/asl-core-kernel.iso"

        & $xorriso -as mkisofs -b boot/limine/limine-bios-cd.bin -no-emul-boot -boot-load-size 4 -boot-info-table --efi-boot boot/limine/limine-uefi-cd.bin -efi-boot-part --efi-boot-image --protective-msdos-label $isoRootCygwin -o $isoOutputCygwin
        if ($LASTEXITCODE -ne 0) {
            Write-Error "xorriso ISO mastering failed with exit code $LASTEXITCODE"
            exit $LASTEXITCODE
        }

        # 5. Install BIOS boot stages
        & $limineExe bios-install $isoOutput
        if ($LASTEXITCODE -ne 0) {
            Write-Error "Limine bios-install failed with exit code $LASTEXITCODE"
            exit $LASTEXITCODE
        }

        $hash = (Get-FileHash -Algorithm SHA256 $isoOutput).Hash
        $size = (Get-Item $isoOutput).Length
        Write-Host "[OK] ISO successfully mastered: $isoOutput ($size bytes, SHA256: $hash)" -ForegroundColor Green
    }

    "run-vm" {
        Write-Host "==> Launching VirtualBox VM 'ASL'..." -ForegroundColor Cyan
        $vboxManage = "C:\Program Files\Oracle\VirtualBox\VBoxManage.exe"
        if (-not (Test-Path $vboxManage)) {
            Write-Error "VirtualBox VBoxManage.exe not found at standard path."
            exit 1
        }
        & $vboxManage startvm "ASL"
        if ($LASTEXITCODE -ne 0) {
            Write-Error "VirtualBox startvm failed with exit code $LASTEXITCODE"
            exit $LASTEXITCODE
        }
        Write-Host "[OK] VirtualBox VM 'ASL' started." -ForegroundColor Green
    }

    "run-qemu" {
        Write-Host "==> Launching QEMU headless test..." -ForegroundColor Cyan
        $qemu = "C:\Program Files\qemu\qemu-system-x86_64.exe"
        if (-not (Test-Path $qemu)) {
            Write-Error "QEMU not found at '$qemu'."
            exit 1
        }
        $iso = Join-Path $Workspace "target\asl-core-kernel.iso"
        if (-not (Test-Path $iso)) {
            Write-Error "ISO not found at '$iso'. Please run 'make iso' first."
            exit 1
        }
        & $qemu -cdrom $iso -serial stdio -display none -m 64M
    }

    "clean" {
        Write-Host "==> Cleaning build artifacts..." -ForegroundColor Cyan
        Set-Location $Workspace
        & cargo clean
        Write-Host "[OK] Cargo target directory cleaned." -ForegroundColor Green
    }

    "build-aarch64" {
        Write-Host "==> Compiling ASL Core Kernel (aarch64-unknown-none-softfloat) in release mode..." -ForegroundColor Cyan
        Set-Location $Workspace
        & cargo build --package asl-kernel-bin --target aarch64-unknown-none-softfloat --release
        if ($LASTEXITCODE -ne 0) {
            Write-Error "Cargo build failed with exit code $LASTEXITCODE"
            exit $LASTEXITCODE
        }
        Write-Host "[OK] AArch64 Kernel ELF successfully compiled." -ForegroundColor Green
    }

    "run-qemu-aarch64" {
        Write-Host "==> Launching AArch64 QEMU bare-metal virt test..." -ForegroundColor Cyan
        $qemu = "C:\Program Files\qemu\qemu-system-aarch64.exe"
        if (-not (Test-Path $qemu)) {
            Write-Error "QEMU AArch64 not found at '$qemu'."
            exit 1
        }
        $kernel = Join-Path $Workspace "target\aarch64-unknown-none-softfloat\release\asl-kernel"
        if (-not (Test-Path $kernel)) {
            Write-Error "AArch64 kernel ELF not found at '$kernel'. Please run 'make build-aarch64' first."
            exit 1
        }
        & $qemu -M virt -cpu cortex-a57 -m 64M -nographic -kernel $kernel -serial stdio
    }
}
