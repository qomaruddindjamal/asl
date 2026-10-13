/* Linker Script Higher-Half x86_64 untuk ASL Core Kernel */
OUTPUT_FORMAT(elf64-x86-64)
OUTPUT_ARCH(i386:x86-64)

ENTRY(_start)

PHDRS
{
    limine_requests PT_LOAD FLAGS(4); /* Read-only */
    text            PT_LOAD FLAGS(5); /* Read + Execute */
    rodata          PT_LOAD FLAGS(4); /* Read-only */
    data            PT_LOAD FLAGS(6); /* Read + Write */
}

SECTIONS
{
    /* Higher-Half Base Virtual Address (0xFFFFFFFF80000000 + 0x100000) */
    . = 0xFFFFFFFF80000000 + 0x100000;

    .limine_requests : {
        KEEP(*(.limine_requests_start))
        KEEP(*(.limine_requests))
        KEEP(*(.limine_requests_end))
    } :limine_requests

    . = ALIGN(4096);

    .text : {
        *(.text.entry)
        *(.text .text.*)
    } :text

    . = ALIGN(4096);

    .rodata : {
        *(.rodata .rodata.*)
    } :rodata

    . = ALIGN(4096);

    .data : {
        *(.data .data.*)
    } :data

    .bss : {
        *(.bss .bss.*)
        *(COMMON)
    } :data

    /DISCARD/ : {
        *(.eh_frame*)
        *(.note .note.*)
    }
}
