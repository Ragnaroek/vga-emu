use std::sync::atomic::Ordering;
use std::sync::Arc;
use std::thread::sleep;
use std::time::{Duration, Instant};

use sdl2::keyboard::Keycode;
use sdl2::event::Event;
use sdl2::pixels::{Color, PixelFormatEnum};
use sdl2::rect::Rect;
use sdl2::ttf;

use crate::input::{InputMonitoring, Keyboard, NumCode};
use crate::backend::{PixelBuffer, is_linear, render_linear, render_planar, mem_offset};
use crate::util::{get_width, get_height, set_de, set_vr};
use crate::{ CRTReg, VGA, Options, FRAME_RATE_SAMPLES, DEBUG_HEIGHT, TARGET_FRAME_RATE_MICRO, VERTICAL_RESET_MICRO };

pub fn start_sdl(vga: Arc<VGA>, options: Options) -> Result<(), String> {
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
        set_de(&vga, false);

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

        options.frame_count.fetch_add(1, Ordering::Relaxed);

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

impl PixelBuffer for [u8] { // TODO Use dedicated SDLBuffer here instead of [u8]
    const PIXEL_WIDTH : usize = 3;
    fn set_rgb(&mut self, offset: usize, r: u8, g: u8, b: u8) {
        self[offset] = r;
        self[offset + 1] = g;
        self[offset + 2] = b;
    }
}

// Input Stuff

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