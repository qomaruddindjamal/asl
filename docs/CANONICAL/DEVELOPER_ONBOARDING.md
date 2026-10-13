# ASL DEVELOPER ONBOARDING & STANDARD OPERATING PROCEDURES
=============================================================
**Target Audience:** Core Kernel Developers, Verification Engineers, Systems Auditors  
**Standard:** DO-178C DAL-A / ARINC 653 IMA  

---

## 1. WORKSPACE PREREQUISITES

The development workstation requires:
1. **Rust Toolchain:** `rustc 1.80+` with targets:
   - `x86_64-pc-windows-msvc` (Host compiler & unit tests)
   - `x86_64-unknown-none` (Bare-metal higher-half launcher target)
2. **ISO Mastering Tools:**
   - `xorriso` (v1.5.0+)
   - `limine` (v8.x binary utilities)
3. **Hardware Emulation / Hypervisor:**
   - Oracle VirtualBox with VM configured as `ASL` (PIIX4 IDE, UART COM1 redirected to file).
4. **Python 3.10+** for governance automation scripts.

---

## 2. REPOSITORY & GOVERNANCE STRUCTURE

- `asl-kernel/`: The unified Cargo workspace containing all 17 microkernel crates.
- `asl-kernel/asl-kernel-bin`: The bare-metal Higher-Half ELF launcher for Limine.
- `asl-kernel/asl-kernel-core`: The pure `#![no_std]` Integrated Modular Avionics microkernel.
- `docs/CANONICAL/`: Authoritative system specifications and architectural models.
- `docs/ROADMAP/`: Formal gate tracking, handoff state, and accountability logs.
- `tools/`: Local governance automation tools (Must remain local-only per `.gitignore`).

---

## 3. CORE COMMAND WORKFLOWS

### A. Running Host Regression Tests (All Crates)
```powershell
cargo test --workspace --target x86_64-pc-windows-msvc
```
*Expected Result:* 26 passed, 0 failed.

### B. Building Bare-Metal Kernel Binary
```powershell
cd C:\asl\asl-kernel
cargo build --package asl-kernel-bin --target x86_64-unknown-none
```
*Output Artifact:* `target\x86_64-unknown-none\debug\asl-kernel` (ELF64 executable).

### C. Rebuilding Bootable Avionics ISO
```powershell
# Copy kernel binary to staging root
Copy-Item C:\asl\asl-kernel\target\x86_64-unknown-none\debug\asl-kernel C:\asl\asl-kernel\target\iso_root\boot\asl-kernel.elf -Force

# Master ISO image
xorriso -as mkisofs -b boot/limine/limine-bios-cd.bin -no-emul-boot -boot-load-size 4 -boot-info-table --efi-boot boot/limine/limine-uefi-cd.bin -efi-boot-part --efi-boot-image --protective-msdos-label /cygdrive/c/asl/asl-kernel/target/iso_root -o /cygdrive/c/asl/asl-kernel/target/asl-core-kernel.iso

# Install Limine BIOS stages
C:\asl\asl-kernel\target\limine_v8_binary\limine.exe bios-install C:\asl\asl-kernel\target\asl-core-kernel.iso
```

### D. Running Verification on VirtualBox VM "ASL"
```powershell
& 'C:\Program Files\Oracle\VirtualBox\VBoxManage.exe' startvm 'ASL' --type headless
```
*Verification Check:* Inspect serial log (`asl_vm_serial.log`) for milestones M1..M3 and monitor VM state (`State: running`).

### E. Workspace Hygiene Audit
```powershell
python C:\asl\tools\Sub_Assist.py --hygiene
```
*Requirement:* Must return `ROOT WORKSPACE BERSIH 100%`.
