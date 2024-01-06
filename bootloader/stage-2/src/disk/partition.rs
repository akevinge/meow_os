// This module provides partition table utilities.
// Note:
// This is not a complete implementation of the partition table.
// It only supports fields needed for reading from disk using DAP.

use crate::screen::print_char;

pub struct PartitionTable {
    ptr: *const u8,
    raw: &'static [u8],
}

impl PartitionTable {
    pub fn from_ptr(ptr: *const u8) -> Self {
        let raw = unsafe { core::slice::from_raw_parts(ptr, 16 * 4) };
        Self { ptr, raw }
    }

    // load_entry loads a partition table entry from the partition table.
    // @param index: The index of the partition table entry to load. Entries are 1-indexed.
    pub fn load_entry(&self, index: usize) -> PartitionTableEntry {
        PartitionTableEntry::from_raw(self.raw, index)
    }
}

#[derive(Clone, Copy)]
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
        assert!(index > 0 && index <= 4);
        let index = index - 1; // Partitions are 1-indexed.
        let start_index = index * 16;
        let end_index = index * 16 + 16;
        let raw_entry = &partition_table_raw[start_index..end_index];

        let bootable = raw_entry[0] == 0x80;
        // print_char((bootable as u8) + b'0');
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
