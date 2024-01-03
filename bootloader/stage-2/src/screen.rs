use core::arch::asm;

pub fn print(s: &str) {
    for c in s.bytes() {
        print_char(c);
    }
}

fn print_char(c: u8) {
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
