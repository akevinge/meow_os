# Query A20 support using BIOS 0x15 interrupt.
# See: https://fd.lod.bz/rbil/interrup/bios_vendor/152403.html
# @return ax = Table 00462 (see link above).    
# TLDR: bit 0 = support keyboard controller
#       bit 1 = support fast gate
# Sets CF if error
query_a20_support:
    push bx
    clc

    mov ax, 0x2400
    int 0x15
    jc query_a20_support__error     # if CF set, error

    test ah, ah                     # if ah == 0, query successful
    jnz query_a20_support__error    # else, error

    mov ax, bx
    jmp query_a20_support__exit

query_a20_support__error:
    stc

query_a20_support__exit:
    pop bx
    ret


# Enable A20 line.
# See for reference: https://wiki.osdev.org/A20_Line#Recommended_Method
# Steps:
#  1. If A20 is already enabled, exit.
#  2. Query A20 support using BIOS 0x15 interrupt.
#     If error, try BIOS method.
#  3. If keyboard controller method is possible, use it.
#  4. If fast gate method is possible, use it.
#  5. If BIOS 0x15 interrupt method is possible, use it.
enable_a20:
    clc             # Clear carry flag.
    pusha           # Push all registers onto stack.

    call check_a20
    test ax, ax            # if ax == 0, then A20 is disabled.
    jnz enable_a20__exit   # else, A20 enabled, exit.

    call query_a20_support
    mov bl, al

    test bl, 1             # Test if keyboard controller method is possible (bit 0 set).
    jnz enable_a20_keyboard_controller

    test bl, 2             # Test if fast gate method is possible (bit 1 set).
    jnz enable_a20__fast_gate

enable_a20__bios:
    # Enable A20 using BIOS 0x15 interrupt.
    # See: http://mirror.cs.msu.ru/oldlinux.org/Linux.old/docs/interrupts/int-html/rb-1336.htm
    mov ax, 0x2401
    int 0x15
    jc enable_a20__fast_gate    # if CF set, error occured, try fast gate instead.
    test ah, ah                 # if ah == 0, A20 successfully enabled
    jnz enable_a20__failed      # else, error occured, fail. No documentation on this?

    call check_a20
    test ax, ax
    jnz enable_a20__exit        # if ax == 1, A20 successfully enabled, exit.

enable_a20__fast_gate:
    # Enable A20 using fast gate method.
    # See: https://www.win.tue.nl/~aeb/linux/kbd/A20.html
    in al, 0x92             # Read port 0x92
    test al, 2              # Test if bit 1 is set (A20 already enabled)
    jnz enable_a20__exit
    or al, 2                # Set bit 1 to 1 (enable A20).
    and al, 0xfe            # Set bit 0 to 0 (disable fast reset)
    out 0x92, al            # Write our new value back to port 0x92

    call check_a20
    test ax, ax
    jnz enable_a20__exit    # if ax == 1, A20 successfully enabled, exit.

enable_a20_keyboard_controller:
    # Enable A20 using keyboard controller method.
    # See: https://www.win.tue.nl/~aeb/linux/kbd/scancodes-11.html
    # 0x64 = keyboard controller command port
    # 0x60 = keyboard controller data port
    cli                    # Disable interrupts.
    
    call enable_a20_keyboard_controller_in_wait
    mov al, 0xAD           # 0xAD = disable keyboard
    out 0x64, al           # Write 0xAD to port 0x64

    call enable_a20_keyboard_controller_in_wait
    mov al, 0xD0           # 0xD0 = read output port
    out 0x64, al           # Write 0xD0 to port 0x64
                           # Output format: https://www.win.tue.nl/~aeb/linux/kbd/scancodes-11.html#outputport

    call enable_a20_keyboard_controller_out_wait
    in al, 0x60             # Read port 0x60
    push ax

    call enable_a20_keyboard_controller_in_wait
    mov al, 0xD1           # 0xD1 = write output port
    out 0x64, al           # Write 0xD1 to port 0x64

    call enable_a20_keyboard_controller_in_wait
    pop ax
    or al, 2               # Set bit 1 to 1 (enable A20).
    out 0x60, al           # Write our new value back to port 0x60

    call enable_a20_keyboard_controller_in_wait
    mov al, 0xAE           # 0xAE = re-enable keyboard
    out 0x64, al           # Write 0xAE to port 0x64

    call enable_a20_keyboard_controller_in_wait

    sti                   # Enable interrupts.

    call check_a20
    test ax, ax
    jnz enable_a20__exit    # if ax == 1, A20 successfully enabled, exit.

    mov bh, 1               # Set bh to 1 (keyboard try failed)

    test bl, 2              # Test if fast gate method is possible (bit 1 set).
                            # bx was set earlier by the query.
                            # See: https://fd.lod.bz/rbil/interrup/bios_vendor/152403.html
    jnz enable_a20__fast_gate

    jmp enable_a20__failed

enable_a20_keyboard_controller_in_wait:
    in al, 0x64            # Read port 0x64
    test al, 2             # Test if bit 1 is set (input buffer full)
    jnz enable_a20_keyboard_controller_in_wait # if input buffer is full, wait
    ret 

enable_a20_keyboard_controller_out_wait:
    in al, 0x64            # Read port 0x64
    test al, 1             # Test if bit 0 is set (output buffer full)
    jz enable_a20_keyboard_controller_out_wait # if output buffer is empty, wait
    ret

enable_a20__failed:
    stc             # Set carry flag to 1.

enable_a20__exit:
    popa
    ret

# Checks if A20 is enabled.
# @return ax = 0 (disabled), 1 (enabled).
check_a20:
    pushf           # Push EFLAG registers onto stack
                    # This includes the IF (interrupt enable flag).

    push ds         # Save the segment registers we're using.
    push si
    push es
    push di

    cli             # Disable interrupts.

    mov ax, 0x0000  # Set ds to 0x0000. We can't set ds directly.
    mov ds, ax
    mov si, 0x0500  # ds:si -> 0x0000:0x0500 -> 0x0500 address
    
                    # 1MiB = 16^5 bytes = 0x100000
                    # If A20 is disabled, 0x100500 should wrap back to 0x000500.
                    # Working backwards:
                    # 0x100500 = 0xffff0 + 0x0510 (di)
                    # 0xffff0 >> 4 = 0xffff -> this is our es

    mov ax, 0xffff              # Set es to 0xffff. We can't set es directly.
    mov es, ax
    mov di, 0x0510              # es:di -> 0xffff:0x0510 -> 0x100500 address

    mov al, byte ptr es:[di]    # Read the byte at 0x100500.
    push ax                     # Save the byte on the stack.

    mov al, byte ptr ds:[si]    # Read the byte at 0x000500.
    push ax                     # Save the byte on the stack.

    mov byte ptr es:[di], 0x00  # Write 0x00 to 0x100500.
    mov byte ptr ds:[si], 0xff  # Write 0xff to 0x000500. 
                                # If A20 is disabled, this will wrap to 0x100500 and 0x100500 will be 0xff.


    cmp byte ptr es:[di], 0xff  # if 0x100500 == 0xff, then A20 is disabled.

    pop ax                      # Restore the byte at 0x000500.
    mov byte ptr ds:[si], al

    pop ax                      # Restore the byte at 0x100500.
    mov byte ptr es:[di], al

    mov ax, 0                   # ax = 0 if A20 is disabled
    je check_a20__exit          # If A20 is disabled, jump to check_a20__exit.

    mov ax, 1                   # ax = 1 if A20 is enabled.

check_a20__exit:
    pop di          # Restore the segment registers.  
    pop es
    pop si
    pop ds
    popf            # Restore the EFLAG registers.
                    # This restores the IF flag, so no need to call sti.
    ret