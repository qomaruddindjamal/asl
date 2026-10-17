/* Linker Script Bare-Metal AArch64 untuk ASL Core Kernel (QEMU virt) */
OUTPUT_FORMAT(elf64-littleaarch64)
OUTPUT_ARCH(aarch64)

ENTRY(_start)

SECTIONS
{
    /* QEMU virt platform loads kernel starting at 0x40080000 (RAM starts at 0x40000000) */
    . = 0x40080000;

    .text : {
        *(.text.entry)
        *(.text .text.*)
    }

    . = ALIGN(4096);

    .rodata : {
        *(.rodata .rodata.*)
    }

    . = ALIGN(4096);

    .data : {
        *(.data .data.*)
    }

    .bss : {
        *(.bss .bss.*)
        *(COMMON)
    }

    . = ALIGN(4096);

    /DISCARD/ : {
        *(.eh_frame*)
        *(.note .note.*)
    }
}
