use std::sync::{Arc,RwLock};
use std::time::{Duration, Instant};
use std::thread::sleep;

use sdl2::event::Event;
use sdl2::keyboard::Keycode;
use sdl2::pixels::Color;
use sdl2::rect::Point;

use super::{VGA, CRTReg, GeneralReg};

const CLEAR_VR_MASK : u8 = 0b11110111;
const CLEAR_DE_MASK : u8 = 1;
const TARGET_FRAME_RATE_MICRO: u128 = 1000_000/60;

//Shows the screen according to the VGA video mode
pub fn start(vga: Arc<RwLock<VGA>>) {
    if vga.read().unwrap().video_mode == 0x10 {
        start_video(vga, 640, 350)
    } else {
        panic!("only video mode 0x10 implemented")
    }
}

//Shows the full content of the VGA buffer as one big screen (for debugging) for
//the planar modes. width and height depends on your virtual screen size (640x819 if
//you did not change the default settings)
pub fn start_debug_planar_mode(vga: Arc<RwLock<VGA>>, w: u32, h: u32) {
    start_video(vga, w, h)
}

fn start_video(vga_lock: Arc<RwLock<VGA>>, w: u32, h: u32) {
    let sdl_context = sdl2::init().unwrap();
    let video_subsystem = sdl_context.video().unwrap();

    let window = video_subsystem
        .window("VGA", w, h)
        .position_centered()
        .build()
        .unwrap();
    let mut canvas = window.into_canvas().build().unwrap();
    let mut event_pump = sdl_context.event_pump().unwrap();

    let offset_delta = vga_lock.read().unwrap().get_crt_data(CRTReg::Offset) as usize;
    if offset_delta <= 0 {
        panic!("illegal CRT offset: {}", offset_delta);
    }

    'running: loop {
        let mut mem_offset = 0;
        let mut x: usize = 0;
        let mut y: usize = 0;
        let frame_start = Instant::now();
        //set VR to 0
        set_vr(&vga_lock, false);
        for _ in 0..(h as usize) {
            //set DE to 0
            let h_start = Instant::now();
            set_de(&vga_lock, false);
            for mem_byte in 0..((w / 8) as usize) {
                let vga = vga_lock.read().unwrap();
                let v0 = vga.mem[0][mem_offset + mem_byte];
                let v1 = vga.mem[1][mem_offset + mem_byte];
                let v2 = vga.mem[2][mem_offset + mem_byte];
                let v3 = vga.mem[3][mem_offset + mem_byte];

                for b in 0..8 {
                    let bx = (1 << (7 - b)) as u8;
                    let mut c = bit_x(v0, bx, 0);
                    c |= bit_x(v1, bx, 1);
                    c |= bit_x(v2, bx, 2);
                    c |= bit_x(v3, bx, 3);
                    canvas.set_draw_color(default_color(c));
                    canvas.draw_point(Point::new(x as i32, y as i32)).unwrap();
                    x += 1;
                }
            }
            x = 0;
            y += 1;
            mem_offset += offset_delta * 2;

            set_de(&vga_lock, true);
            let h_elapsed = h_start.elapsed().as_micros();
            if h_elapsed < 32 {
                //we are currently nowhere near being that fast in the simulation
                sleep(Duration::from_micros(32 - h_elapsed as u64));
            } 
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

        set_de(&vga_lock, true);
        set_vr(&vga_lock, true);
        let v_elapsed = frame_start.elapsed().as_micros();
        if v_elapsed < TARGET_FRAME_RATE_MICRO {
            sleep(Duration::from_micros((TARGET_FRAME_RATE_MICRO -  v_elapsed) as u64));
        } else {
            println!("frame rate miss: {} > {}", v_elapsed, TARGET_FRAME_RATE_MICRO);
        }
    }

    fn bit_x(v: u8, v_ix: u8, dst_ix: u8) -> u8 {
        if v & v_ix != 0 {
            1 << dst_ix
        } else {
            0
        }
    }

    //vertical retrace
    fn set_vr(vga_lock: &Arc<RwLock<VGA>>, set: bool) {
        let mut vga = vga_lock.write().unwrap();
        let v0 = vga.get_general_reg(GeneralReg::InputStatus1);
        if set {
            vga.set_general_reg(GeneralReg::InputStatus1, v0 | !CLEAR_VR_MASK );
        } else {
            vga.set_general_reg(GeneralReg::InputStatus1, v0 & CLEAR_VR_MASK);
        }
    }

    //display enable NOT
    fn set_de(vga_lock: &Arc<RwLock<VGA>>, set: bool) {
        let mut vga = vga_lock.write().unwrap();
        let v0 = vga.get_general_reg(GeneralReg::InputStatus1);
        if set {
            vga.set_general_reg(GeneralReg::InputStatus1, v0 | !CLEAR_DE_MASK );
        } else {
            vga.set_general_reg(GeneralReg::InputStatus1, v0 & CLEAR_DE_MASK);
        }
    }
}

fn default_color(v: u8) -> Color {
    //source: https://wasteland.fandom.com/wiki/EGA_Colour_Palette
    return match v {
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
    };
}
