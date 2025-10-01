use std::sync::Arc;
use std::thread::sleep;
use std::time::{Duration, Instant};

use sdl2::event::Event;
use sdl2::keyboard::Keycode;
use sdl2::pixels::{Color, PixelFormatEnum};
use sdl2::rect::Rect;
use sdl2::ttf;

use super::{AttributeReg, CRTReg, GeneralReg, VGA};

const CLEAR_VR_MASK: u8 = 0b11110111;
const CLEAR_DE_MASK: u8 = 0b11111110;
pub const TARGET_FRAME_RATE_MICRO: u128 = 1_000_000 / 60;
pub const VERTICAL_RESET_MICRO: u64 = 635;

const DEBUG_HEIGHT: usize = 20;
const FRAME_RATE_SAMPLES: usize = 100;

#[derive(Clone, Copy)]
pub struct Options {
    pub show_frame_rate: bool,
    pub start_addr_override: Option<usize>,
}

impl Default for Options {
    fn default() -> Self {
        Options {
            show_frame_rate: false,
            //set in debug mode to ignore the start adress set in the vga
            start_addr_override: None,
        }
    }
}

//Shows the screen according to the VGA video mode
pub fn start(vga: Arc<VGA>, options: Options) -> Result<(), String> {
    let mode = vga.get_video_mode();
    if mode == 0x10 {
        start_video(vga, 640, 350, options, 1)
    } else if mode == 0x13 {
        start_video(vga, 640, 400, options, 2)
    } else {
        panic!("video mode {:x}h not implemented", vga.get_video_mode())
    }
}

//Shows the full content of the VGA buffer as one big screen (for debugging) for
//the planar modes. width and height depends on your virtual screen size (640x819 if
//you did not change the default settings)
pub fn start_debug_planar_mode(
    vga: Arc<VGA>,
    w: usize,
    h: usize,
    options: Options,
) -> Result<(), String> {
    let mut debug_options = options;
    debug_options.start_addr_override = Some(0);
    start_video(vga, w, h, debug_options, 0)
}

fn start_video(
    vga: Arc<VGA>,
    w: usize,
    h: usize,
    options: Options,
    v_stretch: usize,
) -> Result<(), String> {
    let sdl_context = sdl2::init()?;
    let video_subsystem = sdl_context.video()?;
    let ttf_context = ttf::init().map_err(|e| e.to_string())?;
    let font;
    if options.show_frame_rate {
        font = Some(
            ttf_context
                .load_font("font/Roboto-Regular.ttf", 24)
                .unwrap(),
        )
    } else {
        font = None
    }

    let mut fr_buffer_vsync = [0; FRAME_RATE_SAMPLES];
    let mut fr_ix_vsync = 0;
    let mut fr_sum_vsync = 0;
    let mut fr = 1;

    let mut fr_buffer = [0; FRAME_RATE_SAMPLES];
    let mut fr_ix = 0;
    let mut fr_sum = 0;
    let mut fr_vsync = 1;

    let window = video_subsystem
        .window(
            "VGA",
            w as u32,
            if options.show_frame_rate {
                h + DEBUG_HEIGHT
            } else {
                h
            } as u32,
        )
        .position_centered()
        .build()
        .map_err(|e| e.to_string())?;
    let mut canvas = window.into_canvas().build().map_err(|e| e.to_string())?;
    let mut event_pump = sdl_context.event_pump()?;
    let texture_creator = canvas.texture_creator();
    let mut texture = texture_creator
        .create_texture_streaming(PixelFormatEnum::RGB24, w as u32, h as u32)
        .map_err(|e| e.to_string())?;

    let offset_delta = vga.get_crt_data(CRTReg::Offset) as usize;
    if offset_delta == 0 {
        return Err(format!("illegal CRT offset: {}", offset_delta));
    }

    let vmode = vga.get_video_mode();

    'running: loop {
        let mem_offset = mem_offset(&vga, &options);
        let frame_start = Instant::now();
        set_de(&vga, true); //display enable is currently only set for whole frame (not toggled for horizontal retrace)
        texture.with_lock(None, |buffer: &mut [u8], pitch: usize| {
            if vmode == 0x13 {
                render_linear(&vga, mem_offset, offset_delta, h, v_stretch, buffer);
            } else {
                render_planar(
                    &vga,
                    mem_offset,
                    offset_delta,
                    w,
                    h,
                    buffer,
                    pitch,
                );
            }
        })?;
        set_de(&vga, false);

        canvas.clear();
        canvas.copy(&texture, None, Some(Rect::new(0, 0, w as u32, h as u32)))?;
        if options.show_frame_rate {
            let surface = font
                .as_ref()
                .unwrap()
                .render(&format!("DEBUG: {} FPS(VSYNC), {} FPS", fr_vsync, fr))
                .blended(Color::RGBA(0, 255, 255, 255))
                .map_err(|e| e.to_string())?;
            let dbg_texture = texture_creator
                .create_texture_from_surface(&surface)
                .map_err(|e| e.to_string())?;
            canvas.copy(
                &dbg_texture,
                None,
                Some(Rect::new(0, h as i32, 200, DEBUG_HEIGHT as u32)),
            )?;
        }

        canvas.present();

        for event in event_pump.poll_iter() {
            match event {
                Event::Quit { .. }
                | Event::KeyDown {
                    keycode: Some(Keycode::Escape),
                    ..
                } => break 'running,
                _ => {}
            }
        }
        //simulate vertical reset
        set_vr(&vga, true);
        sleep(Duration::from_micros(VERTICAL_RESET_MICRO));
        set_vr(&vga, false);

        let v_elapsed = frame_start.elapsed().as_micros();
        if v_elapsed < TARGET_FRAME_RATE_MICRO {
            sleep(Duration::from_micros(
                (TARGET_FRAME_RATE_MICRO - v_elapsed) as u64,
            ));
            fr_ix = (fr_ix + 1) % FRAME_RATE_SAMPLES;
            fr_sum -= fr_buffer[fr_ix];
            fr_buffer[fr_ix] = v_elapsed;
            fr_sum += v_elapsed;
            fr = 1_000_000 / (fr_sum / (FRAME_RATE_SAMPLES as u128));
        }
        let v_elapsed_vsync = frame_start.elapsed().as_micros();
        fr_ix_vsync = (fr_ix_vsync + 1) % FRAME_RATE_SAMPLES;
        fr_sum_vsync -= fr_buffer_vsync[fr_ix_vsync];
        fr_buffer_vsync[fr_ix_vsync] = v_elapsed_vsync;
        fr_sum_vsync += v_elapsed_vsync;
        fr_vsync = 1_000_000 / (fr_sum_vsync / (FRAME_RATE_SAMPLES as u128));
    }

    Ok(())
}

fn render_planar(
    vga: &VGA,
    mem_offset_p: usize,
    offset_delta: usize,
    w: usize,
    h: usize,
    buffer: &mut [u8],
    pitch: usize,
) {
    let mut x: usize = 0;
    let mut y: usize = 0;
    let mut mem_offset = mem_offset_p;
    let max_scan = (vga.get_crt_data(CRTReg::MaximumScanLine) & 0x1F) as usize + 1;

    for _ in 0..(h / max_scan) {
        for _ in 0..max_scan {
            let hpan = vga.get_attribute_reg(AttributeReg::HorizontalPixelPanning) & 0xF;
            let w_bytes = ((w / 8) as usize) + 1; //+1 for "overshot" with potential hpan
            for mem_byte in 0..w_bytes {
                let v0 = vga.raw_read_mem(0, mem_offset + mem_byte);
                let v1 = vga.raw_read_mem(1, mem_offset + mem_byte);
                let v2 = vga.raw_read_mem(2, mem_offset + mem_byte);
                let v3 = vga.raw_read_mem(3, mem_offset + mem_byte);

                let start = if mem_byte == 0 { hpan } else { 0 };
                let end = if mem_byte == w_bytes - 1 { hpan } else { 8 };
                for b in start..end {
                    let bx = (1 << (7 - b)) as u8;
                    let mut pixel = bit_x(v0, bx, 0);
                    pixel |= bit_x(v1, bx, 1);
                    pixel |= bit_x(v2, bx, 2);
                    pixel |= bit_x(v3, bx, 3);

                    let color = default_16_color(pixel);
                    let offset = y * pitch + x * 3;
                    buffer[offset] = color.r;
                    buffer[offset + 1] = color.g;
                    buffer[offset + 2] = color.b;
                    x += 1;
                }
            }
            x = 0;
            y += 1;
        }
        mem_offset += offset_delta * 2;
    }
}

fn render_linear(
    vga: &VGA,
    mem_offset_p: usize,
    offset_delta: usize,
    h: usize,
    v_stretch: usize,
    buffer: &mut [u8],
) {
    let mut mem_offset = mem_offset_p;
    let max_scan = (vga.get_crt_data(CRTReg::MaximumScanLine) & 0x1F) as usize + 1;

    let mut buffer_offset = 0;
    for y in 0..((h / max_scan) as usize) {
        for _ in 0..max_scan {
            for x_byte in 0..80 {
                for p in 0..4 {
                    let v = vga.raw_read_mem(p, mem_offset + x_byte);
                    let color = DEFAULT_256_COLORS[v as usize];
                    for _ in 0..v_stretch {
                        buffer[buffer_offset] = ((color & 0xFF0000) >> 16) as u8;
                        buffer[buffer_offset + 1] = ((color & 0x00FF00) >> 8) as u8;
                        buffer[buffer_offset + 2] = (color & 0x0000FF) as u8;
                        buffer_offset += 3;
                    }
                }
            }
        }
        mem_offset += offset_delta * 2;
    }
}

fn mem_offset(vga: &VGA, options: &Options) -> usize {
    if let Some(over) = options.start_addr_override {
        return over;
    }
    let low = vga.get_crt_data(CRTReg::StartAdressLow) as u16;
    let mut addr = vga.get_crt_data(CRTReg::StartAdressHigh) as u16;
    addr <<= 8;
    addr |= low;
    addr as usize
}

fn bit_x(v: u8, v_ix: u8, dst_ix: u8) -> u8 {
    if v & v_ix != 0 {
        1 << dst_ix
    } else {
        0
    }
}

//vertical retrace
fn set_vr(vga: &VGA, set: bool) {
    let v0 = vga.get_general_reg(GeneralReg::InputStatus1);
    if set {
        vga.set_general_reg(GeneralReg::InputStatus1, v0 | !CLEAR_VR_MASK);
    } else {
        vga.set_general_reg(GeneralReg::InputStatus1, v0 & CLEAR_VR_MASK);
    }
}

//display enable NOT
fn set_de(vga: &VGA, display_mode: bool) {
    let v0 = vga.get_general_reg(GeneralReg::InputStatus1);
    if display_mode {
        //flag needs to be set to zero (NOT)
        vga.set_general_reg(GeneralReg::InputStatus1, v0 & CLEAR_DE_MASK);
    } else {
        //not in display mode (vertical or horizontal retrace), set to 1
        vga.set_general_reg(GeneralReg::InputStatus1, v0 | !CLEAR_DE_MASK);
    }
}

fn default_16_color(v: u8) -> Color {
    //source: https://wasteland.fandom.com/wiki/EGA_Colour_Palette
    match v {
        0x00 => Color::RGB(0x0, 0x0, 0x0),
        0x01 => Color::RGB(0x0, 0x0, 0xA8),
        0x02 => Color::RGB(0x0, 0xA8, 0x0),
        0x03 => Color::RGB(0x0, 0xA8, 0xA8),
        0x04 => Color::RGB(0xA8, 0x0, 0x0),
        0x05 => Color::RGB(0xA8, 0x0, 0xA8),
        0x06 => Color::RGB(0xA8, 0x54, 0x00),
        0x07 => Color::RGB(0xA8, 0xA8, 0xA8),
        0x08 => Color::RGB(0x54, 0x54, 0x54),
        0x09 => Color::RGB(0x54, 0x54, 0xFE),
        0x0A => Color::RGB(0x54, 0xFE, 0x54),
        0x0B => Color::RGB(0x54, 0xFE, 0xFE),
        0x0C => Color::RGB(0xFE, 0x54, 0x54),
        0x0D => Color::RGB(0xFE, 0x54, 0xFE),
        0x0E => Color::RGB(0xFE, 0xFE, 0x54),
        0x0F => Color::RGB(0xFE, 0xFE, 0xFE),
        _ => panic!("wrong color index"),
    }
}

//taken from https://commons.wikimedia.org/wiki/User:Psychonaut/ipalette.sh
static DEFAULT_256_COLORS: [u32; 256] = [
    0x000000, 0x0000AA, 0x00AA00, 0x00AAAA, 0xAA0000, 0xAA00AA, 0xAA5500, 0xAAAAAA, 0x555555,
    0x5555FF, 0x55FF55, 0x55FFFF, 0xFF5555, 0xFF55FF, 0xFFFF55, 0xFFFFFF, 0x000000, 0x101010,
    0x202020, 0x353535, 0x454545, 0x555555, 0x656565, 0x757575, 0x8A8A8A, 0x9A9A9A, 0xAAAAAA,
    0xBABABA, 0xCACACA, 0xDFDFDF, 0xEFEFEF, 0xFFFFFF, 0x0000FF, 0x4100FF, 0x8200FF, 0xBE00FF,
    0xFF00FF, 0xFF00BE, 0xFF0082, 0xFF0041, 0xFF0000, 0xFF4100, 0xFF8200, 0xFFBE00, 0xFFFF00,
    0xBEFF00, 0x82FF00, 0x41FF00, 0x00FF00, 0x00FF41, 0x00FF82, 0x00FFBE, 0x00FFFF, 0x00BEFF,
    0x0082FF, 0x0041FF, 0x8282FF, 0x9E82FF, 0xBE82FF, 0xDF82FF, 0xFF82FF, 0xFF82DF, 0xFF82BE,
    0xFF829E, 0xFF8282, 0xFF9E82, 0xFFBE82, 0xFFDF82, 0xFFFF82, 0xDFFF82, 0xBEFF82, 0x9EFF82,
    0x82FF82, 0x82FF9E, 0x82FFBE, 0x82FFDF, 0x82FFFF, 0x82DFFF, 0x82BEFF, 0x829EFF, 0xBABAFF,
    0xCABAFF, 0xDFBAFF, 0xEFBAFF, 0xFFBAFF, 0xFFBAEF, 0xFFBADF, 0xFFBACA, 0xFFBABA, 0xFFCABA,
    0xFFDFBA, 0xFFEFBA, 0xFFFFBA, 0xEFFFBA, 0xDFFFBA, 0xCAFFBA, 0xBAFFBA, 0xBAFFCA, 0xBAFFDF,
    0xBAFFEF, 0xBAFFFF, 0xBAEFFF, 0xBADFFF, 0xBACAFF, 0x000071, 0x1C0071, 0x390071, 0x550071,
    0x710071, 0x710055, 0x710039, 0x71001C, 0x710000, 0x711C00, 0x713900, 0x715500, 0x717100,
    0x557100, 0x397100, 0x1C7100, 0x007100, 0x00711C, 0x007139, 0x007155, 0x007171, 0x005571,
    0x003971, 0x001C71, 0x393971, 0x453971, 0x553971, 0x613971, 0x713971, 0x713961, 0x713955,
    0x713945, 0x713939, 0x714539, 0x715539, 0x716139, 0x717139, 0x617139, 0x557139, 0x457139,
    0x397139, 0x397145, 0x397155, 0x397161, 0x397171, 0x396171, 0x395571, 0x394571, 0x515171,
    0x595171, 0x615171, 0x695171, 0x715171, 0x715169, 0x715161, 0x715159, 0x715151, 0x715951,
    0x716151, 0x716951, 0x717151, 0x697151, 0x617151, 0x597151, 0x517151, 0x517159, 0x517161,
    0x517169, 0x517171, 0x516971, 0x516171, 0x515971, 0x000041, 0x100041, 0x200041, 0x310041,
    0x410041, 0x410031, 0x410020, 0x410010, 0x410000, 0x411000, 0x412000, 0x413100, 0x414100,
    0x314100, 0x204100, 0x104100, 0x004100, 0x004110, 0x004120, 0x004131, 0x004141, 0x003141,
    0x002041, 0x001041, 0x202041, 0x282041, 0x312041, 0x392041, 0x412041, 0x412039, 0x412031,
    0x412028, 0x412020, 0x412820, 0x413120, 0x413920, 0x414120, 0x394120, 0x314120, 0x284120,
    0x204120, 0x204128, 0x204131, 0x204139, 0x204141, 0x203941, 0x203141, 0x202841, 0x2D2D41,
    0x312D41, 0x352D41, 0x3D2D41, 0x412D41, 0x412D3D, 0x412D35, 0x412D31, 0x412D2D, 0x41312D,
    0x41352D, 0x413D2D, 0x41412D, 0x3D412D, 0x35412D, 0x31412D, 0x2D412D, 0x2D4131, 0x2D4135,
    0x2D413D, 0x2D4141, 0x2D3D41, 0x2D3541, 0x2D3141, 0x000000, 0x000000, 0x000000, 0x000000,
    0x000000, 0x000000, 0x000000, 0x00000,
];
