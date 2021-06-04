use std::sync::Arc;
use std::time::Duration;

use sdl2::event::Event;
use sdl2::keyboard::Keycode;
use sdl2::pixels::Color;
use sdl2::rect::Point;

use super::CRTReg;
use super::VGA;

//Shows the screen according to the VGA video mode
pub fn start(vga: Arc<VGA>) {
    if vga.video_mode == 0x10 {
        start_video(vga, 640, 350)
    } else {
        panic!("only video mode 0x10 implemented")
    }
}

//Shows the full content of the VGA buffer as one big screen (for debugging) for
//the planar modes
pub fn start_debug_planar_mode(vga: Arc<VGA>) {
    start_video(vga, 640, 820)
}

fn start_video(vga: Arc<VGA>, w: u32, h: u32) {
    let sdl_context = sdl2::init().unwrap();
    let video_subsystem = sdl_context.video().unwrap();

    let window = video_subsystem
        .window("VGA", w, h)
        .position_centered()
        .build()
        .unwrap();
    let mut canvas = window.into_canvas().build().unwrap();
    let mut event_pump = sdl_context.event_pump().unwrap();

    let offset_delta = vga.get_crt_data(CRTReg::Offset) as usize;
    if offset_delta <= 0 {
        panic!("illegal CRT offset: {}", offset_delta);
    }

    'running: loop {
        let mut mem_offset = 0;
        let mut x: usize = 0;
        let mut y: usize = 0;
        for _ in 0..(h as usize) {
            for mem_byte in 0..((w / 8) as usize) {
                let v1 = vga.mem[0][mem_offset + mem_byte];
                let v2 = vga.mem[1][mem_offset + mem_byte];
                let v3 = vga.mem[2][mem_offset + mem_byte];
                let v4 = vga.mem[3][mem_offset + mem_byte];

                for b in 0..8 {
                    let bx = (1 << b) as u8;
                    let mut c = (v1 & bx) << 4;
                    c |= (v2 & bx) << 3;
                    c |= (v3 & bx) << 2;
                    c |= v4 & bx;
                    canvas.set_draw_color(default_color(c));
                    canvas.draw_point(Point::new(x as i32, y as i32)).unwrap();
                    x += 1;
                }
            }
            x = 0;
            y += 1;
            mem_offset += offset_delta * 2;
        }

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

        canvas.present();
        ::std::thread::sleep(Duration::new(0, 1_000_000_000u32 / 60));
    }
}

fn default_color(v: u8) -> Color {
    return match v {
        0x00 => Color::RGB(0x0, 0x0, 0x0),
        0x01 => Color::RGB(0x0, 0x0, 0x99),
        0x02 => Color::RGB(0x0, 0x99, 0x0),
        0x03 => Color::RGB(0x0, 0x99, 0x99),
        0x04 => Color::RGB(0x99, 0x0, 0x0),
        0x05 => Color::RGB(0x99, 0x0, 0x99),
        0x06 => Color::RGB(0x99, 0x33, 0x00),
        0x07 => Color::RGB(0x99, 0x99, 0x99),
        0x08 => Color::RGB(0x66, 0x66, 0x66),
        0x09 => Color::RGB(0x33, 0x33, 0xFF),
        0x0A => Color::RGB(0x66, 0xFF, 0x33),
        0x0B => Color::RGB(0x66, 0xFF, 0xFF),
        0x0C => Color::RGB(0xCC, 0x33, 0x33),
        0x0D => Color::RGB(0xFF, 0x33, 0xFF),
        0x0E => Color::RGB(0xFF, 0xFF, 0x66),
        0x0F => Color::RGB(0xFF, 0xFF, 0xFF),
        _ => Color::RGB(0, 0, 0),
    };
}
