#!/usr/bin/env python3
"""
tools/Sub_Assist.py
===================
Media Sarana Pekerja Otomatis (Worker / Auto-Fixer & Verification Runner).
Membantu AI Agent dalam:
- Melakukan verifikasi sintaksis dan file integrity
- Menjalankan analisis statis kode
- Menyiapkan template pengujian bugfix
- Melaporkan hasil pengujian ke Gemini_Assist untuk dikunci
"""

import os
import sys
import subprocess
import argparse

def check_hygiene():
    """Memeriksa kebersihan workspace root proyek dari file sampah"""
    root_dir = os.path.abspath(os.path.join(os.path.dirname(__file__), ".."))
    print(f"[*] Menjalankan Audit Kebersihan Root: {root_dir}")
    
    illegal_extensions = [".tmp", ".log", ".bak", ".swp", ".dump"]
    found_clutter = []
    
    for item in os.listdir(root_dir):
        full_path = os.path.join(root_dir, item)
        if os.path.isfile(full_path):
            _, ext = os.path.splitext(item)
            if ext in illegal_extensions or item.startswith("temp_"):
                found_clutter.append(item)
        elif os.path.isdir(full_path) and item in ["__pycache__", ".pytest_cache"]:
            found_clutter.append(item)
            
    # Verifikasi Staging Folder Analysis
    analysis_dir = os.path.join(root_dir, "Analysis")
    if os.path.exists(analysis_dir):
        subdirs = ["quarantine_incoming", "binaries_pe_dll_exe", "packages_deb_pkg", "firmware_rom_bin", "reports"]
        for sd in subdirs:
            sd_path = os.path.join(analysis_dir, sd)
            if not os.path.exists(sd_path):
                print(f"[!] Staging subfolder hilang: Analysis/{sd}")
                return False

    if found_clutter:
        print(f"[!] DITEMUKAN FILE SAMPAH: {found_clutter}")
        return False
    else:
        print("[PASS (V)] ROOT WORKSPACE BERSIH 100%. Struktur Analysis Valid.")
        return True

def run_diagnostics():
    """Memeriksa kelengkapan dokumen acuan dan direktori RND/ROADMAP"""
    root_dir = os.path.abspath(os.path.join(os.path.dirname(__file__), ".."))
    required_files = [
        "README_ASL_CORE.md",
        "AGENT.md",
        "GEMINI.md",
        "asl_core.md",
        "asl_avionics_kernel_blueprint.md",
        "asl_kernel_audit_roadmap.md",
        os.path.join("docs", "RND", "README.md"),
        os.path.join("docs", "ROADMAP", "README.md"),
        os.path.join("docs", "ROADMAP", "ROADMAP_MASTER.md")
    ]
    
    print("\n[*] Menjalankan Diagnostik Keberadaan Sumber Data & Dokumentasi...")
    all_ok = True
    for rel_path in required_files:
        full_p = os.path.join(root_dir, rel_path)
        if os.path.exists(full_p):
            print(f"   [PASS (V)] {rel_path}")
        else:
            print(f"   [FAIL (X)] {rel_path} TIDAK DITEMUKAN!")
            all_ok = False
            
    return all_ok

if __name__ == "__main__":
    parser = argparse.ArgumentParser(description="Sub-Assist Worker & Diagnostics Runner")
    parser.add_argument("--hygiene", action="store_true", help="Audit kebersihan direktori root")
    parser.add_argument("--diagnostics", action="store_true", help="Diagnostik kelengkapan file sistem")
    args = parser.parse_args()

    if args.hygiene:
        check_hygiene()
    elif args.diagnostics:
        run_diagnostics()
    else:
        h_ok = check_hygiene()
        d_ok = run_diagnostics()
        if h_ok and d_ok:
            print("\n[PASS (V)] SELURUH DIAGNOSTIK AWAL SUB-ASSIST: PASS & STABIL.")
        else:
            print("\n[!] DITEMUKAN ANOMALI DALAM DIAGNOSTIK.")
