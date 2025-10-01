//Example from https://www.phatcode.net/res/224/files/html/ch47/47-07.html (LISTING 47.6)

use std::sync::Arc;

use vga::screen;
use vga::{SCReg, set_vertical_display_end};

const SCREEN_WIDTH: usize = 80;

const LEFT_CLIP_PLANE_MASK: [u8; 4] = [0x0f, 0x0e, 0x0c, 0x08];
const RIGHT_CLIP_PLANE_MASK: [u8; 4] = [0x0f, 0x01, 0x03, 0x07];

fn main() {
	let vga = vga::new(0x13);

	//enable Mode X
	let mem_mode = vga.get_sc_data(SCReg::MemoryMode);
	vga.set_sc_data(SCReg::MemoryMode, (mem_mode & !0x08) | 0x04); //turn off chain 4 & odd/even
	set_vertical_display_end(&vga, 480);

	fill_rectangle_x(&vga, 0, 0, 320, 240, 0, 0); 

	let mut j = 1;
	while j < 220 {
		let mut i = 1;
		while i < 300 {
			fill_rectangle_x(&vga, i, j, i+20, j+20, 0, (((j/21*15)+i/21) & 0xFF) as u8);
			i += 21;
		}
		j += 21;
	}

	let vga_m = Arc::new(vga);

	let options: screen::Options = vga::screen::Options {
		show_frame_rate: true,
		..Default::default()
	};
	screen::start(vga_m, options).unwrap()
}

fn fill_rectangle_x(vga: &vga::VGA, start_x: usize, start_y: usize, end_x: usize, end_y: usize, page_base: usize, color: u8) {
	
	if end_x <= start_x || end_y <= start_y {
		return;
	}

	let left_clip = LEFT_CLIP_PLANE_MASK[(start_x & 0x03) as usize]; 
	let right_clip = RIGHT_CLIP_PLANE_MASK[(end_x & 0x03) as usize]; 
	
	let mut di = start_y * SCREEN_WIDTH + (start_x >> 2) + page_base; 
	
	let height = end_y - start_y;
	let width = ((end_x - 1) - (start_x & !0x03)) >> 2;

	for _ in 0..height {
		vga.set_sc_data(SCReg::MapMask, left_clip);
		vga.write_mem(di, color);

		vga.set_sc_data(SCReg::MapMask, 0x0F);
		for w in 0..(width-1) {	
			vga.write_mem(di+(w+1), color);
		}

		vga.set_sc_data(SCReg::MapMask, right_clip);
		vga.write_mem(di+width, color);

		di += SCREEN_WIDTH;
	}
}