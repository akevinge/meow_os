// This is the first stage bootloader.
// Its only job is to load and jump to the second stage bootloader.
// See bootsect-link.ld for boot sector disk layout.
#![no_std]
#![no_main]

mod disk;
mod error;
mod screen;

use core::{arch::global_asm, panic::PanicInfo};

use disk::get_partition_table_ptr;

use crate::{
    disk::{check_lba_extension_support, DiskAddressPacket, PartitionTable},
    error::err_fatal,
    screen::print,
};

global_asm!(include_str!("boot.s"));

fn get_second_stage_ptr() -> *const u8 {
    extern "C" {
        static _second_stage_start: u8;
    }
    unsafe { &_second_stage_start as *const u8 }
}

// get_second_stage returns a function pointer to the second stage bootloader.
// This is done by casting the address of the _second_stage_start symbol, which
// is defined in the linker script, to a function pointer.
fn jmp_to_second_stage(drive_number: u16) {
    unsafe {
        let second_stage_ptr = get_second_stage_ptr();
        let second_stage: fn(dist_number: u16, partition_table: *const u8) =
            core::mem::transmute(second_stage_ptr);
        second_stage(drive_number, get_partition_table_ptr());
    }
}

#[no_mangle]
pub extern "C" fn first_stage(drive_number: u16) {
    static BOOT_MSG: &str = "Booted into first stage...";
    print(BOOT_MSG);

    let lba_supported = check_lba_extension_support();
    // Could use CHS if LBA is not supported instead of erroring?
    if !lba_supported {
        err_fatal(b'L');
    }

    // Partitions are 1-indexed.
    let mut partition_table = PartitionTable::from_ptr(get_partition_table_ptr());
    let second_stage_partition = partition_table.load_entry(1);

    let mut sectors_left = second_stage_partition.sector_count;
    let mut current_lba = second_stage_partition.lba_start as u64;
    let mut memory_addr = get_second_stage_ptr() as u16; // Addresses can only be 16-bit in real mode.

    while sectors_left > 0 {
        let sectors_to_read = u32::min(sectors_left, 127) as u16; // Max sectors to read is 127 on some BIOSes.
        let dap = DiskAddressPacket::new(
            sectors_to_read,
            // offset is the lower 4 bits of the address
            // segment is shifted left by 4 bits during translation, so we preemtively shift it right
            // ex: 0x7E00 -> 0x07E0(ds):0x0000(si)
            memory_addr & 0xF,
            memory_addr >> 4,
            current_lba,
        );

        dap.load(drive_number as u8);

        current_lba += sectors_to_read as u64;
        sectors_left -= sectors_to_read as u32;
        memory_addr += sectors_to_read * 512;
    }

    jmp_to_second_stage(drive_number);
}

#[panic_handler]
fn panic(_info: &PanicInfo) -> ! {
    err_fatal(b'P')
}
