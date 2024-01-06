// This module contains utilities for enabling and disabling interrupts.
// Protected mode no longer relies on BIOS IVT and uses IDT to handle interrupts.
// Interrupts must be disabled before entering protected mode
// in order to avoid double and triple faults as the IDT has yet to be setup.
// It is recommended to also disable NMI (non-maskable interrupt caused by hardware errors),
// but this can lead to undefined behavior in the event that an NMI is triggered.
// Instead, we will simply keep NMI's enabled and allow for a triple fault to occur for a more
// defined behavior.
// This is better explained in Brendan's reply here: https://forum.osdev.org/viewtopic.php?f=1&t=32256.

use core::arch::asm;

pub fn disable_interrupts() {
    unsafe {
        asm!("cli");
    }
}

pub fn enable_interrupts() {
    unsafe {
        asm!("sti");
    }
}
