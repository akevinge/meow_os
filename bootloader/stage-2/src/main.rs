#![no_std]
#![no_main]

mod screen;

use core::{arch::global_asm, panic::PanicInfo};

use crate::screen::print;

// Logic to enable A20 line encapsulated in ASM file due to complexity.
global_asm!(include_str!("enable_a20.s"));

// enable_a20 enables the A20 line.
// See enable_a20.s for implementation.
fn enable_a20() {
    extern "C" {
        static enable_a20: u8;
    }

    unsafe {
        let enable_a20_ptr = &enable_a20 as *const u8;
        let _enable_a20: fn() = core::mem::transmute(enable_a20_ptr);
        _enable_a20();
    }
}

#[no_mangle]
#[link_section = ".start"]
pub extern "C" fn _start() -> ! {
    static REAL_MODE_MSG: &str = "Entering second stage...";
    print(REAL_MODE_MSG);
    enable_a20();
    loop {}
}

#[panic_handler]
fn panic(_info: &PanicInfo) -> ! {
    loop {}
}
