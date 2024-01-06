// This module provides utilities for reading and writing to the disk in real mode.
// Reference: https://wiki.osdev.org/Disk_access_using_the_BIOS_(INT_13h)#Converting_LBA_to_CHS.

use core::arch::asm;

// DiskAddressPacket (DAP) is used to read sectors from the disk into memory.
// DAP follows a hardware structure.
// Reference: https://wiki.osdev.org/Disk_access_using_the_BIOS_(INT_13h)#LBA_in_Extended_Mode.
#[repr(C, packed)]
pub struct DiskAddressPacket {
    // Size of the packet (1 byte).
    packet_size: u8,
    // Always zero (1 byte).
    always_zero: u8,
    // Number of sectors to read (1 byte, max 120 on some BIOSes).
    sector_count: u16,
    // Memory location (segment:offset).
    offset: u16,
    segment: u16,
    // 48-bit starting LBA address.
    lba_start: u64,
}

impl DiskAddressPacket {
    pub fn new(sector_count: u16, offset: u16, segment: u16, lba_start: u64) -> Self {
        Self {
            packet_size: 0x10, // Packet size is always 16 bytes.
            always_zero: 0,    // Always zero.
            sector_count,
            segment,
            offset,
            lba_start,
        }
    }

    // load reads sectors from the disk into memory.
    // @param drive_number: The drive number to read from.
    // Reference: https://wiki.osdev.org/Disk_access_using_the_BIOS_(INT_13h)#LBA_in_Extended_Mode.
    pub fn load(&self, drive_number: u8) {
        unsafe {
            let addr = self as *const Self;
            asm!(
                "pusha",
                "push 0x81",         // Error code passed into err_fatal.
                "mov si, {0:x}",    // Load address of DAP.
                "mov ah, 0x42",     // Ritual to read sectors.
                "int 0x13",
                "jc err_fatal",     // Jump to error::err_fatal if read errors.
                "pop ax",
                "popa",
                in(reg) addr,
                in("dl") drive_number
            );
        }
    }
}
