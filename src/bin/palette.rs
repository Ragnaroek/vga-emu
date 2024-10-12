use std::sync::Arc;
use std::fs;
use std::io;
use std::env;

use vga::util;
use vga::{SCReg, ColorReg};

const SCREEN_WIDTH : usize = 320;
const SCREEN_HEIGHT : usize = 200;
const CUBE_SIZE : usize = 10;
const PALETTE_SIZE : usize = 16;


fn main() -> io::Result<()> {
	let vga = vga::new(0x13);
	
	//enable Mode X
	let mem_mode = vga.get_sc_data(SCReg::MemoryMode);
	vga.set_sc_data(SCReg::MemoryMode, (mem_mode & !0x08) | 0x04); //turn off chain 4 & odd/even

	let mut args = env::args();
	if args.len() == 2 {
		let palette = fs::read(args.nth(1).unwrap())?;
		set_palette(&vga, &palette);
	}

	util::fill_rectangle_x(&vga, 0, 0, SCREEN_WIDTH, SCREEN_HEIGHT, 0, 0); 

	let palette_size = (PALETTE_SIZE * (CUBE_SIZE+1)) - 1;
	let x_start = (SCREEN_WIDTH - palette_size) / 2;
	let y_start = (SCREEN_HEIGHT - palette_size) / 2;

	for w in 0..PALETTE_SIZE {
		for h in 0..PALETTE_SIZE {
			let x = x_start + w*(CUBE_SIZE+1);
			let y = y_start + h*(CUBE_SIZE+1);
			util::fill_rectangle_x(&vga, x, y, x+CUBE_SIZE, y+CUBE_SIZE, 0, (h*PALETTE_SIZE+w) as u8);
		}
	}

	let vga_m = Arc::new(vga);

	let options: vga::Options = vga::Options {
		show_frame_rate: true,
		..Default::default()
	};
	vga_m.start(options).unwrap();
	Ok(())
}

pub fn set_palette(vga: &vga::VGA, palette: &[u8]) {
	assert_eq!(palette.len(), 768, "palette file must be exact 768 bytes long");
    vga.set_color_reg(ColorReg::AddressWriteMode, 0);
    for i in 0..768 {
        vga.set_color_reg(ColorReg::Data, palette[i]);
    }
}