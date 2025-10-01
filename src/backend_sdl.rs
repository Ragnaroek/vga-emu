use std::cell::{RefCell, RefMut};
use std::sync::atomic::Ordering;
use std::sync::{Arc, Mutex};
use std::thread::sleep;
use std::time::{Duration, Instant};

use sdl2::Sdl;
use sdl2::event::Event;
use sdl2::keyboard::Keycode;
use sdl2::pixels::{Color, PixelFormatEnum};
use sdl2::rect::Rect;
use sdl2::render::WindowCanvas;
use sdl2::ttf;
use sdl2::video::FullscreenType;

use crate::backend::{EmuInput, PixelBuffer, is_linear, mem_offset, render_linear, render_planar};
use crate::input::{InputMonitoring, NumCode};
use crate::util::{set_de, set_vr};
use crate::{
    CRTReg, DEBUG_HEIGHT, FRAME_RATE_SAMPLES, Options, TARGET_FRAME_RATE_MICRO,
    VERTICAL_RESET_MICRO, VGA, VGABuilder,
};

// A non-sendable Handle to VGA that can control a limited set of things
// on the main thread only.
pub struct VGAHandle {
    sdl_context: Sdl,
    canvas: RefCell<WindowCanvas>,
    width: usize,
    height: usize,
}

impl VGAHandle {
    pub fn set_fullscreen(&self, fullscreen: bool) {
        if fullscreen == false {
            let result = self
                .canvas
                .borrow_mut()
                .window_mut()
                .set_fullscreen(FullscreenType::True);
            if result.is_err() {
                println!("error enabling fullscreen: {:?}", result.err());
            }
        } else {
            let result = self
                .canvas
                .borrow_mut()
                .window_mut()
                .set_fullscreen(FullscreenType::Off);
            if result.is_err() {
                println!("error disabling fullscreen: {:?}", result.err());
            }
        }
    }

    #[inline]
    pub fn update_canvas<F>(&self, f: F) -> Result<(), String>
    where
        F: FnOnce(RefMut<'_, WindowCanvas>) -> Result<(), String>,
    {
        let canvas = self.canvas.borrow_mut();
        f(canvas)
    }
}

pub fn setup_sdl(width: usize, height: usize, builder: &VGABuilder) -> Result<VGAHandle, String> {
    let sdl_context = sdl2::init()?;
    let video_subsystem = sdl_context.video()?;

    let win_heigt = if builder.show_frame_rate {
        height + DEBUG_HEIGHT
    } else {
        height
    };
    let win_width = width;

    let mut window_builder = video_subsystem.window("VGA", win_width as u32, win_heigt as u32);
    window_builder.position_centered();
    if builder.fullscreen {
        window_builder.fullscreen();
    }

    let window = window_builder.build().map_err(|e| e.to_string())?;
    let mut canvas = window.into_canvas().build().map_err(|e| e.to_string())?;
    // the logical size is important for fullscreen upscaling
    canvas
        .set_logical_size(width as u32, height as u32)
        .map_err(|e| e.to_string())?;

    Ok(VGAHandle {
        sdl_context,
        canvas: RefCell::new(canvas),
        height: win_heigt,
        width: win_width,
    })
}

pub fn start_sdl(vga: Arc<VGA>, handle: Arc<VGAHandle>, options: Options) -> Result<(), String> {
    let w = handle.width as u32;
    let h = handle.height as u32;

    let ttf_context = ttf::init().map_err(|e| e.to_string())?;
    let mut event_pump = handle.sdl_context.event_pump()?;
    let texture_creator = handle.canvas.borrow().texture_creator();

    let mut texture = texture_creator
        .create_texture_streaming(PixelFormatEnum::RGB24, w, h)
        .map_err(|e| e.to_string())?;

    let offset_delta = vga.regs.get_crt_data(CRTReg::Offset) as usize;
    if offset_delta == 0 {
        return Err(format!("illegal CRT offset: {}", offset_delta));
    }

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

    let vmode = vga.regs.get_video_mode();
    let linear = is_linear(vmode);
    //TODO: inaccurate and currently a hack. This must be somehow inferred from the register states
    //but I haven't figured out how yet
    let v_stretch = if vmode == 0x13 { 2 } else { 1 };

    let mut fr_buffer_vsync = [0; FRAME_RATE_SAMPLES];
    let mut fr_ix_vsync = 0;
    let mut fr_sum_vsync = 0;
    let mut fr = 1;

    let mut fr_buffer = [0; FRAME_RATE_SAMPLES];
    let mut fr_ix = 0;
    let mut fr_sum = 0;
    let mut fr_vsync = 1;

    let mut fullscreen = false;
    let mut emu_input = EmuInput::new();

    'running: loop {
        let mem_offset = mem_offset(&vga, &options);
        let frame_start = Instant::now();

        set_de(&vga, true); //display enable is currently only set for whole frame (not toggled for horizontal retrace)
        texture.with_lock(None, |buffer: &mut [u8], pitch: usize| {
            if linear {
                render_linear(
                    &vga,
                    mem_offset,
                    offset_delta,
                    h as usize,
                    v_stretch,
                    buffer,
                );
            } else {
                render_planar(&vga, mem_offset, offset_delta, h as usize, buffer, pitch);
            }
        })?;

        handle.update_canvas(|mut canvas| {
            canvas.clear();
            canvas.copy(&texture, None, Some(Rect::new(0, 0, w as u32, h as u32)))
        })?;

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

            handle.update_canvas(|mut canvas| {
                canvas.copy(
                    &dbg_texture,
                    None,
                    Some(Rect::new(0, h as i32, 200, DEBUG_HEIGHT as u32)),
                )
            })?;
        }

        handle.update_canvas(|mut canvas| {
            canvas.present();
            Ok(())
        })?;

        set_de(&vga, false);

        for event in event_pump.poll_iter() {
            match event {
                Event::Quit { .. } => break 'running,
                Event::KeyUp { keycode, .. } => {
                    if keycode == Some(Keycode::LALT) {
                        emu_input.alt = false;
                    }
                    if keycode == Some(Keycode::F) {
                        emu_input.f = false;
                    }
                }
                Event::KeyDown { keycode, .. } => {
                    if keycode == Some(Keycode::LALT) {
                        emu_input.alt = true;
                    }
                    if keycode == Some(Keycode::F) {
                        emu_input.f = true;
                    }
                }
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

        toggle_fullscreen(&mut emu_input, &handle, &mut fullscreen);
    }

    Ok(())
}

fn toggle_fullscreen(emu_input: &mut EmuInput, handle: &Arc<VGAHandle>, fullscreen: &mut bool) {
    if emu_input.alt == true && emu_input.f == true {
        handle.set_fullscreen(*fullscreen);
        *fullscreen = !*fullscreen;
        emu_input.clear_keys();
    }
}

impl PixelBuffer for [u8] {
    // TODO Use dedicated SDLBuffer here instead of [u8]
    const PIXEL_WIDTH: usize = 3;
    fn set_rgb(&mut self, offset: usize, r: u8, g: u8, b: u8) {
        self[offset] = r;
        self[offset + 1] = g;
        self[offset + 2] = b;
    }
}

// Input Stuff

fn update_inputs(inputs: &Option<Arc<Mutex<InputMonitoring>>>, event: Event) {
    if let Some(mon) = inputs {
        let im = &mut *mon.lock().unwrap();
        match event {
            Event::KeyUp { keycode, .. } => clear_key(keycode, im),
            Event::KeyDown { keycode, .. } => set_key(keycode, im),
            _ => {} //ignore
        }
    }
}

fn clear_key(keycode: Option<Keycode>, state: &mut InputMonitoring) {
    if let Some(code) = keycode {
        let num_code = to_num_code(code);
        if num_code != NumCode::Bad {
            state.keyboard.buttons[num_code as usize] = false;
        }
    }
}

fn set_key(keycode: Option<Keycode>, state: &mut InputMonitoring) {
    if let Some(code) = keycode {
        let num_code = to_num_code(code);
        if num_code != NumCode::Bad {
            state.keyboard.buttons[num_code as usize] = true;
            state.keyboard.update_last_value(num_code);
        }
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
        Keycode::NumLockClear => return NumCode::NumLock,
        Keycode::ScrollLock => return NumCode::ScrollLock,
        Keycode::PrintScreen => return NumCode::PrintScreen,
        Keycode::Home => return NumCode::Home,
        Keycode::End => return NumCode::End,
        Keycode::PageUp => return NumCode::PgUp,
        Keycode::PageDown => return NumCode::PgDn,
        Keycode::Minus => return NumCode::Minus,
        Keycode::Equals => return NumCode::Equals,
        Keycode::LeftBracket => return NumCode::LeftBracket,
        Keycode::RightBracket => return NumCode::RightBracket,
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
