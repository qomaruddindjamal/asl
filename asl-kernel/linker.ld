/* Linker Script Higher-Half x86_64 untuk ASL Core Kernel */
OUTPUT_FORMAT(elf64-x86-64)
OUTPUT_ARCH(i386:x86-64)

ENTRY(_start)

PHDRS
{
    limine_requests PT_LOAD FLAGS(6); /* Read + Write (Mandatory for Limine responses) */
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

    /* Carve-out region: Secondary failover engine + datanya (isolasi fisik, G2 audit).
       WAJIB sebelum .bss: pola .bss.* akan menyerap .bss.asl_secondary jika lebih dulu. */
    . = ALIGN(4096);

    .asl_secondary : ALIGN(4096) {
        __asl_secondary_start = .;
        KEEP(*(.asl_secondary .asl_secondary.* .bss.asl_secondary))
        . = ALIGN(4096);
        __asl_secondary_end = .;
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
