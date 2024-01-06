use core::arch::asm;

pub fn print(s: &str) {
    for c in s.bytes() {
        print_char(c);
    }
}

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

pub fn print_hex(x: u32) {
    let mut buf = [0u8; 8];
    let mut i = 0;
    let mut x = x;
    while x > 0 {
        let rem = x % 16;
        buf[i] = match rem {
            0..=9 => b'0' + rem as u8,
            _ => b'A' + (rem - 10) as u8,
        };
        x /= 16;
        i += 1;
    }
    print("0x");
    for c in buf[..i].iter().rev() {
        print_char(*c);
    }
}
