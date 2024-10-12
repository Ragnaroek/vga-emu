// Provides various utils for implementing something with the VGA

use std::future::Future;
use std::time::Duration;
use async_std::task::{self, JoinHandle};

use crate::{GeneralReg, CRTReg};
use crate::{SCReg, GCReg, VGA, PLANE_SIZE};

const SCREEN_WIDTH : usize = 80;
const PATTERN_BUFFER : usize = 0xfffc;

const LEFT_CLIP_PLANE_MASK: [u8; 4] = [0x0f, 0x0e, 0x0c, 0x08];
const RIGHT_CLIP_PLANE_MASK: [u8; 4] = [0x0f, 0x01, 0x03, 0x07];

const VSYNC_MASK: u8 = 0x08;
const DE_MASK: u8 = 0x01;

const POLL_WAIT_MICROS : u64 = 500;

const CLEAR_DE_MASK: u8 = 0b11111110;
const CLEAR_VR_MASK: u8 = 0b11110111;

#[cfg(feature = "sdl")]
pub fn spawn_task<F, T>(future: F) -> JoinHandle<T>
where
    F: Future<Output = T> + Send + 'static,
    T: Send + 'static,
{
	task::spawn(future)
}

#[cfg(feature = "web")]
pub fn spawn_task<F, T>(future: F) -> JoinHandle<T>
where
    F: Future<Output = T> + 'static,
    T: 'static,
{
    task::spawn_local(future)
}

// wait routines

pub async fn display_enable(vga: &VGA) {
    loop {
        let in1 = vga.get_general_reg(GeneralReg::InputStatus1);
        if in1 & DE_MASK == 0 {
            break;
        }
		task::sleep(Duration::from_micros(POLL_WAIT_MICROS)).await; 
    }
}

pub async fn vsync(vga: &VGA) {
    loop {
        let in1 = vga.get_general_reg(GeneralReg::InputStatus1);
        if in1 & VSYNC_MASK != 0 {
            break;
        }
		task::sleep(Duration::from_micros(POLL_WAIT_MICROS)).await; 
    }
}

/// width in pixel
pub fn get_width(vga: &VGA) -> u32 {
    (vga.get_crt_data(CRTReg::HorizontalDisplayEnd) as u32 + 1) * 8
}

pub fn get_height(vga: &VGA) -> u32 {
    get_vertical_display_end(&vga) + 1
}

/// Constructs the Vertical Display End from the register + offset register
fn get_vertical_display_end(vga: &VGA) -> u32 {
    let vde_lower = vga.get_crt_data(CRTReg::VerticalDisplayEnd);
    let overflow = vga.get_crt_data(CRTReg::Overflow);
    let vde_bit_8 = (overflow & 0b0000_0010) >> 1;
    let vde_bit_9 = (overflow & 0b0100_0000) >> 5;
    let vde_upper = vde_bit_8 | vde_bit_9;
    let vde = vde_lower as u32;
    vde | ((vde_upper as u32) << 8)
}

/// display enable NOT
pub fn set_de(vga: &VGA, display_mode: bool) {
    let v0 = vga.get_general_reg(GeneralReg::InputStatus1);
    if display_mode {
        //flag needs to be set to zero (NOT)
        vga.set_general_reg(GeneralReg::InputStatus1, v0 & CLEAR_DE_MASK);
    } else {
        //not in display mode (vertical or horizontal retrace), set to 1
        vga.set_general_reg(GeneralReg::InputStatus1, v0 | !CLEAR_DE_MASK);
    }
}

/// vertical retrace
pub fn set_vr(vga: &VGA, set: bool) {
    let v0 = vga.get_general_reg(GeneralReg::InputStatus1);
    if set {
        vga.set_general_reg(GeneralReg::InputStatus1, v0 | !CLEAR_VR_MASK);
    } else {
        vga.set_general_reg(GeneralReg::InputStatus1, v0 & CLEAR_VR_MASK);
    }
}

/// Drawing helper

pub fn fill_pattern_x(vga: &VGA, start_x: usize, start_y: usize, end_x: usize, end_y: usize, page_base: usize, pattern: &[u8; 16]) {
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

	let mut left_clip = LEFT_CLIP_PLANE_MASK[start_x & 0x03];
	let right_clip = RIGHT_CLIP_PLANE_MASK[end_x & 0x03];

	let height = end_y - start_y;
	let width = ((end_x - 1) - (start_x & !0x03)) >> 2;

	if width == 0 {
		left_clip = left_clip & right_clip;
	}

	for _ in 0..height {
		let _ = vga.read_mem(si); //latch pattern
		si += 1;
		if si >= PLANE_SIZE {
			si -= 4;
		}
		vga.set_sc_data(SCReg::MapMask, left_clip);
		vga.write_mem(di, 0x00);

		if width > 0 {
			vga.set_sc_data(SCReg::MapMask, 0x0F);
			for w in 0..(width-1) {
				vga.write_mem(di+(w+1), 0x00);
			}

			vga.set_sc_data(SCReg::MapMask, right_clip);
			vga.write_mem(di+width, 0x00);
		}

		di += SCREEN_WIDTH;
	}

	vga.set_gc_data(GCReg::BitMask, 0xFF);
}

pub fn fill_rectangle_x(vga: &VGA, start_x: usize, start_y: usize, end_x: usize, end_y: usize, page_base: usize, color: u8) {
	if end_x <= start_x || end_y <= start_y {
		return;
	}

	let mut left_clip = LEFT_CLIP_PLANE_MASK[(start_x & 0x03) as usize]; 
	let right_clip = RIGHT_CLIP_PLANE_MASK[(end_x & 0x03) as usize]; 
	
	let mut di = start_y * SCREEN_WIDTH + (start_x >> 2) + page_base; 
	
	let height = end_y - start_y;
	let width = ((end_x - 1) - (start_x & !0x03)) >> 2;

	if width == 0 {
		left_clip = left_clip & right_clip;
	}

	for _ in 0..height {
		vga.set_sc_data(SCReg::MapMask, left_clip);
		vga.write_mem(di, color);

		if width > 0 {
			vga.set_sc_data(SCReg::MapMask, 0x0F);
			for w in 0..(width-1) {
				vga.write_mem(di+(w+1), color);
			}

			vga.set_sc_data(SCReg::MapMask, right_clip);
			vga.write_mem(di+width, color);
		}

		di += SCREEN_WIDTH;
	}
}

pub fn copy_screen_to_screen_x(vga: &VGA, src_start_x: usize, src_start_y: usize, src_end_x: usize, src_end_y: usize, dst_start_x: usize, dst_start_y: usize, src_page_base: usize, dst_page_base: usize, src_bitmap_width: usize, dst_bitmap_width: usize) {
	vga.set_gc_data(GCReg::BitMask, 0);
	
	let dst_page_width = dst_bitmap_width >> 2;
	let mut di = (dst_page_width * dst_start_y) + (dst_start_x >> 2) + dst_page_base;

	let src_page_width = src_bitmap_width >> 2;
	let mut si = (src_page_width * src_start_y) + (src_start_x >> 2) + src_page_base;

	let left_clip = LEFT_CLIP_PLANE_MASK[(src_start_x & 0x03) as usize];
	let right_clip = RIGHT_CLIP_PLANE_MASK[(src_end_x & 0x03) as usize];

	let width_bytes = src_end_x - src_start_x;
	let src_height = src_end_y - src_start_y;

	let src_next_offset =  src_page_width - width_bytes;
	let dst_next_offset = dst_page_width - width_bytes; 

	for _ in 0..src_height {
		vga.set_sc_data(SCReg::MapMask, left_clip);
		let _ = vga.read_mem(si);
		vga.write_mem(di, 0x00);
		si += 1;
		di += 1;

		vga.set_sc_data(SCReg::MapMask, 0x0F);
		for _ in 0..(width_bytes-1) {
			let _ = vga.read_mem(si);
			vga.write_mem(di, 0x00);
			si += 1;
			di += 1;
		}

		vga.set_sc_data(SCReg::MapMask, right_clip);
		let _ = vga.read_mem(si);
		vga.write_mem(di+width_bytes, 0x00);	
		si += 1;
		di += 1;

		si += src_next_offset;
		di += dst_next_offset;
	}
}

//TODO fix dst offset shifted by some pixel, why?
pub fn copy_system_to_screen_masked_x(vga: &VGA,
	src_start_x: usize, src_start_y: usize, src_end_x: usize, src_end_y: usize, 
	dst_start_x: usize, dst_start_y: usize, 
	source: &[u8], dst_page_base: usize, 
	src_bitmap_width: usize, dst_bitmap_width: usize,
    mask: &[u8]) {

		let dst_page_width = dst_bitmap_width >> 2;
		let mut di = (dst_page_width * dst_start_y) + (dst_start_x >> 2) + dst_page_base;
	
		let mut si = src_bitmap_width * src_start_y + src_start_x;

		let width_bytes = src_end_x - src_start_x;
		let src_height = src_end_y - src_start_y;

		for _ in 0..src_height {
			let mut ix = di & !0b11;
			let mut plane = di & 0b11;
			for _ in 0..width_bytes {
				if mask[si] != 0 {
					vga.set_sc_data(SCReg::MapMask, 1 << plane);
					vga.write_mem(ix, source[si]);
				}
				if plane == 3 {
					ix += 1;
					plane = 0;
				} else {
					plane += 1;
				}
				si += 1;
			}
			di += dst_page_width;
		}
}
