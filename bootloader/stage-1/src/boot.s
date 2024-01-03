# The entry point for the boot sector.
# Setups up the stack and calls to Rust code.
# Due to limited space, A20 line is set up in second stage.
.section .boot, "awx"
.global _start
.code16

_start:
    # Initialize stack to ds:0x7c00
    # Stack growns downwards from 0x7c00 -> 0x0500
    # See _stack_end in bootsect-link.ld.
    mov sp, _stack_end

rust:
    push dx     # disk number
    call first_stage

spin:
    hlt
    jmp spin
