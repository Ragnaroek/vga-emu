//Ball example from https://github.com/jagregory/abrash-black-book/blob/master/src/chapter-23.md
use std::sync::{Arc};
use std::thread;
use std::time::Duration;
use vga::screen;
use vga::{CRTReg, GCReg, SCReg, GeneralReg};

const LOGICAL_SCREEN_WIDTH: usize = 672 / 8; //width in bytes and height in scan
const LOGICAL_SCREEN_HEIGHT: usize = 384; //lines of the virtual screen we'll work with
const PAGE0: usize = 0; //flag for page 0 when page flipping
const PAGE1: usize = 1; //flag for page 1 when page flipping
const PAGE0_OFFSET: usize = 0; //start offset of page 0 in VGA memory
const PAGE1_OFFSET: usize = LOGICAL_SCREEN_WIDTH * LOGICAL_SCREEN_HEIGHT; //start offset of page 1 (both pages are 672x384 virtual screens)
const BALL_WIDTH: usize = 24 / 8; //width of ball in display memory bytes
const BALL_HEIGHT: usize = 24; //height of ball in scan lines
const BLANK_OFFSET: usize = PAGE1_OFFSET * 2; //start of blank image in VGA memory
const BALL_OFFSET: usize = BLANK_OFFSET + (BALL_WIDTH * BALL_HEIGHT); //start offset of ball image in VGA memory
const NUM_BALLS: usize = 4;

const VSYNC_MASK : u8 = 0x08;
const DE_MASK : u8 = 0x01;

const BALL_0_CONTROL: [i16; 13] = [10, 1, 4, 10, -1, 4, 10, -1, -4, 10, 1, -4, 0];
const BALL_1_CONTORL: [i16; 13] = [12, -1, 1, 28, -1, -1, 12, 1, -1, 28, 1, 1, 0];
const BALL_2_CONTORL: [i16; 13] = [20, 0, -1, 40, 0, 1, 20, 0, -1, 0, 0, 0, 0];
const BALL_3_CONTORL: [i16; 13] = [8, 1, 0, 52, -1, 0, 44, 1, 0, 0, 0, 0, 0];
const BALL_CONTROL_STRING: [[i16; 13]; 4] = [
    BALL_0_CONTROL,
    BALL_1_CONTORL,
    BALL_2_CONTORL,
    BALL_3_CONTORL,
];

pub fn main() {
    let mut vga = vga::new(0x10);

    draw_border(&mut vga, PAGE0_OFFSET);
    draw_border(&mut vga, PAGE1_OFFSET);

    let plane_1_data = vec![
        0x00, 0x3c, 0x00, 0x01, 0xff, 0x80, //
        0x07, 0xff, 0xe0, 0x0f, 0xff, 0xf0, //
        0x00, 0x00, 0x00, 0x00, 0x00, 0x00, //
        0x00, 0x00, 0x00, 0x00, 0x00, 0x00, //
        0x7f, 0xff, 0xfe, 0xff, 0xff, 0xff, //
        0xff, 0xff, 0xff, 0xff, 0xff, 0xff, //
        0x00, 0x00, 0x00, 0x00, 0x00, 0x00, //
        0x00, 0x00, 0x00, 0x00, 0x00, 0x00, //
        0x7f, 0xff, 0xfe, 0x3f, 0xff, 0xfc, //
        0x3f, 0xff, 0xfc, 0x1f, 0xff, 0xf8, //
        0x00, 0x00, 0x00, 0x00, 0x00, 0x00, //
        0x00, 0x00, 0x00, 0x00, 0x00, 0x00, //
    ];

    let plane_2_data = vec![
        0x00, 0x00, 0x00, 0x00, 0x00, 0x00, //
        0x00, 0x00, 0x00, 0x00, 0x00, 0x00, //
        0x1f, 0xff, 0xf8, 0x3f, 0xff, 0xfc, //
        0x3f, 0xff, 0xfc, 0x7f, 0xff, 0xfe, //
        0x7f, 0xff, 0xfe, 0xff, 0xff, 0xff, //
        0xff, 0xff, 0xff, 0xff, 0xff, 0xff, //
        0x00, 0x00, 0x00, 0x00, 0x00, 0x00, //
        0x00, 0x00, 0x00, 0x00, 0x00, 0x00, //
        0x00, 0x00, 0x00, 0x00, 0x00, 0x00, //
        0x00, 0x00, 0x00, 0x00, 0x00, 0x00, //
        0x0f, 0xff, 0xf0, 0x07, 0xff, 0xe0, //
        0x01, 0xff, 0x80, 0x00, 0x3c, 0x00, //
    ];

    let plane_3_data = vec![
        0x00, 0x00, 0x00, 0x00, 0x00, 0x00, //
        0x00, 0x00, 0x00, 0x00, 0x00, 0x00, //
        0x00, 0x00, 0x00, 0x00, 0x00, 0x00, //
        0x00, 0x00, 0x00, 0x00, 0x00, 0x00, //
        0x00, 0x00, 0x00, 0x00, 0x00, 0x00, //
        0x00, 0x00, 0x00, 0x00, 0x00, 0x00, //
        0xff, 0xff, 0xff, 0xff, 0xff, 0xff, //
        0xff, 0xff, 0xff, 0x7f, 0xff, 0xfe, //
        0x7f, 0xff, 0xfe, 0x3f, 0xff, 0xfc, //
        0x3f, 0xff, 0xfc, 0x1f, 0xff, 0xf8, //
        0x0f, 0xff, 0xf0, 0x07, 0xff, 0xe0, //
        0x01, 0xff, 0x80, 0x00, 0x3c, 0x00, //
    ];

    let plane_4_data = vec![
        0x00, 0x3c, 0x00, 0x01, 0xff, 0x80, //
        0x07, 0xff, 0xe0, 0x0f, 0xff, 0xf0, //
        0x1f, 0xff, 0xf8, 0x3f, 0xff, 0xfc, //
        0x3f, 0xff, 0xfc, 0x7f, 0xff, 0xfe, //
        0x7f, 0xff, 0xfe, 0xff, 0xff, 0xff, //
        0xff, 0xff, 0xff, 0xff, 0xff, 0xff, //
        0xff, 0xff, 0xff, 0xff, 0xff, 0xff, //
        0xff, 0xff, 0xff, 0x7f, 0xff, 0xfe, //
        0x7f, 0xff, 0xfe, 0x3f, 0xff, 0xfc, //
        0x3f, 0xff, 0xfc, 0x1f, 0xff, 0xf8, //
        0x0f, 0xff, 0xf0, 0x07, 0xff, 0xe0, //
        0x01, 0xff, 0x80, 0x00, 0x3c, 0x00, //
    ];

    //draw ball data to offscreen memory
    vga.set_sc_data(SCReg::MapMask, 0x01);
    vga.write_mem_chunk(BALL_OFFSET, &plane_1_data);
    vga.set_sc_data(SCReg::MapMask, 0x02);
    vga.write_mem_chunk(BALL_OFFSET, &plane_2_data);
    vga.set_sc_data(SCReg::MapMask, 0x04);
    vga.write_mem_chunk(BALL_OFFSET, &plane_3_data);
    vga.set_sc_data(SCReg::MapMask, 0x08);
    vga.write_mem_chunk(BALL_OFFSET, &plane_4_data);
    //blank image of ball to offscreen memory
    vga.set_sc_data(SCReg::MapMask, 0x0F);
    for i in 0..(BALL_WIDTH * BALL_HEIGHT) {
        vga.write_mem(BLANK_OFFSET + i, 0x00);
    }

    //set scan line width (in bytes)
    vga.set_crt_data(CRTReg::Offset, (LOGICAL_SCREEN_WIDTH / 2) as u8);

    //enable write mode 1
    let mut gc_mode = vga.get_gc_data(GCReg::GraphicsMode);
    gc_mode &= 0xFC;
    gc_mode |= 0x01;
    vga.set_gc_data(GCReg::GraphicsMode, gc_mode);

    let vga_m = Arc::new(vga);
    let vga_t = vga_m.clone();

    thread::spawn(move || {
        let mut ball_x = [15, 50, 40, 70];
        let mut ball_y = [40, 200, 110, 300];
        let mut last_ball_x = [15, 50, 40, 70];
        let mut last_ball_y = [40, 200, 110, 300];
        let mut ball_x_inc = [1, 1, 1, 1];
        let mut ball_y_inc = [8, 8, 8, 8];
        let mut ball_rep = [1, 1, 1, 1];
        let mut ball_control = [0, 0, 0, 0];

        loop {
            {
                for bx in (0..NUM_BALLS).rev() {
                    draw_ball(&vga_t, BLANK_OFFSET, last_ball_x[bx], last_ball_y[bx]);

                    let mut ax = ball_x[bx];
                    last_ball_x[bx] = ax;
                    ax = ball_y[bx];
                    last_ball_y[bx] = ax;

                    ball_rep[bx] -= 1;
                    if ball_rep[bx] == 0 {
                        //repeat factor run out, reset it
                        let mut bc_ptr = ball_control[bx];
                        if BALL_CONTROL_STRING[bx][bc_ptr] == 0 {
                            bc_ptr = 0;
                        }
                        ball_rep[bx] = BALL_CONTROL_STRING[bx][bc_ptr];
                        ball_x_inc[bx] = BALL_CONTROL_STRING[bx][bc_ptr + 1];
                        ball_y_inc[bx] = BALL_CONTROL_STRING[bx][bc_ptr + 2];

                        ball_control[bx] = bc_ptr + 3;
                    }

                    ball_x[bx] = (ball_x[bx] as i16 + ball_x_inc[bx]) as usize;
                    ball_y[bx] = (ball_y[bx] as i16 + ball_y_inc[bx]) as usize;

                    draw_ball(&vga_t, BALL_OFFSET, ball_x[bx], ball_y[bx]);
                }
            }

            //adjust_panning()
            wait_display_enable(&vga_t);
            //TODO Flip to new page
            wait_vsync(&vga_t);

            ::std::thread::sleep(Duration::new(0, 1_000_000_000u32 / 30));
        }
    });

    screen::start_debug_planar_mode(vga_m, 672, 780);
}

fn draw_ball(vga: &vga::VGA, src_offset: usize, x: usize, y: usize) {
    let offset = y * LOGICAL_SCREEN_WIDTH + x; //TODO add CurrentPageOffset (once frame buffers are implemented)
    let mut si = src_offset;
    let mut di = offset;
    for _ in 0..BALL_HEIGHT {
        let mut dix = di;
        for _ in 0..BALL_WIDTH {
            vga.read_mem(si);
            vga.write_mem(dix, 0x00);
            si += 1;
            dix += 1;
        }
        di += LOGICAL_SCREEN_WIDTH;
    }
}

fn draw_border(vga: &vga::VGA, offset: usize) {
    let mut di = offset;
    //left border
    for _ in 0..(LOGICAL_SCREEN_HEIGHT / 16) {
        vga.set_sc_data(SCReg::MapMask, 0x0c);
        draw_border_block(vga, di);
        di += LOGICAL_SCREEN_WIDTH * 8;
        vga.set_sc_data(SCReg::MapMask, 0x0e);
        draw_border_block(vga, di);
        di += LOGICAL_SCREEN_WIDTH * 8;
    }
    //right border
    di = offset + LOGICAL_SCREEN_WIDTH - 1;
    for _ in 0..(LOGICAL_SCREEN_HEIGHT / 16) {
        vga.set_sc_data(SCReg::MapMask, 0x0e);
        draw_border_block(vga, di);
        di += LOGICAL_SCREEN_WIDTH * 8;
        vga.set_sc_data(SCReg::MapMask, 0x0c);
        draw_border_block(vga, di);
        di += LOGICAL_SCREEN_WIDTH * 8;
    }
    //top border
    di = offset;
    for _ in 0..((LOGICAL_SCREEN_WIDTH - 2) / 2) {
        di += 1;
        vga.set_sc_data(SCReg::MapMask, 0x0e);
        draw_border_block(vga, di);
        di += 1;
        vga.set_sc_data(SCReg::MapMask, 0x0c);
        draw_border_block(vga, di);
    }
    //bottom border
    di = offset + (LOGICAL_SCREEN_HEIGHT - 8) * LOGICAL_SCREEN_WIDTH;
    for _ in 0..((LOGICAL_SCREEN_WIDTH - 2) / 2) {
        di += 1;
        vga.set_sc_data(SCReg::MapMask, 0x0e);
        draw_border_block(vga, di);
        di += 1;
        vga.set_sc_data(SCReg::MapMask, 0x0c);
        draw_border_block(vga, di);
    }
}

fn draw_border_block(vga: &vga::VGA, offset: usize) {
    let mut di = offset;
    for _ in 0..8 {
        vga.write_mem(di, 0xff);
        di += LOGICAL_SCREEN_WIDTH;
    }
}

fn wait_display_enable(vga: &Arc<vga::VGA>) {
    loop {
        let in1 = vga.get_general_reg(GeneralReg::InputStatus1);
        if in1 & DE_MASK == 0 {
            break;
        }
    } 
}

fn wait_vsync(vga: &Arc<vga::VGA>) {
    loop {
        let in1 = vga.get_general_reg(GeneralReg::InputStatus1);
        if in1 & VSYNC_MASK != 0 {
            break;
        }
    }
}