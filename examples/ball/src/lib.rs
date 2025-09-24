#![allow(clippy::comparison_chain)]

#[cfg(feature = "web")]
pub mod web;

/// Ball example from https://github.com/jagregory/abrash-black-book/blob/master/src/chapter-23.md
use vga::{AttributeReg, CRTReg, GCReg, SCReg, VGABuilder, util::sleep};

const LOGICAL_SCREEN_WIDTH: usize = 672 / 8; //width in bytes and height in scan
const LOGICAL_SCREEN_HEIGHT: usize = 384; //lines of the virtual screen we'll work with
const PAGE1: usize = 1; //flag for page 1 when page flipping
const PAGE0_OFFSET: usize = 0; //start offset of page 0 in VGA memory
const PAGE1_OFFSET: usize = LOGICAL_SCREEN_WIDTH * LOGICAL_SCREEN_HEIGHT; //start offset of page 1 (both pages are 672x384 virtual screens)
const BALL_WIDTH: usize = 24 / 8; //width of ball in display memory bytes
const BALL_HEIGHT: usize = 24; //height of ball in scan lines
const BLANK_OFFSET: usize = PAGE1_OFFSET * 2; //start of blank image in VGA memory
const BALL_OFFSET: usize = BLANK_OFFSET + (BALL_WIDTH * BALL_HEIGHT); //start offset of ball image in VGA memory
const NUM_BALLS: usize = 4;

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
const PANNING_CONTROL_STRING: [i16; 13] = [32, 1, 0, 34, 0, 1, 32, -1, 0, 34, 0, -1, 0];

struct PanningState {
    hpan: i16,
    panning_rep: i16,
    panning_x_inc: i16,
    panning_y_inc: i16,
    panning_start_offset: usize,
    panning_control: usize,
}
struct RenderState {
    ball_x: [usize; 4],
    ball_y: [usize; 4],
    last_ball_x: [usize; 4],
    last_ball_y: [usize; 4],
    ball_x_inc: [i16; 4],
    ball_y_inc: [i16; 4],
    ball_rep: [i16; 4],
    ball_control: [usize; 4],

    current_page: usize,
    current_page_offset: usize,

    panning_state: PanningState,
}

fn initial_panning_state() -> PanningState {
    PanningState {
        hpan: 0,
        panning_rep: 1,
        panning_x_inc: 1,
        panning_y_inc: 1,
        panning_start_offset: 0,
        panning_control: 0,
    }
}

fn initial_render_state() -> RenderState {
    RenderState {
        ball_x: [15, 50, 40, 70],
        ball_y: [40, 200, 110, 300],
        last_ball_x: [15, 50, 40, 70],
        last_ball_y: [40, 200, 110, 300],
        ball_x_inc: [1, 1, 1, 1],
        ball_y_inc: [8, 8, 8, 8],
        ball_rep: [1, 1, 1, 1],
        ball_control: [0, 0, 0, 0],
        current_page: PAGE1,
        current_page_offset: PAGE1_OFFSET,
        panning_state: initial_panning_state(),
    }
}

pub async fn start_ball() -> Result<(), String> {
    let mut vga = VGABuilder::new()
        .fullscreen(false)
        .simulate_vertical_reset()
        .build()?;

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

    let mut state = initial_render_state();

    //let vga = std::sync::Arc::new(vga);
    loop {
        for bx in (0..NUM_BALLS).rev() {
            draw_ball(
                &vga,
                BLANK_OFFSET,
                state.current_page_offset,
                state.last_ball_x[bx],
                state.last_ball_y[bx],
            );

            let mut ax = state.ball_x[bx];
            state.last_ball_x[bx] = ax;
            ax = state.ball_y[bx];
            state.last_ball_y[bx] = ax;

            state.ball_rep[bx] -= 1;
            if state.ball_rep[bx] == 0 {
                //repeat factor run out, reset it
                let mut bc_ptr = state.ball_control[bx];
                if BALL_CONTROL_STRING[bx][bc_ptr] == 0 {
                    bc_ptr = 0;
                }
                state.ball_rep[bx] = BALL_CONTROL_STRING[bx][bc_ptr];
                state.ball_x_inc[bx] = BALL_CONTROL_STRING[bx][bc_ptr + 1];
                state.ball_y_inc[bx] = BALL_CONTROL_STRING[bx][bc_ptr + 2];

                state.ball_control[bx] = bc_ptr + 3;
            }

            state.ball_x[bx] = (state.ball_x[bx] as i16 + state.ball_x_inc[bx]) as usize;
            state.ball_y[bx] = (state.ball_y[bx] as i16 + state.ball_y_inc[bx]) as usize;

            draw_ball(
                &vga,
                BALL_OFFSET,
                state.current_page_offset,
                state.ball_x[bx],
                state.ball_y[bx],
            );
        }

        adjust_panning(&mut state.panning_state);

        // Flip to new page by setting new start adress
        let addr_parts =
            (state.current_page_offset + state.panning_state.panning_start_offset).to_le_bytes();
        vga.set_crt_data(CRTReg::StartAdressLow, addr_parts[0]);
        vga.set_crt_data(CRTReg::StartAdressHigh, addr_parts[1]);

        vga.set_attribute_reg(
            AttributeReg::HorizontalPixelPanning,
            state.panning_state.hpan as u8,
        );

        // Flip pages for next loop
        state.current_page ^= 1;
        if state.current_page == 0 {
            state.current_page_offset = PAGE0_OFFSET;
        } else {
            state.current_page_offset = PAGE1_OFFSET
        }

        if vga.draw_frame() {
            return Ok(()); // quit
        }
        sleep(14).await; // target 70 fps
    }
}

fn draw_ball(vga: &vga::VGA, src_offset: usize, page_offset: usize, x: usize, y: usize) {
    let offset = page_offset + (y * LOGICAL_SCREEN_WIDTH + x);
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

fn adjust_panning(state: &mut PanningState) {
    state.panning_rep -= 1;
    if state.panning_rep <= 0 {
        let ax = PANNING_CONTROL_STRING[state.panning_control];
        if ax == 0 {
            //end of control string
            state.panning_control = 0;
        }
        state.panning_rep = PANNING_CONTROL_STRING[state.panning_control];
        state.panning_x_inc = PANNING_CONTROL_STRING[state.panning_control + 1];
        state.panning_y_inc = PANNING_CONTROL_STRING[state.panning_control + 2];
        state.panning_control += 3;
    }

    //horizontal pan
    if state.panning_x_inc < 0 {
        //pan left
        state.hpan -= 1;
        if state.hpan < 0 {
            state.hpan = 7;
            state.panning_start_offset -= 1;
        }
    } else if state.panning_x_inc > 0 {
        //pan right
        state.hpan += 1;
        if state.hpan >= 8 {
            state.hpan = 0;
            state.panning_start_offset += 1;
        }
    }

    //vertical pan
    if state.panning_y_inc < 0 {
        //pan up
        state.panning_start_offset -= LOGICAL_SCREEN_WIDTH;
    } else if state.panning_y_inc > 0 {
        //pan down
        state.panning_start_offset += LOGICAL_SCREEN_WIDTH;
    }
}
