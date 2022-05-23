use std::sync::Arc;
use std::thread::sleep;
use std::time::{Duration, Instant};

use sdl2::keyboard::Keycode;
use sdl2::event::Event;
use sdl2::pixels::{Color, PixelFormatEnum};
use sdl2::rect::Rect;
use sdl2::ttf;

use super::{
    is_linear, set_horizontal_display_end, set_vertical_display_end, AttributeReg, CRTReg,
    GeneralReg, VGA,
};
use super::input::{InputMonitoring, NumCode, Keyboard};

const CLEAR_VR_MASK: u8 = 0b11110111;
const CLEAR_DE_MASK: u8 = 0b11111110;
pub const TARGET_FRAME_RATE_MICRO: u128 = 1_000_000 / 60;
pub const VERTICAL_RESET_MICRO: u64 = 635;

const DEBUG_HEIGHT: usize = 20;
const FRAME_RATE_SAMPLES: usize = 100;

#[derive(Clone)]
pub struct Options {
    pub show_frame_rate: bool,
    pub start_addr_override: Option<usize>,
    pub input_monitoring: Option<InputMonitoring>,
}

impl Default for Options {
    fn default() -> Self {
        Options {
            show_frame_rate: false,
            //set in debug mode to ignore the start adress set in the vga
            start_addr_override: None,
            input_monitoring: None,
        }
    }
}

//Shows the full content of the VGA buffer as one big screen (for debugging) for
//the planar modes. width and height depends on your virtual screen size (640x819 if
//you did not change the default settings)
pub fn start_debug_planar_mode(
    vga: Arc<VGA>, w: usize, h: usize, options: Options,
) -> Result<(), String> {
    let mut debug_options = options;
    debug_options.start_addr_override = Some(0);

    set_horizontal_display_end(&vga, w as u32);
    set_vertical_display_end(&vga, h as u32);

    start(vga, debug_options)
}

pub fn start(vga: Arc<VGA>, options: Options) -> Result<(), String> {
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

    let vmode = vga.get_video_mode();
    let linear = is_linear(vmode);

    let w = get_width(&vga) as usize;
    let h = get_height(&vga) as usize;

    //TODO: inaccurate and currently a hack. This must be somehow inferred from the register states
    //but I haven't figured out how yet
    let v_stretch = if vmode == 0x13 { 2 } else { 1 };

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

    'running: loop {
        let mem_offset = mem_offset(&vga, &options);
        let frame_start = Instant::now();
        set_de(&vga, true); //display enable is currently only set for whole frame (not toggled for horizontal retrace)
        texture.with_lock(None, |buffer: &mut [u8], pitch: usize| {
            if linear {
                render_linear(&vga, mem_offset, offset_delta, h, v_stretch, buffer);
            } else {
                render_planar(&vga, mem_offset, offset_delta, h, buffer, pitch);
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

            update_inputs(&options.input_monitoring, event);
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

fn update_inputs(inputs: &Option<InputMonitoring>, event: Event) {
    if let Some(mon) = inputs {
        let state = &mut *mon.keyboard.lock().unwrap();
        match event {
            Event::KeyUp { keycode, .. } => clear_key(keycode, state),
            Event::KeyDown { keycode, .. } => set_key(keycode, state),
            _ => {} //ignore
        }
    }
}

fn clear_key(keycode: Option<Keycode>, state: &mut Keyboard) {
    if let Some(code) = keycode {
        state.buttons[to_num_code(code) as usize] = false;
    }
}

fn set_key(keycode: Option<Keycode>, state: &mut Keyboard) {
    if let Some(code) = keycode {
        state.buttons[to_num_code(code) as usize] = true;
    }
}

fn to_num_code(keycode: Keycode) -> NumCode {
    match keycode {
        Keycode::Backspace => return NumCode::BackSpace,
        Keycode::Tab => return NumCode::Tab,
        Keycode::Return => return NumCode::Return,
        Keycode::Escape => return NumCode::Escape,
        Keycode::Space => return NumCode::Space,
        Keycode::LAlt => return NumCode::Alt,
        Keycode::RAlt => return NumCode::Alt,
        Keycode::LCtrl => return NumCode::Control,
        Keycode::RCtrl => return NumCode::Control,
        Keycode::CapsLock => return NumCode::CapsLock,
        Keycode::LShift => return NumCode::LShift,
        Keycode::RShift => return NumCode::RShift,
        Keycode::Up => return NumCode::UpArrow,
        Keycode::Down => return NumCode::DownArrow,
        Keycode::Left => return NumCode::LeftArrow,
        Keycode::Right => return NumCode::RightArrow,
        Keycode::Insert => return NumCode::Insert,
        Keycode::Delete => return NumCode::Delete,
        Keycode::Home => return NumCode::Home,
        Keycode::End => return NumCode::End,
        Keycode::PageUp => return NumCode::PgUp,
        Keycode::PageDown => return NumCode::PgDn,
        Keycode::F1 => return NumCode::F1,
        Keycode::F2 => return NumCode::F2,
        Keycode::F3 => return NumCode::F3,
        Keycode::F4 => return NumCode::F4,
        Keycode::F5 => return NumCode::F5,
        Keycode::F6 => return NumCode::F6,
        Keycode::F7 => return NumCode::F7,
        Keycode::F8 => return NumCode::F8,
        Keycode::F9 => return NumCode::F9,
        Keycode::F10 => return NumCode::F10,
        Keycode::F11 => return NumCode::F11,
        Keycode::F12 => return NumCode::F12,
        Keycode::Num1 => return NumCode::Num1,
        Keycode::Num2 => return NumCode::Num2,
        Keycode::Num3 => return NumCode::Num3,
        Keycode::Num4 => return NumCode::Num4,
        Keycode::Num5 => return NumCode::Num5,
        Keycode::Num6 => return NumCode::Num6,
        Keycode::Num7 => return NumCode::Num7,
        Keycode::Num8 => return NumCode::Num8,
        Keycode::Num9 => return NumCode::Num9,
        Keycode::Num0 => return NumCode::Num0,
        Keycode::A => return NumCode::A,
        Keycode::B => return NumCode::B,
        Keycode::C => return NumCode::C,
        Keycode::D => return NumCode::D,
        Keycode::E => return NumCode::E,
        Keycode::F => return NumCode::F,
        Keycode::G => return NumCode::G,
        Keycode::H => return NumCode::H,
        Keycode::I => return NumCode::I,
        Keycode::J => return NumCode::J,
        Keycode::K => return NumCode::K,
        Keycode::L => return NumCode::L,
        Keycode::M => return NumCode::M,
        Keycode::N => return NumCode::N,
        Keycode::O => return NumCode::O,
        Keycode::P => return NumCode::P,
        Keycode::Q => return NumCode::Q,
        Keycode::R => return NumCode::R,
        Keycode::S => return NumCode::S,
        Keycode::T => return NumCode::T,
        Keycode::U => return NumCode::U,
        Keycode::V => return NumCode::V,
        Keycode::W => return NumCode::W,
        Keycode::X => return NumCode::X,
        Keycode::Y => return NumCode::Y,
        Keycode::Z => return NumCode::Z,
        _ => return NumCode::Bad,
    }
}

fn render_planar(
    vga: &VGA, mem_offset_p: usize, offset_delta: usize, h: usize, buffer: &mut [u8], pitch: usize,
) {
    let mut x: usize = 0;
    let mut y: usize = 0;
    let mut mem_offset = mem_offset_p;
    let max_scan = (vga.get_crt_data(CRTReg::MaximumScanLine) & 0x1F) as usize + 1;
    let w_bytes = vga.get_crt_data(CRTReg::HorizontalDisplayEnd) as usize + 2; //+1 for exclusive intervall, +1 for "overshot" with potential hpan

    for _ in 0..(h / max_scan) {
        for _ in 0..max_scan {
            let hpan = vga.get_attribute_reg(AttributeReg::HorizontalPixelPanning) & 0xF;
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
    vga: &VGA, mem_offset_p: usize, offset_delta: usize, h: usize, v_stretch: usize,
    buffer: &mut [u8],
) {
    let mut mem_offset = mem_offset_p;
    let max_scan = (vga.get_crt_data(CRTReg::MaximumScanLine) & 0x1F) as usize + 1;
    let w_bytes = vga.get_crt_data(CRTReg::HorizontalDisplayEnd) as usize + 1;

    let mut buffer_offset = 0;
    for _ in 0..((h / max_scan) as usize) {
        for _ in 0..max_scan {
            for x_byte in 0..w_bytes {
                for p in 0..4 {
                    let v = vga.raw_read_mem(p, mem_offset + x_byte);
                    let color = vga.get_color_palette_256(v as usize);
                    for _ in 0..v_stretch {
                        // each color part (RGB) contains the high-order 6 bit values. 
                        // To get a "real" RGB value for display the value have to shifted
                        // by 2 bits (otherwise the color will be dimmed)
                        buffer[buffer_offset] = ((color & 0xFF0000) >> 14) as u8;
                        buffer[buffer_offset + 1] = ((color & 0x00FF00) >> 6) as u8;
                        buffer[buffer_offset + 2] = ((color & 0x0000FF) << 2) as u8;
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

//Constructs the Vertical Display End from the register + offset register
pub fn get_vertical_display_end(vga: &VGA) -> u32 {
    let vde_lower = vga.get_crt_data(CRTReg::VerticalDisplayEnd);
    let overflow = vga.get_crt_data(CRTReg::Overflow);
    let vde_bit_8 = (overflow & 0b0000_0010) >> 1;
    let vde_bit_9 = (overflow & 0b0100_0000) >> 5;
    let vde_upper = vde_bit_8 | vde_bit_9;
    let vde = vde_lower as u32;
    vde | ((vde_upper as u32) << 8)
}

//width in pixel
pub fn get_width(vga: &VGA) -> u32 {
    (vga.get_crt_data(CRTReg::HorizontalDisplayEnd) as u32 + 1) * 8
}

pub fn get_height(vga: &VGA) -> u32 {
    get_vertical_display_end(&vga) + 1
}
