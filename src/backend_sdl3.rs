use std::sync::{RwLock, RwLockWriteGuard};
use std::thread::sleep;
use std::time::Duration;

use sdl3::{
    EventPump,
    event::Event,
    keyboard::{Keycode, Mod},
    pixels::PixelFormat,
    render::{Canvas, Texture},
    sys::pixels::SDL_PixelFormat,
    sys::render::SDL_RendererLogicalPresentation,
    video::Window,
};

use crate::backend::{EmuInput, PixelBuffer, is_linear, render_linear, render_planar};
use crate::input::{InputMonitoring, MouseButton, NumCode};
use crate::util::{set_de, set_vr};
use crate::{CRTReg, VERTICAL_RESET_MICRO, VGABuilder, VGAEmu};

pub struct RenderContext {
    canvas: Canvas<Window>,
    texture: Texture,
    event_pump: EventPump,
    height: usize,
    fullscreen: bool,
    simulate_vertical_reset: bool,
    input_monitoring: RwLock<InputMonitoring>,
}

impl RenderContext {
    pub fn init(width: usize, height: usize, builder: VGABuilder) -> Result<RenderContext, String> {
        let sdl = sdl3::init().map_err(|e| e.to_string())?;

        let vid = sdl.video().map_err(|e| e.to_string())?;
        let event_pump = sdl.event_pump().map_err(|e| e.to_string())?;

        let mut window_builder = vid.window(&builder.title, width as u32, height as u32);
        window_builder.position_centered();
        if builder.fullscreen {
            window_builder.fullscreen();
        }

        let window = window_builder.build().map_err(|e| e.to_string())?;
        let mut canvas = window.into_canvas();
        // the logical size is important for fullscreen upscaling
        canvas
            .set_logical_size(
                width as u32,
                height as u32,
                SDL_RendererLogicalPresentation::LETTERBOX,
            )
            .map_err(|e| e.to_string())?;

        let texture_builder = canvas.texture_creator();
        let texture = texture_builder
            .create_texture_streaming(
                unsafe { PixelFormat::from_ll(SDL_PixelFormat::RGB24) },
                width as u32,
                height as u32,
            )
            .map_err(|e| e.to_string())?;

        Ok(RenderContext {
            canvas,
            texture,
            event_pump,
            height,
            fullscreen: builder.fullscreen,
            simulate_vertical_reset: builder.simulate_vertical_reset,
            input_monitoring: RwLock::new(InputMonitoring::new()),
        })
    }

    pub fn draw_frame(&mut self, vga: &mut VGAEmu) -> bool {
        let offset_delta = vga.regs.get_crt_data(CRTReg::Offset) as usize;
        if offset_delta == 0 {
            panic!("illegal CRT offset: {}", offset_delta);
        }

        let vmode = vga.regs.get_video_mode();
        let linear = is_linear(vmode);
        //TODO: inaccurate and currently a hack. This must be somehow inferred from the register states
        //but I haven't figured out how yet
        let v_stretch = if vmode == 0x13 { 2 } else { 1 };
        let mem_offset = vga.mem_offset();

        set_de(vga, true); //display enable is currently only set for whole frame (not toggled for horizontal retrace)
        self.texture
            .with_lock(None, |buffer: &mut [u8], pitch: usize| {
                if linear {
                    render_linear(
                        vga,
                        mem_offset,
                        offset_delta,
                        self.height,
                        v_stretch,
                        buffer,
                    );
                } else {
                    render_planar(vga, mem_offset, offset_delta, self.height, buffer, pitch);
                }
            })
            .expect("SDL texture lock");

        self.canvas.clear();
        self.canvas.copy(&self.texture, None, None).expect("copy");
        self.canvas.present();
        set_de(vga, false);

        let (emu_input, quit) = self.handle_keys();
        if quit {
            return true;
        }

        if self.simulate_vertical_reset {
            set_vr(vga, true);
            sleep(Duration::from_micros(VERTICAL_RESET_MICRO));
            set_vr(vga, false);
        }

        self.toggle_fullscreen(&emu_input);

        false
    }

    pub fn input_monitoring<'a>(&'a mut self) -> RwLockWriteGuard<'a, InputMonitoring> {
        self.input_monitoring
            .write()
            .expect("write lock to InputMonitoring")
    }

    fn handle_keys(&mut self) -> (EmuInput, bool) {
        let mut emu_input = EmuInput::new();
        let mut events = Vec::new();
        for event in self.event_pump.poll_iter() {
            match event {
                Event::Quit { .. } => return (emu_input, true),
                Event::KeyUp {
                    keycode, keymod, ..
                } => {
                    if keymod.contains(Mod::LALTMOD) {
                        emu_input.alt = false;
                    }
                    if keycode == Some(Keycode::F) {
                        emu_input.f = false;
                    }
                }
                Event::KeyDown {
                    keycode, keymod, ..
                } => {
                    if keymod.contains(Mod::LALTMOD) {
                        emu_input.alt = true;
                    }
                    if keycode == Some(Keycode::F) {
                        emu_input.f = true;
                    }
                }
                _ => {}
            }
            events.push(event);
        }

        for event in events {
            self.update_inputs(event);
        }

        (emu_input, false)
    }

    fn toggle_fullscreen(&mut self, emu_input: &EmuInput) {
        if emu_input.alt == true && emu_input.f == true {
            self.set_fullscreen(self.fullscreen);
            self.fullscreen = !self.fullscreen;
        }
    }

    pub fn set_fullscreen(&mut self, fullscreen: bool) {
        if fullscreen == false {
            let result = self.canvas.window_mut().set_fullscreen(true);
            if result.is_err() {
                println!("error enabling fullscreen: {:?}", result.err());
            }
        } else {
            let result = self.canvas.window_mut().set_fullscreen(false);
            if result.is_err() {
                println!("error disabling fullscreen: {:?}", result.err());
            }
        }
    }

    fn update_inputs(&mut self, event: Event) {
        match event {
            Event::KeyUp { keycode, .. } => self.clear_key(keycode),
            Event::KeyDown { keycode, .. } => self.set_key(keycode),
            Event::MouseButtonUp { mouse_btn, .. } => self.clear_mouse_button(mouse_btn),
            Event::MouseButtonDown { mouse_btn, .. } => self.set_mouse_button(mouse_btn),
            _ => {} //ignore
        }
    }

    fn clear_key(&mut self, keycode: Option<Keycode>) {
        if let Some(code) = keycode {
            let num_code = to_num_code(code);
            if num_code != NumCode::Bad {
                self.input_monitoring().keyboard.buttons[num_code as usize] = false;
            }
        }
    }

    fn clear_mouse_button(&mut self, sdl_mouse_btn: sdl3::mouse::MouseButton) {
        let button = to_mouse_button(sdl_mouse_btn);
        self.input_monitoring().mouse.buttons[button as usize] = false;
    }

    fn set_mouse_button(&mut self, sdl_mouse_btn: sdl3::mouse::MouseButton) {
        let button = to_mouse_button(sdl_mouse_btn);
        self.input_monitoring().mouse.buttons[button as usize] = true;
    }

    fn set_key(&mut self, keycode: Option<Keycode>) {
        if let Some(code) = keycode {
            let num_code = to_num_code(code);
            if num_code != NumCode::Bad {
                self.input_monitoring().keyboard.buttons[num_code as usize] = true;
                self.input_monitoring().keyboard.update_last_value(num_code);
            }
        }
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
        Keycode::_1 => return NumCode::Num1,
        Keycode::_2 => return NumCode::Num2,
        Keycode::_3 => return NumCode::Num3,
        Keycode::_4 => return NumCode::Num4,
        Keycode::_5 => return NumCode::Num5,
        Keycode::_6 => return NumCode::Num6,
        Keycode::_7 => return NumCode::Num7,
        Keycode::_8 => return NumCode::Num8,
        Keycode::_9 => return NumCode::Num9,
        Keycode::_0 => return NumCode::Num0,
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

fn to_mouse_button(sdl_mouse_btn: sdl3::mouse::MouseButton) -> MouseButton {
    match sdl_mouse_btn {
        sdl3::mouse::MouseButton::Left => MouseButton::Left,
        sdl3::mouse::MouseButton::Right => MouseButton::Right,
        sdl3::mouse::MouseButton::Middle => MouseButton::Middle,
        _ => MouseButton::None,
    }
}
