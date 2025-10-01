use async_std::task;
use std::sync::{Arc, Mutex};
use std::time::Duration;

use wasm_bindgen::prelude::*;
use wasm_bindgen::Clamped;
use wasm_bindgen_futures::spawn_local;

use crate::backend::{is_linear, mem_offset, render_linear, render_planar, PixelBuffer};
use crate::input::{InputMonitoring, NumCode};
use crate::util::{get_height, get_width, set_de, set_vr};
use crate::CRTReg;
use crate::TARGET_FRAME_RATE_MICRO;
use crate::VERTICAL_RESET_MICRO;
use crate::{Options, VGA};

struct WebBuffer {
    data: Vec<u8>,
}

pub fn start_web(vga: Arc<VGA>, options: Options) -> Result<(), String> {
    let document = web_sys::window().unwrap().document().unwrap();

    if let Some(ref mon) = options.input_monitoring {
        let mon_down = mon.clone();
        let mon_up = mon.clone();
        let keydown_handler: Closure<dyn Fn(_)> =
            Closure::wrap(Box::new(move |e: web_sys::Event| {
                handle_key(false, mon_down.clone(), e)
            }));
        let keyup_handler: Closure<dyn Fn(_)> =
            Closure::wrap(Box::new(move |e: web_sys::Event| {
                handle_key(true, mon_up.clone(), e)
            }));
        document
            .add_event_listener_with_callback("keydown", keydown_handler.as_ref().unchecked_ref())
            .expect("add keydown event");
        document
            .add_event_listener_with_callback("keyup", keyup_handler.as_ref().unchecked_ref())
            .expect("add keyup event");
        keydown_handler.forget();
        keyup_handler.forget();
    }

    let canvas = document
        .get_element_by_id("vga")
        .expect("canvas element with id 'vga' not found");
    let canvas: web_sys::HtmlCanvasElement = canvas
        .dyn_into::<web_sys::HtmlCanvasElement>()
        .map_err(|_| ())
        .unwrap();

    // TODO vmode, linear, w, h, v_stretch, offset_delta => provide init function for this and remove
    // code duplication in web and sdl impl!
    let vmode = vga.get_video_mode();
    let linear = is_linear(vmode);

    let w = get_width(&vga);
    let h = get_height(&vga);

    //TODO: inaccurate and currently a hack. This must be somehow inferred from the register states
    //but I haven't figured out how yet
    let v_stretch = if vmode == 0x13 { 2 } else { 1 };

    canvas.set_width(w);
    canvas.set_height(h);

    let ctx = canvas
        .get_context("2d")
        .unwrap()
        .unwrap()
        .dyn_into::<web_sys::CanvasRenderingContext2d>()
        .unwrap();

    let offset_delta = vga.get_crt_data(CRTReg::Offset) as usize;
    if offset_delta == 0 {
        return Err(format!("illegal CRT offset: {}", offset_delta));
    }

    let mut buffer = WebBuffer {
        data: vec![0; (w * h * 4) as usize],
    };
    spawn_local(async move {
        loop {
            let mem_offset = mem_offset(&vga, &options);
            let frame_start = js_sys::Date::now();

            for x in 0..100 {
                for y in 0..100 {
                    let red = ((y * w * 4) + x * 4) as usize;
                    buffer.data[red] = 128;
                    buffer.data[red + 1] = 128;
                    buffer.data[red + 2] = 128;
                    buffer.data[red + 3] = 255;
                }
            }

            set_de(&vga, true);
            task::sleep(Duration::ZERO).await;

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
                    &vga,
                    mem_offset,
                    offset_delta,
                    h as usize,
                    &mut buffer,
                    w as usize * WebBuffer::PIXEL_WIDTH,
                );
            }

            let image_data =
                web_sys::ImageData::new_with_u8_clamped_array(Clamped(&buffer.data), w)
                    .expect("image data");
            ctx.put_image_data(&image_data, 0.0, 0.0)
                .expect("put image data");
            ctx.begin_path();

            set_de(&vga, false);
            task::sleep(Duration::ZERO).await;

            set_vr(&vga, true);
            task::sleep(Duration::from_micros(VERTICAL_RESET_MICRO)).await;
            set_vr(&vga, false);
            task::sleep(Duration::ZERO).await;

            let v_elapsed = (js_sys::Date::now() - frame_start) as u128 * 1000;
            if v_elapsed < TARGET_FRAME_RATE_MICRO {
                task::sleep(Duration::from_micros(
                    (TARGET_FRAME_RATE_MICRO - v_elapsed) as u64,
                ))
                .await;
            } else {
                web_sys::console::log_1(&format!("frame miss!: {}", v_elapsed).into());
            }
        }
    });
    Ok(())
}

fn handle_key(up: bool, input: Arc<Mutex<InputMonitoring>>, event: web_sys::Event) {
    let keyboard_event = event
        .dyn_into::<web_sys::KeyboardEvent>()
        .expect("a KeyboardEvent");
    let key = to_num_code(&keyboard_event.key());
    let mut mon = input.lock().expect("keyboard lock");
    mon.keyboard.buttons[key as usize] = !up;
}

fn to_num_code(key: &str) -> NumCode {
    match key {
        "ArrowUp" => NumCode::UpArrow,
        "ArrowDown" => NumCode::DownArrow,
        "ArrowLeft" => NumCode::LeftArrow,
        "ArrowRight" => NumCode::RightArrow,
        "Control" => NumCode::Control,
        " " => NumCode::Space,
        _ => NumCode::Bad,
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
