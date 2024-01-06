// This module is responsible for enabling protected mode.

use core::arch::{asm, global_asm};

use crate::{
    gdt::{self, CODE_SEG, DATA_SEG},
    interrupts::{disable_interrupts, enable_interrupts},
};

// set_protected_mode_bit sets the protected mode bit in the CR0 register.
pub fn set_protected_mode_bit() {
    unsafe {
        asm!(
            "mov eax, cr0",
            "or al, 1",
            "mov cr0, eax",
            options(nomem, nostack, preserves_flags)
        );
    }
}

// unset_protected_mode_bit unsets the protected mode bit in the CR0 register.
pub fn unset_protected_mode_bit() {
    unsafe {
        asm!(
            "mov eax, cr0",
            "and eax, 0xfe",
            "mov cr0, eax",
            options(nostack, preserves_flags)
        );
    }
}

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

// enter_pm_and_jmp switches to protected mode and jumps to the kernel.
// This function assumes that A20 line was enabled and GDT was already loaded.
// @param dst: Destination address to jump to.
pub fn enter_pm_and_jmp(dst: *const u8) {
    // Disable interrupts.
    disable_interrupts();

    // Set protected mode bit.
    set_protected_mode_bit();

    // Long jump to set CS and clear CPU pipeline of real mode instructions.
    unsafe {
        // AT&T syntax long jump to protected mode.
        // Odd bug with Intel syntax: https://www.reddit.com/r/rust/comments/o8lrz8/how_do_i_get_a_far_absolute_jump_with_inline.
        // Jump to CODE_SEG:local_label. See CODE_SEG in GDT.
        asm!(
            "ljmp ${code_seg}, $2f",
            code_seg = const CODE_SEG,
            options(att_syntax)
        );
        asm!(
            "2:",                  // Local label for long jump.
            ".code32",
            "mov ax, {0:x}",       // Set segment registers to DATA_SEG in GDT.
            "mov ds, ax",
            "mov es, ax",
            "mov ss, ax",

            // Change stack pointer to 0x90000.
            "mov ebp, 0x90000",
            "mov esp, ebp",

            // Jump to kernel.
            "mov {2}, {1}",
            "call {2}",
            in(reg) DATA_SEG,
            in(reg) dst,
            out(reg) _,
        );
    }
}

// enter_unreal_mode enters unreal mode.
// Unreal mode is a mode where we stay in real mode, but
// load our GDT so we can use 32-bit addressing.
// This gives us access to more memory to load our kernel.
pub fn enter_unreal_mode() {
    // Enable A20 line.
    enable_a20();
    // Disabling interrupts prevents triple faults when switching
    // to protected mode (due to lack of IDT).
    disable_interrupts();

    // Save real mode DS and SS.
    unsafe {
        asm!("push ds", "push ss");
    }

    // Load GDT.
    gdt::load();

    // Switch to protected mode.
    set_protected_mode_bit();

    // Set DS and SS to GDT segment descriptor.
    // We don't set CS because we don't want to jump to protected mode.
    unsafe {
        asm!("mov ds, {0:x}", "mov ss, {0:x}", in(reg) DATA_SEG);
    }

    // Switch back to real mode.
    unset_protected_mode_bit();

    // Restore real mode DS and SS.
    unsafe {
        asm!("pop ss", "pop ds");
    }

    // Re-enable interrupts.
    enable_interrupts();
}
