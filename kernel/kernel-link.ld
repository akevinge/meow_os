ENTRY(_start)

SECTIONS {
    /*  Ensure that _start function is at the beginning of
        the kernel.
    */
    .start : { *(.start .start.*)}

    .text : { *(.text .text.*) }
    .bss : { *(.bss .bss.*) }
    .data : { *(.data .data.*) }
    .rodata : { *(.rodata .rodata.*) }
    .got : { *(.got .got.*) }
}