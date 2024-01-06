// This module defines a buffer for reading from disk.
//
// DAP can only load into memory using segment registers.
// This means that our maximum memory address is 0xFFFFF.
// Our kernel sits at 0x100000, so we need to use a buffer and copy
// the kernel into its final destination.
// Our buffer is 4KB, so we can only load 4KB at a time.
// This number can be adjusted if needed, but will affect the size
// of our bootloader.

pub struct DiskBuffer<const BYTE_SIZE: usize> {
    pub buffer: [u8; BYTE_SIZE],
    cursor: usize,
}

impl<const BYTE_SIZE: usize> DiskBuffer<BYTE_SIZE> {
    pub const fn new() -> Self {
        Self {
            buffer: [0; BYTE_SIZE],
            cursor: 0,
        }
    }

    pub fn len(&self) -> usize {
        self.cursor
    }

    pub fn space_left(&self) -> usize {
        BYTE_SIZE - self.len()
    }

    pub fn as_ptr(&self) -> *const u8 {
        self.buffer.as_ptr_range().start
    }

    // clear resets the buffer and cursor.
    pub fn clear(&mut self) {
        for i in 0..BYTE_SIZE {
            self.buffer[i] = 0;
        }
        self.cursor = 0;
    }

    // advance moves the cursor forward by the given number of bytes.
    pub fn advance(&mut self, bytes: usize) {
        self.cursor += bytes;
    }

    // copy_from_ptr copies the entire used portion of the buffer to memory.
    pub fn copy_to_ptr(&self, ptr: *mut u8) {
        unsafe {
            core::ptr::copy(self.buffer[0..self.cursor].as_ptr(), ptr, self.len());
        }
    }
}
