// This is the second stage of the bootloader.
// This stage is responsible for entering protected mode.
#![no_std]
#![no_main]
#![feature(asm_const)]

mod disk;
mod error;
mod gdt;
mod interrupts;
mod protected_mode;
mod screen;

use core::panic::PanicInfo;

use crate::{
    disk::{buffer::DiskBuffer, dap, partition},
    error::err_fatal,
    protected_mode::enter_unreal_mode,
    screen::{print, print_char, print_hex},
};

// DAP only can only load into memory using segment registers.
// This means that our maximum memory address is 0xFFFFF.
// Our kernel sits at 0x100000, so we need to use a buffer and copy
// the kernel into its final destination.
// Our buffer is 4KB, so we can only load 4KB at a time.
// This number can be adjusted if needed, but will affect the size
// of our bootloader.
const _4KB: usize = 0x1000;
const DISK_BUFFER: DiskBuffer<_4KB> = DiskBuffer::new();
// The kernel is loaded at 0x100000. This is just outside of the memory
// that the bootloader has access to.
const KERNEL_DST: *const u8 = 0x100_000 as *const u8;
// Sector size is 512 bytes.
const LBA_SECTOR_SIZE: u16 = 512;

// Entry point for the second stage bootloader.
// Arguments are provided the previous stage.
// @param drive_number: The drive number to read from.
// @param partition_table_ptr: Pointer to the partition table.
#[no_mangle]
#[link_section = ".start"]
pub extern "C" fn _start(drive_number: u16, partition_table_ptr: *const u8) {
    static REAL_MODE_MSG: &str = "Entering second stage...";
    print(REAL_MODE_MSG);

    enter_unreal_mode();

    let mut partition_table = partition::PartitionTable::from_ptr(partition_table_ptr);
    let kernel_partition = partition_table.load_entry(2);

    let mut sectors_left = kernel_partition.sector_count;
    let mut current_lba = kernel_partition.lba_start as u64;
    let mut memory_addr = DISK_BUFFER.as_ptr() as u16; // Address will be in 16-bit space.
    let mut kernel_dst = KERNEL_DST as *mut u8;

    print_hex(unsafe { DISK_BUFFER.buffer.as_ptr() as *const u8 } as u32);
    // If partition is non-bootable, there was likely a formatting error.
    // Or we were unable to properly read the partition table.
    if !kernel_partition.bootable {
        err_fatal(b'B');
    }

    while sectors_left > 0 {
        let sectors_to_read = u32::min(sectors_left, 127) as u16; // Max sectors to read is 127 on some BIOSes.
        let bytes_to_read = sectors_to_read * LBA_SECTOR_SIZE;

        if bytes_to_read > DISK_BUFFER.space_left() as u16 {
            // When we need to read more bytes than we have space for, we need to copy the buffer
            // to memory and reset the cursor.
            // We also advance the kernel destination pointer.
            DISK_BUFFER.copy_to_ptr(kernel_dst);
            kernel_dst = unsafe { kernel_dst.add(DISK_BUFFER.len()) };
            memory_addr = DISK_BUFFER.as_ptr() as u16;
            DISK_BUFFER.clear();
        }

        let dap = dap::DiskAddressPacket::new(
            sectors_to_read,
            // offset is the lower 4 bits of the address
            // segment is shifted left by 4 bits during translation, so we preemtively shift it right
            // ex: 0x8D45 -> 0x08D4(ds):0x0005(si)
            memory_addr & 0xF,
            memory_addr >> 4,
            current_lba,
        );
        dap.load(drive_number as u8);

        current_lba += sectors_to_read as u64;
        sectors_left -= sectors_to_read as u32;
        memory_addr += sectors_to_read * LBA_SECTOR_SIZE;
        DISK_BUFFER.advance(bytes_to_read as usize);
    }
    // Copy the remaining bytes to memory.
    DISK_BUFFER.copy_to_ptr(kernel_dst);

    // enter_pm_and_jmp(KERNEL_DST);

    err_fatal(b'X');
}

#[panic_handler]
fn panic(_info: &PanicInfo) -> ! {
    err_fatal(b'P')
}
