#[cfg(feature = "web")]
pub mod web;

#[cfg(feature = "sdl2")]
use vga::backend_sdl2::Sdl2RenderBackend;
#[cfg(feature = "sdl3")]
use vga::backend_sdl3::Sdl3RenderBackend;
#[cfg(feature = "web")]
use vga::backend_web::WebCanvasRenderBackend;
//Example from https://www.phatcode.net/res/224/files/html/ch31/31-03.html (LISTING 31.3)
use vga::{CRTReg, GCReg, RenderBackend, SCReg, VGA, VGABuilder, input::NumCode, util::sleep};

const SCREEN_WIDTH: usize = 320;

struct LineControl {
    start_x: i16,
    start_y: i16,
    x_inc: i16,
    y_inc: i16,
    base_len: i16,
    color: u8,
}

fn new_line(
    start_x: i16, start_y: i16, x_inc: i16, y_inc: i16, base_len: i16, color: u8,
) -> LineControl {
    LineControl {
        start_x,
        start_y,
        x_inc,
        y_inc,
        base_len,
        color,
    }
}

pub async fn start_m320x400(arg: String) -> Result<(), String> {
    let builder = VGABuilder::new()
        .video_mode(0x13)
        .title(format!("VGA m320x400 Example ({})", arg))
        .fullscreen(false);

    match arg.as_str() {
        #[cfg(feature = "sdl3")]
        "sdl3" | "" => run_m320x400(builder.build(Sdl3RenderBackend::new())?).await,
        #[cfg(not(feature = "sdl3"))]
        "sdl3" => return Err("sdl3 backend not available (build with --features sdl3)".to_string()),

        #[cfg(feature = "sdl2")]
        "sdl2" => run_m320x400(builder.build(Sdl2RenderBackend::new())?).await,
        #[cfg(not(feature = "sdl2"))]
        "sdl2" => return Err("sdl2 backend not available (build with --features sdl2)".to_string()),

        #[cfg(feature = "web")]
        "web" => run_m320x400(builder.build(WebCanvasRenderBackend::new())?).await,
        #[cfg(not(feature = "web"))]
        "web" => {
            return Err("web backend not available (build with --features web)".to_string());
        }

        other => return Err(format!("unknown backend: {}", other)),
    }
}

async fn run_m320x400<R: RenderBackend>(mut vga: VGA<R>) -> Result<(), String> {
    //set 320x400 mode
    let mem_mode = vga.get_sc_data(SCReg::MemoryMode);
    vga.set_sc_data(SCReg::MemoryMode, (mem_mode & !0x08) | 0x04); //turn off chain 4 & odd/even

    let gc_mode = vga.get_gc_data(GCReg::GraphicsMode);
    vga.set_gc_data(GCReg::GraphicsMode, gc_mode & !0x10); //turn off odd/even

    let gc_misc = vga.get_gc_data(GCReg::MiscGraphics);
    vga.set_gc_data(GCReg::MiscGraphics, gc_misc & !0x02); //turn off chain

    //clear display memory
    vga.set_sc_data(SCReg::MapMask, 0x0F);
    for cx in 0..vga::PLANE_SIZE {
        vga.write_mem(cx, 0);
    }

    let max_scan = vga.get_crt_data(CRTReg::MaximumScanLine);
    vga.set_crt_data(CRTReg::MaximumScanLine, max_scan & !0x1F);

    let underline = vga.get_crt_data(CRTReg::UnderlineLocation);
    vga.set_crt_data(CRTReg::UnderlineLocation, underline & !0x40); //turn off doubleword

    let crt_mode = vga.get_crt_data(CRTReg::CRTCModeControl);
    vga.set_crt_data(CRTReg::CRTCModeControl, crt_mode & 0x40); //turn on byte mode bit

    let line_list = vec![
        new_line(130, 110, 1, 0, 60, 0),
        new_line(190, 110, 1, 1, 60, 1),
        new_line(250, 170, 0, 1, 60, 2),
        new_line(250, 230, -1, 1, 60, 3),
        new_line(190, 290, -1, 0, 60, 4),
        new_line(130, 290, -1, -1, 60, 5),
        new_line(70, 230, 0, -1, 60, 6),
        new_line(70, 170, 1, -1, 60, 7),
    ];

    loop {
        for b in 0..8 {
            for line in &line_list {
                let mut x = line.start_x;
                let mut y = line.start_y;
                for _ in 0..line.base_len {
                    write_pixel(&mut vga, x, y, b + line.color);
                    x += line.x_inc;
                    y += line.y_inc;
                }
            }

            loop {
                if vga.input_monitoring().key_pressed(NumCode::Return) {
                    vga.input_monitoring().clear_keyboard();
                    break;
                }

                if vga.draw_frame() {
                    return Ok(()); // quit
                }

                sleep(14).await; // target 70 fps
            }
        }
    }
}

fn write_pixel<R: RenderBackend>(vga: &mut vga::VGA<R>, x: i16, y: i16, color: u8) {
    let mut offset = (SCREEN_WIDTH / 4) * y as usize;
    offset += x as usize / 4;

    let plane = x & 0x03;
    let mask = 1 << plane;
    vga.set_sc_data(SCReg::MapMask, mask);
    vga.write_mem(offset, color);
}
