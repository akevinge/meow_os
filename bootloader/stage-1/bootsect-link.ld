/* Based on https://github.com/rust-osdev/bootloader/blob/main/bios/boot_sector/boot-sector-link.ld */


ENTRY(_start)

SECTIONS {
    . = 0x500;
    _stack_start = .;
    . = 0x7c00;
    _stack_end = .;

    _mbr_start = .;
    /*  Linker needs to place .boot_sect at start because 
        bootloader will load and jmp to 0x7C00.
        Without this, the .boot_sec will be placed at the end of the .text segment.
    */
    .boot :
    {
        *(.boot .boot.*)
    }
    .text :
    {
        *(.text .text.*)
    }
    .rodata :
    {
        *(.rodata .rodata.*)
    }
    .data :
    {
        *(.rodata .rodata.*)
        *(.data .data.*)
        *(.got .got.*)
    }
    _mbr_end = .;

    /*  Partition table is 48 bytes (16 bytes/entry, 4 entries). 
        For structure, see: https://en.wikipedia.org/wiki/Master_boot_record.
    */
    . = 0x7C00 + 446;
    _partition_table = .;
    .partition_table :
    {
        FILL(0)
        . = 0x7C00 + 510;
    }

    /* magic number for bootable disk */
    .magic_number : { SHORT(0xaa55) }

    _second_stage_start = .;
}