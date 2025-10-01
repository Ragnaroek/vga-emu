//Rectangle example from https://github.com/jagregory/abrash-black-book/blob/master/src/chapter-48.md (LISTING 48.2)

use std::sync::Arc;

use vga::util;
use vga::{set_vertical_display_end, SCReg, VGA};

static PATT_TABLE: [[u8; 16]; 16] = [
    [10, 0, 10, 0, 0, 10, 0, 10, 10, 0, 10, 0, 0, 10, 0, 10],
    [9, 0, 0, 0, 0, 9, 0, 0, 0, 0, 9, 0, 0, 0, 0, 9],
    [5, 0, 0, 0, 0, 0, 5, 0, 5, 0, 0, 0, 0, 0, 5, 0],
    [14, 0, 0, 14, 0, 14, 14, 0, 0, 14, 14, 0, 14, 0, 0, 14],
    [15, 15, 15, 1, 15, 15, 1, 1, 15, 1, 1, 1, 1, 1, 1, 1],
    [12, 12, 12, 12, 6, 6, 6, 12, 6, 6, 6, 12, 6, 6, 6, 12],
    [
        80, 80, 80, 80, 80, 80, 80, 80, 80, 80, 80, 80, 80, 80, 80, 15,
    ],
    [
        78, 78, 78, 78, 80, 80, 80, 80, 82, 82, 82, 82, 84, 84, 84, 84,
    ],
    [
        78, 80, 82, 84, 80, 82, 84, 78, 82, 84, 78, 80, 84, 78, 80, 82,
    ],
    [
        78, 80, 82, 84, 78, 80, 82, 84, 78, 80, 82, 84, 78, 80, 82, 84,
    ],
    [0, 1, 2, 3, 4, 5, 6, 7, 8, 9, 10, 11, 12, 13, 14, 15],
    [0, 1, 2, 3, 0, 1, 2, 3, 0, 1, 2, 3, 0, 1, 2, 3],
    [14, 14, 9, 9, 14, 9, 9, 14, 9, 9, 14, 14, 9, 14, 14, 9],
    [15, 8, 8, 8, 15, 15, 15, 8, 15, 15, 15, 8, 15, 8, 8, 8],
    [3, 3, 3, 3, 3, 7, 7, 3, 3, 7, 7, 3, 3, 3, 3, 3],
    [0, 0, 0, 0, 0, 64, 0, 0, 0, 0, 0, 0, 0, 0, 0, 89],
];

pub fn main() -> Result<(), String> {
    let (vga, handle) = VGA::setup(0x13, false)?;

    //enable Mode X
    let mem_mode = vga.get_sc_data(SCReg::MemoryMode);
    vga.set_sc_data(SCReg::MemoryMode, (mem_mode & !0x08) | 0x04); //turn off chain 4 & odd/even
    set_vertical_display_end(&vga, 480);

    for j in 0..4 {
        for i in 0..4 {
            util::fill_pattern_x(
                &vga,
                i * 80,
                j * 60,
                i * 80 + 80,
                j * 60 + 60,
                0,
                &PATT_TABLE[j * 4 + i],
            );
        }
    }

    let vga_m = Arc::new(vga);

    let options: vga::Options = vga::Options {
        show_frame_rate: true,
        ..Default::default()
    };
    let handle_ref = Arc::new(handle);
    vga_m.start(handle_ref, options)?;
    Ok(())
}
