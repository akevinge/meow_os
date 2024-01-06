use crate::screen::{print, print_char, print_hex};

use core::arch::asm;

#[inline(never)]
#[no_mangle]
pub fn err_fatal(failbit: u8) -> ! {
    static ERR_MSG: &str = "Fatal error: ";
    print(ERR_MSG);
    print_char(failbit);
    print_char(b'[');
    print_hex(failbit as u32);
    print_char(b']');

    unsafe {
        asm!("hlt");
    }

    loop {} // Should never reach here, this is just to satisfy the no return type.
}
