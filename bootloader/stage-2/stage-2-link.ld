ENTRY(_start)

SECTIONS {
    . = 0x7E00;

    /*  Ensure that _start function is at the beginning of
        the second stage.
    */
    .start : { *(.start .start.*)}

    .text : { *(.text .text.*) }
    .bss : { *(.bss .bss.*) }
    .data : { *(.data .data.*) }
    .rodata : { *(.rodata .rodata.*) }
    .got : { *(.got .got.*) }
}