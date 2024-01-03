// This module provides utilities for printing to screen.
// This is so we can sanity check when we're booting.
// Since we are in real mode, we have to use BIOS interrupts to print to the screen.
use core::arch::asm;

#[inline(never)]
#[no_mangle]
pub fn print(s: &str) {
    for c in s.bytes() {
        print_char(c);
    }
}

#[inline(never)]
#[no_mangle]
pub fn print_char(c: u8) {
    unsafe {
        asm!(
            "pusha",        // save registers
            "mov ah, 0x0e", // tty mode
            "int 0x10",     // call BIOS interrupt
            "popa",         // restore registers
            in("al") c      // char to print
        );
    }
}
