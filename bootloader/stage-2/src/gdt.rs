// This module defines the Global Descriptor Table (GDT).
// Reference: https://wiki.osdev.org/Global_Descriptor_Table

use core::{arch::asm, mem::size_of};

pub const GDT: GDT = GDT {
    null_segment: 0,
    code_segment: u64::from_le_bytes([
        0xFF,       // limit[0:15]
        0xFF,       //
        0x00,       // base[0:15]
        0x00,       //
        0x00,       // base[16:23]
        0b10011010, // access byte
        0b11001111, // flags + limit[16:19]
        0x00,       // base[24:31]
    ]),
    data_segment: u64::from_le_bytes([
        0xFF, // limit[0:15]
        0xFF, //
        0x00, // base[0:15]
        0x00, //
        0x00, // base[16:23]
        // 0b10010110, // access byte
        0b10010010, //
        0b11001111, // flags + limit[16:19]
        0x00,       // base[24:31]
    ]),
};

pub const CODE_SEG: usize = memoffset::offset_of!(GDT, code_segment);
pub const DATA_SEG: usize = memoffset::offset_of!(GDT, data_segment);

const GDT_DESCRIPTOR: GDTDescriptor = GDTDescriptor {
    offset: &GDT,
    size: (size_of::<GDT>() - 1) as u16,
};

pub fn load() {
    unsafe {
        asm!("lgdt [{0}]", in(reg) &GDT_DESCRIPTOR);
    }
}

// Global Descriptor Table (GDT).
// Entries are 64-bits long.
#[repr(C, packed)]
pub struct GDT {
    // Null segment is always 0.
    null_segment: u64,
    // Code segment:
    // - limit[0:15] = 0xFFFF
    // - base[0:15] = 0x0000
    // - base[16:23] = 0x00
    // - access byte = b10011010
    //   - present = b1 (present in mem)
    //   - privilege = b00 (ring-0)
    //   - descriptor type = b1 (code or data segment)
    //   - executable = b1 (executable segment)
    //   - conforming = b0 (can only be exec by ring-0)
    //   - readable = b1 (readable segment)
    //   - accessed = b0 (for CPU to set)
    // - limit[16:19] = 0xF
    // - flags (4-bits) = b1100
    //  - granularity = b1 (limit is in 4KiB blocks)
    //  - DB = b1 (32-bit protected mode)
    //  - L = b0 (not 64-bit)
    //  - Reserved = always b0
    // - base[24:31] = 0x00
    code_segment: u64,
    // data segment:
    // - limit[0:15] = 0xFFFF
    // - base[0:15] = 0x0000
    // - base[16:23] = 0x00
    // - access byte = b10010110
    //   - present = b1 (present in mem)
    //   - privilege = b00 (ring-0)
    //   - descriptor type = b1 (code or data segment)
    //   - executable = b0 (data segment)
    //   - direction = b1 (segment grows down)
    //   - writeable = b1 (writeable segment)
    //   - accessed = b0 (for CPU to set)
    // - limit[16:19] = 0xF
    // - flags (4-bits) = b1100
    //  - granularity = b1 (limit is in 4KiB blocks)
    //  - DB = b1 (32-bit protected mode)
    //  - L = b0 (not 64-bit)
    //  - Reserved = always b0
    // - base[24:31] = 0x00
    data_segment: u64,
}

#[repr(C, packed)]
pub struct GDTDescriptor {
    // Size of the GDT.
    pub size: u16,
    // Pointer to GDT.
    pub offset: *const GDT,
}
