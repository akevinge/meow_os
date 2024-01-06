// This module contains utilities for disk operations.

use core::arch::asm;

// check_lba_extension_support checks if the BIOS supports LBA extensions.
// Carry flag is set if LBA extensions are not supported.
// Reference: https://wiki.osdev.org/Disk_access_using_the_BIOS_(INT_13h)#Converting_LBA_to_CHS.
pub fn check_lba_extension_support() -> bool {
    let mut supported: u8;
    unsafe {
        asm!(
            "pushf",            // Push EFLAG register to stack.
                                // This is so we can restore the carry flag later.
            "push bx",
            "push dx",
            "clc",              // Clear carry flag.
            "mov ah, 0x41",     // Ritual to check extensions.
            "mov bx, 0x55aa",
            "mov dl, 0x80",
            "int 0x13",
            "setc al",          // Set al to carry flag (1 if set, else 0).
            "popf",
            "pop dx",
            "pop bx",
            out("al") supported,
        );
    }
    supported == 0 // Carry flag is NOT set if LBA extensions are supported.
}
