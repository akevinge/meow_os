// This module provides utilities for reading and writing to the disk in real mode.
// Reference: https://wiki.osdev.org/Disk_access_using_the_BIOS_(INT_13h)#Converting_LBA_to_CHS.

use core::arch::asm;

pub fn get_partition_table() -> &'static [u8] {
    extern "C" {
        static _partition_table: u8;
    }

    unsafe {
        let partition_table_ptr = &_partition_table as *const u8;
        let partition_table = core::slice::from_raw_parts(partition_table_ptr, 16 * 4);
        partition_table
    }
}

// check_lba_extension_support checks if the BIOS supports LBA extensions.
// Carry flag is set if LBA extensions are not supported.
// Reference: https://wiki.osdev.org/Disk_access_using_the_BIOS_(INT_13h)#Converting_LBA_to_CHS.
pub fn check_lba_extension_support() -> bool {
    let mut supported: u8;
    unsafe {
        asm!(
            "pushf",            // Push EFLAG register to stack.
                                // This is so we can restore the carry flag later.
            "push bx",
            "push dx",
            "clc",              // Clear carry flag.
            "mov ah, 0x41",     // Ritual to check extensions.
            "mov bx, 0x55aa",
            "mov dl, 0x80",
            "int 0x13",
            "setc al",          // Set al to carry flag (1 if set, else 0).
            "popf",
            "pop dx",
            "pop bx",
            out("al") supported,
        );
    }
    supported == 0 // Carry flag is NOT set if LBA extensions are supported.
}

#[repr(C)]
pub struct PartitionTableEntry {
    // If true, partition is bootable.
    pub bootable: bool,
    // LBA start address of partition.
    pub lba_start: u32,
    // Number of sectors in partition.
    pub sector_count: u32,
}

impl PartitionTableEntry {
    // from_raw converts a raw partition table entry to a PartitionTableEntry.
    // @param partition_table_raw: The raw partition table bytes.
    // Reference: https://en.wikipedia.org/wiki/Master_boot_record#PTE.
    pub fn from_raw(partition_table_raw: &[u8], index: usize) -> Self {
        let index = index - 1; // Partitions are 1-indexed.
        let start_index = index * 16;
        let end_index = index * 16 + 16;
        let raw_entry = &partition_table_raw[start_index..end_index];

        let bootable = raw_entry[0] == 0x80;
        let lba_start = raw_entry[8] as u32
            | (raw_entry[9] as u32) << 8
            | (raw_entry[10] as u32) << 16
            | (raw_entry[11] as u32) << 24;
        let sectors = raw_entry[12] as u32
            | (raw_entry[13] as u32) << 8
            | (raw_entry[14] as u32) << 16
            | (raw_entry[15] as u32) << 24;

        PartitionTableEntry {
            bootable,
            lba_start,
            sector_count: sectors,
        }
    }
}

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
                "push 'D'",
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
