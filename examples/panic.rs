#![no_std]
#![feature(start)]

extern crate gba;

use gba::display;

#[start]
fn main(_argc: isize, _argv: *const *const u8) -> isize {
    let gba = gba::Gba::new();
    let bitmap = gba.display.bitmap3();

    let mut input = gba::input::ButtonController::new();

    loop {
        input.update();
        if input.is_just_pressed(gba::input::Button::A) {
            bitmap.draw_point(display::WIDTH, 0, 0x05);
        }
    }
}
