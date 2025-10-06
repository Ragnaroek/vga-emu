#[cfg(feature = "web")]
pub mod web;

//Kite example from https://github.com/jagregory/abrash-black-book/blob/master/src/chapter-49.md (LISTING 49.5)
use vga::{
    SCReg, VGABuilder, set_vertical_display_end,
    util::{
        copy_screen_to_screen_x, copy_system_to_screen_masked_x, fill_pattern_x, fill_rectangle_x,
        sleep,
    },
};

const SCREEN_WIDTH: usize = 320;
const SCREEN_HEIGHT: usize = 240;
const PAGE0_START_OFFSET: usize = 0;
const PAGE1_START_OFFSET: usize = (SCREEN_HEIGHT * SCREEN_WIDTH) / 4;
const BG_START_OFFSET: usize = (SCREEN_HEIGHT * SCREEN_WIDTH * 2) / 4;

static GREEN_AND_BROWN_PATTERN: [u8; 16] = [2, 6, 2, 6, 6, 2, 6, 2, 2, 6, 2, 6, 6, 2, 6, 2];
static PINE_TREE_PATTERN: [u8; 16] = [2, 2, 2, 2, 2, 6, 2, 6, 2, 2, 6, 2, 2, 2, 2, 2];
static BRICK_PATTERN: [u8; 16] = [6, 6, 7, 6, 7, 7, 7, 7, 7, 6, 6, 6, 7, 7, 7, 7];
static ROOF_PATTERN: [u8; 16] = [8, 8, 8, 7, 7, 7, 7, 7, 8, 8, 8, 7, 8, 8, 8, 7];

const SMOKE_WIDTH: usize = 7;
const SMOKE_HEIGHT: usize = 7;
static SMOKE_PIXELS: [u8; 49] = [
    0, 0, 15, 15, 15, 0, 0, 0, 7, 7, 15, 15, 15, 0, 8, 7, 7, 7, 15, 15, 15, 8, 7, 7, 7, 7, 15, 15,
    0, 8, 7, 7, 7, 7, 15, 0, 0, 8, 7, 7, 7, 0, 0, 0, 0, 8, 8, 0, 0,
];
static SMOKE_MASK: [u8; 49] = [
    0, 0, 1, 1, 1, 0, 0, 0, 1, 1, 1, 1, 1, 0, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1,
    1, 1, 1, 0, 1, 1, 1, 1, 1, 0, 0, 0, 1, 1, 1, 0, 0,
];

pub async fn start_kite() -> Result<(), String> {
    let mut vga = VGABuilder::new()
        .video_mode(0x13)
        .title("VGA Kite Example".to_string())
        .fullscreen(false)
        .build()?;

    //enable Mode X
    let mem_mode = vga.get_sc_data(SCReg::MemoryMode);
    vga.set_sc_data(SCReg::MemoryMode, (mem_mode & !0x08) | 0x04); //turn off chain 4 & odd/even
    set_vertical_display_end(&vga, 480);

    draw_background(&mut vga, BG_START_OFFSET);
    copy_screen_to_screen_x(
        &mut vga,
        0,
        0,
        SCREEN_WIDTH,
        SCREEN_HEIGHT,
        0,
        0,
        BG_START_OFFSET,
        PAGE0_START_OFFSET,
        SCREEN_WIDTH,
        SCREEN_WIDTH,
    );
    copy_screen_to_screen_x(
        &mut vga,
        0,
        0,
        SCREEN_WIDTH,
        SCREEN_HEIGHT,
        0,
        0,
        BG_START_OFFSET,
        PAGE1_START_OFFSET,
        SCREEN_WIDTH,
        SCREEN_WIDTH,
    );

    loop {
        // TODO kite animation

        if vga.draw_frame() {
            return Ok(()); // quit
        }
        sleep(14).await; // target 70 fps
    }
}

fn draw_background(vga: &mut vga::VGA, page_start: usize) {
    //cyan background
    fill_rectangle_x(vga, 0, 0, SCREEN_WIDTH, SCREEN_HEIGHT, page_start, 11);
    //brown plain
    fill_pattern_x(
        vga,
        0,
        160,
        SCREEN_WIDTH,
        SCREEN_HEIGHT,
        page_start,
        &GREEN_AND_BROWN_PATTERN,
    );
    //blue water
    fill_rectangle_x(
        vga,
        0,
        SCREEN_HEIGHT - 30,
        SCREEN_WIDTH,
        SCREEN_HEIGHT,
        page_start,
        1,
    );
    //brown mountain
    for i in 0..120 {
        fill_rectangle_x(
            vga,
            SCREEN_WIDTH / 2 - 30 - i,
            51 + i,
            SCREEN_WIDTH / 2 - 30 + i + 1,
            51 + i + 1,
            page_start,
            6,
        );
    }
    //yellow sun
    for i in 0..=21 {
        let tmp = (20.0 * 20.0 - (i * i) as f64 + 0.5).sqrt() as usize;
        fill_rectangle_x(
            vga,
            SCREEN_WIDTH - 25 - i,
            30 - tmp,
            SCREEN_WIDTH - 25 + i + 1,
            30 + tmp + 1,
            page_start,
            14,
        );
    }
    //green trees
    for i in (10..90).step_by(15) {
        for j in 0..20 {
            fill_pattern_x(
                vga,
                SCREEN_WIDTH / 2 + i - j / 3 - 15,
                i + j + 51,
                SCREEN_WIDTH / 2 + i + j / 3 - 15 + 1,
                i + j + 51 + 1,
                page_start,
                &PINE_TREE_PATTERN,
            );
        }
    }
    //brick house
    fill_pattern_x(vga, 265, 150, 295, 170, page_start, &BRICK_PATTERN);
    fill_pattern_x(vga, 265, 130, 270, 150, page_start, &BRICK_PATTERN);
    for i in 0..12 {
        fill_pattern_x(
            vga,
            280 - i * 2,
            138 + i,
            280 + i * 2 + 1,
            138 + i + 1,
            page_start,
            &ROOF_PATTERN,
        );
    }
    //draw smoke puffs
    for i in 0..4 {
        copy_system_to_screen_masked_x(
            vga,
            0,
            0,
            SMOKE_WIDTH,
            SMOKE_HEIGHT,
            264,
            110 - i * 20,
            &SMOKE_PIXELS,
            page_start,
            SMOKE_WIDTH,
            SCREEN_WIDTH,
            &SMOKE_MASK,
        );
    }
}
