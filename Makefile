# =============================================================================
# ASL Core Kernel Root Makefile
# Target: Windows (PowerShell / GNU Make) & Cross-Platform
# =============================================================================

.PHONY: all build test iso run-vm run-qemu clean help

# Configuration variables
WORKSPACE_DIR := asl-kernel
TARGET_ARCH := x86_64-unknown-none
HOST_TARGET := x86_64-pc-windows-msvc
ISO_PATH := $(WORKSPACE_DIR)/target/asl-core-kernel.iso

all: build

help:
	@echo "ASL Core Kernel Build System"
	@echo "Targets:"
	@echo "  make build    - Compile bare-metal kernel binary (release mode)"
	@echo "  make test     - Run full host unit test suite"
	@echo "  make iso      - Package bootable hybrid El Torito ISO"
	@echo "  make run-vm   - Launch existing Oracle VirtualBox VM 'ASL'"
	@echo "  make run-qemu - Launch kernel in headless QEMU with serial console"
	@echo "  make clean    - Remove build artifacts"

build:
	@powershell -ExecutionPolicy Bypass -File ./scripts/build_kernel.ps1 -Action build

test:
	@powershell -ExecutionPolicy Bypass -File ./scripts/build_kernel.ps1 -Action test

iso:
	@powershell -ExecutionPolicy Bypass -File ./scripts/build_kernel.ps1 -Action iso

run-vm:
	@powershell -ExecutionPolicy Bypass -File ./scripts/build_kernel.ps1 -Action run-vm

run-qemu:
	@powershell -ExecutionPolicy Bypass -File ./scripts/build_kernel.ps1 -Action run-qemu

clean:
	@powershell -ExecutionPolicy Bypass -File ./scripts/build_kernel.ps1 -Action clean
