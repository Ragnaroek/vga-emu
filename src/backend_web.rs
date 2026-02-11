use std::sync::{Arc, RwLock, RwLockWriteGuard};

use wasm_bindgen::Clamped;
use wasm_bindgen::prelude::*;
use web_sys::CanvasRenderingContext2d;

use crate::backend::{PixelBuffer, is_linear, render_linear, render_planar};
use crate::input::{InputMonitoring, NumCode};
use crate::util::{get_height, get_width, set_de};
use crate::{CRTReg, VGABuilder, VGAEmu};

pub struct RenderContext {
    ctx: CanvasRenderingContext2d,
    input_monitoring: Arc<RwLock<InputMonitoring>>,
}

struct WebBuffer {
    data: Vec<u8>,
}

impl RenderContext {
    pub fn init(
        width: usize, height: usize, _builder: VGABuilder,
    ) -> Result<RenderContext, String> {
        let document = web_sys::window().unwrap().document().unwrap();

        let canvas = document
            .get_element_by_id("vga")
            .expect("canvas element with id 'vga' not found");
        let canvas: web_sys::HtmlCanvasElement = canvas
            .dyn_into::<web_sys::HtmlCanvasElement>()
            .map_err(|_| ())
            .unwrap();

        canvas.set_width(width as u32);
        canvas.set_height(height as u32);

        let ctx = canvas
            .get_context("2d")
            .unwrap()
            .unwrap()
            .dyn_into::<web_sys::CanvasRenderingContext2d>()
            .unwrap();

        // setup input monitoring

        let input_monitoring = Arc::new(RwLock::new(InputMonitoring::new()));

        let mon_down = input_monitoring.clone();
        let mon_up = input_monitoring.clone();
        let keydown_handler: Closure<dyn Fn(_)> =
            Closure::wrap(Box::new(move |e: web_sys::Event| {
                handle_key(false, mon_down.clone(), e)
            }));
        let keyup_handler: Closure<dyn Fn(_)> =
            Closure::wrap(Box::new(move |e: web_sys::Event| {
                handle_key(true, mon_up.clone(), e)
            }));
        canvas
            .add_event_listener_with_callback("keydown", keydown_handler.as_ref().unchecked_ref())
            .expect("add keydown event");
        canvas
            .add_event_listener_with_callback("keyup", keyup_handler.as_ref().unchecked_ref())
            .expect("add keyup event");
        keydown_handler.forget();
        keyup_handler.forget();

        Ok(RenderContext {
            ctx,
            input_monitoring: input_monitoring,
        })
    }

    pub fn draw_frame(&self, vga: &mut VGAEmu) -> bool {
        let w = get_width(&vga);
        let h = get_height(&vga);

        // TODO vmode, linear, w, h, v_stretch, offset_delta => provide init function for this and remove
        // code duplication in web and sdl impl!
        let vmode = vga.get_video_mode();
        let linear = is_linear(vmode);

        //TODO: inaccurate and currently a hack. This must be somehow inferred from the register states
        //but I haven't figured out how yet
        let v_stretch = if vmode == 0x13 { 2 } else { 1 };

        let offset_delta = vga.regs.get_crt_data(CRTReg::Offset) as usize;
        if offset_delta == 0 {
            panic!("illegal CRT offset: {}", offset_delta); // TODO don't panic? it is not nice
        }

        let mut buffer = WebBuffer {
            data: vec![0; (w * h * 4) as usize],
        };

        let mem_offset = vga.mem_offset();

        for x in 0..100 {
            for y in 0..100 {
                let red = ((y * w * 4) + x * 4) as usize;
                buffer.data[red] = 128;
                buffer.data[red + 1] = 128;
                buffer.data[red + 2] = 128;
                buffer.data[red + 3] = 255;
            }
        }

        set_de(vga, true);

        if linear {
            render_linear(
                &vga,
                mem_offset,
                offset_delta,
                h as usize,
                v_stretch,
                &mut buffer,
            );
        } else {
            render_planar(
                vga,
                mem_offset,
                offset_delta,
                h as usize,
                &mut buffer,
                w as usize * WebBuffer::PIXEL_WIDTH,
            );
        }

        let image_data = web_sys::ImageData::new_with_u8_clamped_array(Clamped(&buffer.data), w)
            .expect("image data");
        self.ctx
            .put_image_data(&image_data, 0.0, 0.0)
            .expect("put image data");
        self.ctx.begin_path();

        false
    }

    pub fn input_monitoring<'a>(&'a mut self) -> RwLockWriteGuard<'a, InputMonitoring> {
        self.input_monitoring
            .write()
            .expect("write access to InputMonitoring")
    }
}

fn handle_key(up: bool, input: Arc<RwLock<InputMonitoring>>, event: web_sys::Event) {
    let keyboard_event = event
        .dyn_into::<web_sys::KeyboardEvent>()
        .expect("a KeyboardEvent");
    let (key, shift) = to_num_code(&keyboard_event.key());
    if key != NumCode::Bad {
        let mut im = input.write().expect("input write");
        im.set_key_pressed(NumCode::LShift, shift);
        im.set_key_pressed(key, !up);
        if !up {
            im.keyboard.update_last_value(key);
        }
    }
}

fn to_num_code(key: &str) -> (NumCode, bool) {
    match key {
        "Backspace" => (NumCode::BackSpace, false),
        "Tab" => (NumCode::Tab, false),
        "Enter" => (NumCode::Return, false),
        "Escape" => (NumCode::Escape, false),
        " " => (NumCode::Space, false),
        "Alt" => (NumCode::Alt, false),
        "AltGraph" => (NumCode::Alt, false),
        "Control" => (NumCode::Control, false),
        "CapsLock" => (NumCode::CapsLock, false),
        "Shift" => (NumCode::LShift, false),
        "ArrowUp" => (NumCode::UpArrow, false),
        "ArrowDown" => (NumCode::DownArrow, false),
        "ArrowLeft" => (NumCode::LeftArrow, false),
        "ArrowRight" => (NumCode::RightArrow, false),
        "Insert" => (NumCode::Insert, false),
        "Delete" => (NumCode::Delete, false),
        "NumLock" => (NumCode::NumLock, false),
        "ScrollLock" => (NumCode::ScrollLock, false),
        "PrintScreen" => (NumCode::PrintScreen, false),
        "Home" => (NumCode::Home, false),
        "End" => (NumCode::End, false),
        "PageUp" => (NumCode::PgUp, false),
        "PageDown" => (NumCode::PgDn, false),
        "-" => (NumCode::Minus, false),
        "+" => (NumCode::Plus, false),
        "=" => (NumCode::Equals, false),
        "[" => (NumCode::LeftBracket, false),
        "]" => (NumCode::RightBracket, false),
        "a" => (NumCode::A, false),
        "A" => (NumCode::A, true),
        "b" => (NumCode::B, false),
        "B" => (NumCode::B, true),
        "c" => (NumCode::C, false),
        "C" => (NumCode::C, true),
        "d" => (NumCode::D, false),
        "D" => (NumCode::D, true),
        "e" => (NumCode::E, false),
        "E" => (NumCode::E, true),
        "f" => (NumCode::F, false),
        "F" => (NumCode::F, true),
        "g" => (NumCode::G, false),
        "G" => (NumCode::G, true),
        "h" => (NumCode::H, false),
        "H" => (NumCode::H, true),
        "i" => (NumCode::I, false),
        "I" => (NumCode::I, true),
        "j" => (NumCode::J, false),
        "J" => (NumCode::J, true),
        "k" => (NumCode::K, false),
        "K" => (NumCode::K, true),
        "l" => (NumCode::L, false),
        "L" => (NumCode::L, true),
        "m" => (NumCode::M, false),
        "M" => (NumCode::M, true),
        "n" => (NumCode::N, false),
        "N" => (NumCode::N, true),
        "o" => (NumCode::O, false),
        "O" => (NumCode::O, true),
        "p" => (NumCode::P, false),
        "P" => (NumCode::P, true),
        "q" => (NumCode::Q, false),
        "Q" => (NumCode::Q, true),
        "r" => (NumCode::R, false),
        "R" => (NumCode::R, true),
        "s" => (NumCode::S, false),
        "S" => (NumCode::S, true),
        "t" => (NumCode::T, false),
        "T" => (NumCode::T, true),
        "u" => (NumCode::U, false),
        "U" => (NumCode::U, true),
        "v" => (NumCode::V, false),
        "V" => (NumCode::V, true),
        "w" => (NumCode::W, false),
        "W" => (NumCode::W, true),
        "x" => (NumCode::X, false),
        "X" => (NumCode::X, true),
        "y" => (NumCode::Y, false),
        "Y" => (NumCode::Y, true),
        "z" => (NumCode::Z, false),
        "Z" => (NumCode::Z, true),
        "0" => (NumCode::Num0, false),
        "1" => (NumCode::Num1, false),
        "2" => (NumCode::Num2, false),
        "3" => (NumCode::Num3, false),
        "4" => (NumCode::Num4, false),
        "5" => (NumCode::Num5, false),
        "6" => (NumCode::Num6, false),
        "7" => (NumCode::Num7, false),
        "8" => (NumCode::Num8, false),
        "9" => (NumCode::Num9, false),
        "F1" => (NumCode::F1, false),
        "F2" => (NumCode::F2, false),
        "F3" => (NumCode::F3, false),
        "F4" => (NumCode::F4, false),
        "F5" => (NumCode::F5, false),
        "F6" => (NumCode::F6, false),
        "F7" => (NumCode::F7, false),
        "F8" => (NumCode::F8, false),
        "F9" => (NumCode::F9, false),
        "F10" => (NumCode::F10, false),
        "F11" => (NumCode::F11, false),
        "F12" => (NumCode::F12, false),
        _ => (NumCode::Bad, false),
    }
}

impl PixelBuffer for WebBuffer {
    const PIXEL_WIDTH: usize = 4;
    fn set_rgb(&mut self, offset: usize, r: u8, g: u8, b: u8) {
        self.data[offset] = r;
        self.data[offset + 1] = g;
        self.data[offset + 2] = b;
        self.data[offset + 3] = 255;
    }
}
