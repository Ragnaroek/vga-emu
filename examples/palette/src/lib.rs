#[cfg(feature = "web")]
pub mod web;

use std::env;
use std::fs;

use vga::RenderBackend;
use vga::VGA;
#[cfg(feature = "sdl2")]
use vga::backend_sdl2::Sdl2RenderBackend;
#[cfg(feature = "sdl3")]
use vga::backend_sdl3::Sdl3RenderBackend;
#[cfg(feature = "web")]
use vga::backend_web::WebCanvasRenderBackend;
use vga::util::{self, sleep};
use vga::{ColorReg, SCReg, VGABuilder};

const SCREEN_WIDTH: usize = 320;
const SCREEN_HEIGHT: usize = 200;
const CUBE_SIZE: usize = 10;
const PALETTE_SIZE: usize = 16;

pub async fn start_palette(arg: String) -> Result<(), String> {
    let builder = VGABuilder::new()
        .video_mode(0x13)
        .fullscreen(false)
        .title(format!("VGA Palette Example ({})", arg));

    match arg.as_str() {
        #[cfg(feature = "sdl3")]
        "sdl3" | "" => run_palette(builder.build(Sdl3RenderBackend::new())?).await,
        #[cfg(not(feature = "sdl3"))]
        "sdl3" => return Err("sdl3 backend not available (build with --features sdl3)".to_string()),

        #[cfg(feature = "sdl2")]
        "sdl2" => run_palette(builder.build(Sdl2RenderBackend::new())?).await,
        #[cfg(not(feature = "sdl2"))]
        "sdl2" => return Err("sdl2 backend not available (build with --features sdl2)".to_string()),

        #[cfg(feature = "web")]
        "web" => run_palette(builder.build(WebCanvasRenderBackend::new())?).await,
        #[cfg(not(feature = "web"))]
        "web" => {
            return Err("web backend not available (build with --features web)".to_string());
        }

        other => return Err(format!("unknown backend: {}", other)),
    }
}

async fn run_palette<R: RenderBackend>(mut vga: VGA<R>) -> Result<(), String> {
    //enable Mode X
    let mem_mode = vga.get_sc_data(SCReg::MemoryMode);
    vga.set_sc_data(SCReg::MemoryMode, (mem_mode & !0x08) | 0x04); //turn off chain 4 & odd/even

    let mut args = env::args();
    if args.len() == 3 {
        let palette = fs::read(args.nth(2).unwrap()).map_err(|e| e.to_string())?;
        set_palette(&mut vga, &palette);
    }

    util::fill_rectangle_x(&mut vga, 0, 0, SCREEN_WIDTH, SCREEN_HEIGHT, 0, 0);

    let palette_size = (PALETTE_SIZE * (CUBE_SIZE + 1)) - 1;
    let x_start = (SCREEN_WIDTH - palette_size) / 2;
    let y_start = (SCREEN_HEIGHT - palette_size) / 2;

    for w in 0..PALETTE_SIZE {
        for h in 0..PALETTE_SIZE {
            let x = x_start + w * (CUBE_SIZE + 1);
            let y = y_start + h * (CUBE_SIZE + 1);
            util::fill_rectangle_x(
                &mut vga,
                x,
                y,
                x + CUBE_SIZE,
                y + CUBE_SIZE,
                0,
                (h * PALETTE_SIZE + w) as u8,
            );
        }
    }

    loop {
        if vga.draw_frame() {
            return Ok(()); // quit
        }
        sleep(14).await; // target 70 fps
    }
}

pub fn set_palette<R: RenderBackend>(vga: &mut vga::VGA<R>, palette: &[u8]) {
    assert_eq!(
        palette.len(),
        768,
        "palette file must be exact 768 bytes long"
    );
    vga.set_color_reg(ColorReg::AddressWriteMode, 0);
    for i in 0..768 {
        vga.set_color_reg(ColorReg::Data, palette[i]);
    }
}
