#![no_main]
#![no_std]

extern crate alloc;

mod kitten;
mod screen;

use kitten::KittenPen;
use uefi::{
    entry,
    proto::console::gop::GraphicsOutput,
    table::{Boot, SystemTable},
    Handle, Status,
};

#[entry]
fn main(_image_handle: Handle, mut system_table: SystemTable<Boot>) -> Status {
    uefi_services::init(&mut system_table).unwrap();
    let bt = system_table.boot_services();

    let gop_handle = bt.get_handle_for_protocol::<GraphicsOutput>().unwrap();
    let mut gop = bt
        .open_protocol_exclusive::<GraphicsOutput>(gop_handle)
        .unwrap();

    let (width, height) = gop.current_mode_info().resolution();

    let mut kitten_pen = KittenPen::new(width, height, 4);
    kitten_pen.paint_background(&mut gop);
    kitten_pen.paint_effective_pixel(0, 0, &mut gop);

    system_table.boot_services().stall(10_000_000);
    Status::SUCCESS
}
