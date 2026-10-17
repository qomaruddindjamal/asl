# =============================================================================
# ASL Core Kernel Root Makefile
# Target: Windows (PowerShell / GNU Make) & Cross-Platform
# =============================================================================

.PHONY: all build build-aarch64 test iso run-vm run-qemu run-qemu-aarch64 clean help

# Configuration variables
WORKSPACE_DIR := asl-kernel
TARGET_ARCH := x86_64-unknown-none
HOST_TARGET := x86_64-pc-windows-msvc
ISO_PATH := $(WORKSPACE_DIR)/target/asl-core-kernel.iso

all: build

help:
	@echo "ASL Core Kernel Build System"
	@echo "Targets:"
	@echo "  make build             - Compile bare-metal x86_64 kernel binary (release mode)"
	@echo "  make build-aarch64     - Compile bare-metal AArch64 kernel binary (release mode)"
	@echo "  make test              - Run full host unit test suite"
	@echo "  make iso               - Package bootable hybrid El Torito ISO (x86_64)"
	@echo "  make run-vm            - Launch existing Oracle VirtualBox VM 'ASL'"
	@echo "  make run-qemu          - Launch x86_64 kernel in headless QEMU with serial console"
	@echo "  make run-qemu-aarch64  - Launch AArch64 kernel in QEMU virt platform"
	@echo "  make clean             - Remove build artifacts"

build:
	@powershell -ExecutionPolicy Bypass -File ./scripts/build_kernel.ps1 -Action build

build-aarch64:
	@powershell -ExecutionPolicy Bypass -File ./scripts/build_kernel.ps1 -Action build-aarch64

test:
	@powershell -ExecutionPolicy Bypass -File ./scripts/build_kernel.ps1 -Action test

iso:
	@powershell -ExecutionPolicy Bypass -File ./scripts/build_kernel.ps1 -Action iso

run-vm:
	@powershell -ExecutionPolicy Bypass -File ./scripts/build_kernel.ps1 -Action run-vm

run-qemu:
	@powershell -ExecutionPolicy Bypass -File ./scripts/build_kernel.ps1 -Action run-qemu

run-qemu-aarch64:
	@powershell -ExecutionPolicy Bypass -File ./scripts/build_kernel.ps1 -Action run-qemu-aarch64

clean:
	@powershell -ExecutionPolicy Bypass -File ./scripts/build_kernel.ps1 -Action clean
