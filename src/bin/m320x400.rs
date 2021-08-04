//Example from https://www.phatcode.net/res/224/files/html/ch31/31-03.html (LISTING 31.3)

use std::sync::Arc;

use vga::screen;
use vga::{CRTReg, GCReg, GeneralReg, SCReg, AttributeReg};

const SCREEN_WIDTH : usize = 80;

pub fn main() {

	let vga = vga::new(0x13);

	//set 320x400 mode
	let mem_mode = vga.get_sc_data(SCReg::MemoryMode);	
	vga.set_sc_data(SCReg::MemoryMode, (mem_mode & !0x08)| 0x04); //turn off chain 4 & odd/even

	let gc_mode = vga.get_gc_data(GCReg::GraphicsMode);
	vga.set_gc_data(GCReg::GraphicsMode, gc_mode & !0x10); //turn off odd/even

	let gc_misc = vga.get_gc_data(GCReg::MiscGraphics);
	vga.set_gc_data(GCReg::MiscGraphics, gc_misc & !0x02); //turn off chain

	let max_scan = vga.get_crt_data(CRTReg::MaximumScanLine);
	vga.set_crt_data(CRTReg::MaximumScanLine, max_scan & !0x1F); 

	let underline = vga.get_crt_data(CRTReg::UnderlineLocation);
	vga.set_crt_data(CRTReg::UnderlineLocation, underline & !0x40); //turn off doubleword

	let crt_mode = vga.get_crt_data(CRTReg::CRTCModeControl);
	vga.set_crt_data(CRTReg::CRTCModeControl, crt_mode & 0x40); //turn on byte mode bit

	vga.set_sc_data(SCReg::MapMask, 0x0F);
	
	//clear display memory
	for cx in 0..vga::PLANE_SIZE {
		vga.write_mem(cx, 0);
	}

	//TODO How does reg state exactly control rendering? Which settings are importan => chain?
}