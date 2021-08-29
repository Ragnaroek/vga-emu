//Rectangle example from https://github.com/jagregory/abrash-black-book/blob/master/src/chapter-48.md (LISTING 48.2)

use std::sync::Arc;

use vga::screen;
use vga::{SCReg, GCReg, set_vertical_display_end};

const SCREEN_WIDTH : usize = 80;
const PATTERN_BUFFER : usize = 0xfffc;

const LEFT_CLIP_PLANE_MASK: [u8; 4] = [0x0f, 0x0e, 0x0c, 0x08];
const RIGHT_CLIP_PLANE_MASK: [u8; 4] = [0x0f, 0x01, 0x03, 0x07];

const PATT_TABLE: [[u8; 16]; 16] = [
	[10,0,10,0,0,10,0,10,10,0,10,0,0,10,0,10],
	[9,0,0,0,0,9,0,0,0,0,9,0,0,0,0,9],
    [5,0,0,0,0,0,5,0,5,0,0,0,0,0,5,0],
	[14,0,0,14,0,14,14,0,0,14,14,0,14,0,0,14],
	[15,15,15,1,15,15,1,1,15,1,1,1,1,1,1,1],
	[12,12,12,12,6,6,6,12,6,6,6,12,6,6,6,12],
	[80,80,80,80,80,80,80,80,80,80,80,80,80,80,80,15],
	[78,78,78,78,80,80,80,80,82,82,82,82,84,84,84,84],
	[78,80,82,84,80,82,84,78,82,84,78,80,84,78,80,82],
	[78,80,82,84,78,80,82,84,78,80,82,84,78,80,82,84],
	[0,1,2,3,4,5,6,7,8,9,10,11,12,13,14,15],
	[0,1,2,3,0,1,2,3,0,1,2,3,0,1,2,3],
	[14,14,9,9,14,9,9,14,9,9,14,14,9,14,14,9],
	[15,8,8,8,15,15,15,8,15,15,15,8,15,8,8,8],
	[3,3,3,3,3,7,7,3,3,7,7,3,3,3,3,3],
	[0,0,0,0,0,64,0,0,0,0,0,0,0,0,0,89],
];

pub fn main() {
	let vga = vga::new(0x13);

	//enable Mode X
	let mem_mode = vga.get_sc_data(SCReg::MemoryMode);
	vga.set_sc_data(SCReg::MemoryMode, (mem_mode & !0x08) | 0x04); //turn off chain 4 & odd/even
	set_vertical_display_end(&vga, 480);

	for j in 0..4 {
		for i in 0..4 {
			fill_pattern_x(&vga, i*80, j*60, i*80+80, j*60+60, 0, &PATT_TABLE[j*4+i]);
		}
	}

	let vga_m = Arc::new(vga);

	let options: screen::Options = vga::screen::Options {
		show_frame_rate: true,
		..Default::default()
	};
	screen::start(vga_m, options).unwrap()
}

fn fill_pattern_x(vga: &vga::VGA, start_x: usize, start_y: usize, end_x: usize, end_y: usize, page_base: usize, pattern: &[u8; 16]) {
	
	if end_x <= start_x || end_y <= start_y {
		return;
	}

	for i in 0..4 {
		vga.set_sc_data(SCReg::MapMask, 1);
		vga.write_mem((PATTERN_BUFFER-1) + i, pattern[i*4]);

		vga.set_sc_data(SCReg::MapMask, 2);
		vga.write_mem((PATTERN_BUFFER-1) + i, pattern[i*4+1]);
		
		vga.set_sc_data(SCReg::MapMask, 4);
		vga.write_mem((PATTERN_BUFFER-1) + i, pattern[i*4+2]);

		vga.set_sc_data(SCReg::MapMask, 8);
		vga.write_mem((PATTERN_BUFFER-1) + i, pattern[i*4+3]);
	}
	vga.set_gc_data(GCReg::BitMask, 0);

	let mut si = (start_y & 0x03) + (PATTERN_BUFFER-1);
	let mut di = start_y * SCREEN_WIDTH + (start_x >> 2) + page_base;

	let left_clip = LEFT_CLIP_PLANE_MASK[start_x & 0x03];
	let right_clip = RIGHT_CLIP_PLANE_MASK[end_x & 0x03];

	let height = end_y - start_y;
	let width = ((end_x - 1) - (start_x & !0x03)) >> 2;

	for _ in 0..height {
		let _ = vga.read_mem(si); //latch pattern
		si += 1;
		if si >= vga::PLANE_SIZE {
			si -= 4;
		}
		vga.set_sc_data(SCReg::MapMask, left_clip);
		vga.write_mem(di, 0x00);

		vga.set_sc_data(SCReg::MapMask, 0x0F);
		for w in 0..(width-1) {
			vga.write_mem(di+(w+1), 0x00);
		}

		vga.set_sc_data(SCReg::MapMask, right_clip);
		vga.write_mem(di+width, 0x00);

		di += SCREEN_WIDTH;
	}

	vga.set_gc_data(GCReg::BitMask, 0xFF);
}