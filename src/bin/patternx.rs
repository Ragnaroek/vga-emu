//Rectangle example from https://github.com/jagregory/abrash-black-book/blob/master/src/chapter-48.md (LISTING 48.2)

use std::sync::Arc;

use vga::screen;
//use vga::{CRTReg, GCReg, GeneralReg, SCReg, AttributeReg};

const SCREEN_WIDTH : usize = 80;

pub fn main() {
	let vga = vga::new(0x13);

	fill_rectangle_x(&vga, 0, 0, 320, 240, 0, 0xFF);

	let options : screen::Options = vga::screen::Options { show_frame_rate: true, ..Default::default() };
	screen::start(Arc::new(vga), options).unwrap()
}

fn fill_rectangle_x(vga: &vga::VGA, start_x: usize, start_y: usize, end_x: usize, end_y: usize, page_base: usize, color: u8) {
	let width = end_x - start_x;
	let height = end_y - start_y;

	for h in 0..height/4 {
		for w in 0..width/4 {
			vga.write_mem(h * SCREEN_WIDTH + w, color);
		}
	}
}