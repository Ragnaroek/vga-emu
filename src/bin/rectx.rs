//Example from https://www.phatcode.net/res/224/files/html/ch47/47-07.html (LISTING 47.6)

mod lib;

use std::sync::Arc;

use vga::screen;
use vga::{SCReg, set_vertical_display_end};

fn main() {
	let vga = vga::new(0x13);

	//enable Mode X
	let mem_mode = vga.get_sc_data(SCReg::MemoryMode);
	vga.set_sc_data(SCReg::MemoryMode, (mem_mode & !0x08) | 0x04); //turn off chain 4 & odd/even
	set_vertical_display_end(&vga, 480);

	lib::fill_rectangle_x(&vga, 0, 0, 320, 240, 0, 0); 

	let mut j = 1;
	while j < 220 {
		let mut i = 1;
		while i < 300 {
			lib::fill_rectangle_x(&vga, i, j, i+20, j+20, 0, (((j/21*15)+i/21) & 0xFF) as u8);
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