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

# CRC32 (IEEE 802.3, poly 0xEDB88320, init/xor 0xFFFFFFFF) as used by the GPT headers.
# All arithmetic stays in Int64: Windows PowerShell's -shr/-bxor on a UInt32 whose high bit
# is set first narrows it to a negative Int32, which then either shifts wrongly or fails to cast.
function Get-Crc32([byte[]]$Data) {
    # Decimal on purpose: Windows PowerShell 5.1 parses a hex literal above 0x7FFFFFFF as a
    # negative Int32, so 0xFFFFFFFF -bxor/-band would sign-extend into Int64 and corrupt the CRC.
    $mask32 = [long]4294967295
    $poly = [long]3988292384
    if (-not $script:Crc32Table) {
        $table = New-Object 'System.Int64[]' 256
        for ($i = 0; $i -lt 256; $i++) {
            $c = [long]$i
            for ($k = 0; $k -lt 8; $k++) {
                if ($c -band 1) { $c = ($poly -bxor ($c -shr 1)) -band $mask32 }
                else { $c = ($c -shr 1) -band $mask32 }
            }
            $table[$i] = $c
        }
        $script:Crc32Table = $table
    }
    $crc = [long]4294967295
    foreach ($b in $Data) {
        $crc = ($script:Crc32Table[($crc -bxor $b) -band 0xFF] -bxor ($crc -shr 8)) -band $mask32
    }
    return ($crc -bxor $mask32) -band $mask32
}

function Get-Slice([byte[]]$Buffer, [int]$Offset, [int]$Length) {
    $out = New-Object 'System.Byte[]' $Length
    [Array]::Copy($Buffer, $Offset, $out, 0, $Length)
    return ,$out
}

function Set-UInt32Le([byte[]]$Buffer, [int]$Offset, [long]$Value) {
    $v = $Value -band [long]4294967295
    $Buffer[$Offset]     = [System.Byte]($v -band 0xFF)
    $Buffer[$Offset + 1] = [System.Byte](($v -shr 8) -band 0xFF)
    $Buffer[$Offset + 2] = [System.Byte](($v -shr 16) -band 0xFF)
    $Buffer[$Offset + 3] = [System.Byte](($v -shr 24) -band 0xFF)
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
        # K-5: the host utility lives OUTSIDE target/, because `cargo clean` deletes the
        # whole of target/ and had taken the owner-installed limine.exe with it (fault log #50/#54).
        $limineExe = Join-Path $RepoRoot "tools\limine\v8.7.0\limine.exe"
        if (-not (Test-Path $limineExe)) {
            Write-Error "ERROR: Limine binary utility not found at '$limineExe'. See docs/TOOLING/LIMINE-PROVISIONING.md."
            exit 1
        }

        # 3. Copy kernel ELF into iso_root
        $kernelElf = Join-Path $Workspace "target\$TargetArch\release\asl-kernel"
        $isoBootDir = Join-Path $Workspace "target\iso_root\boot"
        if (-not (Test-Path $isoBootDir)) {
            New-Item -ItemType Directory -Path $isoBootDir -Force | Out-Null
        }
        Copy-Item $kernelElf (Join-Path $isoBootDir "asl-kernel.elf") -Force

        # 4. Master ISO via xorriso (PHASE-7 T1: pin image dates)
        #    Unpinned, xorriso 1.5.2 stamps the generation time in the volume descriptors
        #    and copies source mtimes into Rock Ridge, so every rebuild changes the ISO
        #    SHA256 even though the enclosed ELF is byte-identical (AUD-03). SOURCE_DATE_EPOCH
        #    is honoured when set; otherwise the baked constant 1767225600
        #    (2026-01-01T00:00:00Z) is used so two clean builds on one host match.
        $isoOutput = Join-Path $Workspace "target\asl-core-kernel.iso"
        $isoRootCygwin = "/cygdrive/c/asl/asl-kernel/target/iso_root"
        $isoOutputCygwin = "/cygdrive/c/asl/asl-kernel/target/asl-core-kernel.iso"

        $epochDefault = 1767225600
        if ($env:SOURCE_DATE_EPOCH -match '^\d{1,10}$') {
            $epoch = [long]$env:SOURCE_DATE_EPOCH
            $epochFrom = "SOURCE_DATE_EPOCH=$env:SOURCE_DATE_EPOCH"
        } else {
            $epoch = [long]$epochDefault
            $epochFrom = "baked default $epochDefault"
        }
        $pinnedDate = [DateTimeOffset]::FromUnixTimeSeconds($epoch).ToUniversalTime().ToString("yyyyMMddHHmmss") + "00"
        Write-Host "==> Pinning ISO image dates to $pinnedDate (UTC, from $epochFrom)" -ForegroundColor Cyan

        & $xorriso -as mkisofs --modification-date=$pinnedDate --set_all_file_dates $pinnedDate -b boot/limine/limine-bios-cd.bin -no-emul-boot -boot-load-size 4 -boot-info-table --efi-boot boot/limine/limine-uefi-cd.bin -efi-boot-part --efi-boot-image --protective-msdos-label $isoRootCygwin -o $isoOutputCygwin
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

        # 6. Pin the GPT GUIDs that limine bios-install randomises (PHASE-7 T1)
        #    limine writes a random Disk GUID into both GPT headers and derives each
        #    partition entry's Unique GUID from it, together with the header and partition
        #    array CRC32s. Measured in PHASE-7 T1: two clean builds on this host differed by
        #    exactly 144 bytes, all of them inside those fields. The GUIDs carry no boot
        #    meaning (stage 2 is located by LBA, not by GUID), so replacing them with a value
        #    derived from the pinned epoch and recomputing both CRC32s per the UEFI GPT
        #    layout keeps the image bootable and byte-stable.
        $isoBytes = [System.IO.File]::ReadAllBytes($isoOutput)
        $sha = [System.Security.Cryptography.SHA256]::Create()
        $digest = $sha.ComputeHash([System.Text.Encoding]::ASCII.GetBytes("ASL-ISO-GPT:$epoch"))
        $sha.Dispose()
        $diskGuid = New-Object 'System.Byte[]' 16
        [Array]::Copy($digest, 0, $diskGuid, 0, 16)
        $diskGuid[6] = [System.Byte]((($diskGuid[6] -band 0x0F) -bor 0x40))
        $diskGuid[8] = [System.Byte]((($diskGuid[8] -band 0x3F) -bor 0x80))
        $diskGuid[9] = [System.Byte]0

        $headerOffsets = @()
        # GPT headers are 512-byte aligned and start with the "EFI PART" signature.
        for ($off = 0; ($off + 92) -le $isoBytes.Length; $off += 512) {
            if ($isoBytes[$off] -ne 0x45) { continue }
            if ($isoBytes[$off + 1] -ne 0x46) { continue }
            if ($isoBytes[$off + 2] -ne 0x49 -or $isoBytes[$off + 3] -ne 0x20) { continue }
            if ($isoBytes[$off + 4] -ne 0x50 -or $isoBytes[$off + 5] -ne 0x41) { continue }
            if ($isoBytes[$off + 6] -ne 0x52 -or $isoBytes[$off + 7] -ne 0x54) { continue }
            if ([BitConverter]::ToUInt32($isoBytes, $off + 8) -ne 0x00010000) { continue }
            $headerOffsets += $off
        }
        if ($headerOffsets.Count -eq 0) {
            Write-Error "GPT header not found after bios-install; refusing to emit an unpinned image."
            exit 1
        }

        foreach ($off in $headerOffsets) {
            [Array]::Copy($diskGuid, 0, $isoBytes, $off + 56, 16)
            $entryLba   = [BitConverter]::ToUInt64($isoBytes, $off + 72)
            $numEntries = [BitConverter]::ToUInt32($isoBytes, $off + 80)
            $entrySize  = [BitConverter]::ToUInt32($isoBytes, $off + 84)
            $arrayOff   = [int]($entryLba * 512)
            $arrayLen   = [int]($numEntries * $entrySize)
            if (($arrayOff + $arrayLen) -gt $isoBytes.Length) {
                Write-Error "GPT partition array lies outside the image (offset $arrayOff length $arrayLen)."
                exit 1
            }
            for ($i = 0; $i -lt $numEntries; $i++) {
                $slot = $arrayOff + ($i * $entrySize)
                if (((Get-Slice $isoBytes $slot 16) | Measure-Object -Sum).Sum -eq 0) { continue }
                [Array]::Copy($diskGuid, 0, $isoBytes, $slot + 16, 16)
                $isoBytes[$slot + 25] = [System.Byte](($i + 1) -band 0xFF)
            }
            Set-UInt32Le $isoBytes ($off + 88) (Get-Crc32 (Get-Slice $isoBytes $arrayOff $arrayLen))
            Set-UInt32Le $isoBytes ($off + 16) 0
            $headSize = [int][BitConverter]::ToUInt32($isoBytes, $off + 12)
            Set-UInt32Le $isoBytes ($off + 16) (Get-Crc32 (Get-Slice $isoBytes $off $headSize))
        }
        [System.IO.File]::WriteAllBytes($isoOutput, $isoBytes)
        Write-Host "==> GPT Disk GUID pinned to $(([System.Guid]::new($diskGuid)).ToString()) (derived from epoch $epoch)" -ForegroundColor Cyan

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
