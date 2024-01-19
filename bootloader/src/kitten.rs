use alloc::vec;
use alloc::vec::Vec;
use uefi::{
    proto::console::gop::{BltPixel, GraphicsOutput},
    Result,
};

use crate::screen::Buffer;

const BACKGROUND_COLOR: BltPixel = BltPixel::new(3, 240, 252);

pub struct KittenPen {
    actual_width: usize,
    actual_height: usize,
    effective_width: usize,
    effective_height: usize,
    downsize_factor: usize,
    buffer: Buffer,
}

impl KittenPen {
    pub fn new(width: usize, height: usize, downsize_factor: usize) -> Self {
        Self {
            actual_width: width,
            actual_height: height,
            effective_width: width / downsize_factor,
            effective_height: height / downsize_factor,
            downsize_factor,
            buffer: Buffer::new(width, height),
        }
    }

    pub fn paint_background(&mut self, gop: &mut GraphicsOutput) -> Result {
        for y in 0..self.buffer.get_height() {
            for x in 0..self.buffer.get_width() {
                self.buffer.set_pixel(x, y, BACKGROUND_COLOR);
            }
        }

        self.buffer.blit(gop)
    }

    pub fn paint_effective_pixel(
        &mut self,
        x: usize,
        y: usize,
        gop: &mut GraphicsOutput,
    ) -> Result {
        let actual_x = x * self.downsize_factor;
        let actual_y = y * self.downsize_factor;
        for i in 0..self.downsize_factor {
            for j in 0..self.downsize_factor {
                self.buffer
                    .set_pixel(actual_x + i, actual_y + j, BltPixel::new(0, 0, 0));
            }
        }

        self.buffer.blit(gop)
    }
}
