//Example from https://www.phatcode.net/res/224/files/html/ch47/47-07.html (LISTING 47.6)
use std::{thread::sleep, time::Duration};

use vga::{SCReg, set_vertical_display_end};
use vga::{VGABuilder, util};

fn main() -> Result<(), String> {
    let mut vga = VGABuilder::new()
        .video_mode(0x13)
        .fullscreen(false)
        .build()?;

    //enable Mode X
    let mem_mode = vga.get_sc_data(SCReg::MemoryMode);
    vga.set_sc_data(SCReg::MemoryMode, (mem_mode & !0x08) | 0x04); //turn off chain 4 & odd/even
    set_vertical_display_end(&vga, 480);

    util::fill_rectangle_x(&vga, 0, 0, 320, 240, 0, 0);

    let mut j = 1;
    while j < 220 {
        let mut i = 1;
        while i < 300 {
            util::fill_rectangle_x(
                &vga,
                i,
                j,
                i + 20,
                j + 20,
                0,
                (((j / 21 * 15) + i / 21) & 0xFF) as u8,
            );
            i += 21;
        }
        j += 21;
    }

    loop {
        if vga.draw_frame() {
            return Ok(()); // quit
        }
        sleep(Duration::from_millis(14)); // target 70 fps
    }
}
