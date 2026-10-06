#[cfg(feature = "web")]
pub mod web;

#[cfg(feature = "sdl2")]
use vga::backend_sdl2::Sdl2RenderBackend;
#[cfg(feature = "sdl3")]
use vga::backend_sdl3::Sdl3RenderBackend;
#[cfg(feature = "web")]
use vga::backend_web::WebCanvasRenderBackend;
//Example from https://www.phatcode.net/res/224/files/html/ch47/47-07.html (LISTING 47.6)
use vga::{RenderBackend, SCReg, VGA, set_vertical_display_end};
use vga::{
    VGABuilder,
    util::{fill_rectangle_x, sleep},
};

pub async fn start_rectx(arg: String) -> Result<(), String> {
    let builder = VGABuilder::new()
        .video_mode(0x13)
        .title("VGA RectX Example".to_string())
        .fullscreen(false);

    match arg.as_str() {
        #[cfg(feature = "sdl3")]
        "sdl3" | "" => run_rectx(builder.build(Sdl3RenderBackend::new())?).await,
        #[cfg(not(feature = "sdl3"))]
        "sdl3" => return Err("sdl3 backend not available (build with --features sdl3)".to_string()),

        #[cfg(feature = "sdl2")]
        "sdl2" => run_rectx(builder.build(Sdl2RenderBackend::new())?).await,
        #[cfg(not(feature = "sdl2"))]
        "sdl2" => return Err("sdl2 backend not available (build with --features sdl2)".to_string()),

        #[cfg(feature = "web")]
        "web" => run_rectx(builder.build(WebCanvasRenderBackend::new())?).await,
        #[cfg(not(feature = "web"))]
        "web" => {
            return Err("web backend not available (build with --features web)".to_string());
        }

        other => return Err(format!("unknown backend: {}", other)),
    }
}

async fn run_rectx<R: RenderBackend>(mut vga: VGA<R>) -> Result<(), String> {
    //enable Mode X
    let mem_mode = vga.get_sc_data(SCReg::MemoryMode);
    vga.set_sc_data(SCReg::MemoryMode, (mem_mode & !0x08) | 0x04); //turn off chain 4 & odd/even
    set_vertical_display_end(&mut vga, 480);

    fill_rectangle_x(&mut vga, 0, 0, 320, 240, 0, 0);

    let mut j = 1;
    while j < 220 {
        let mut i = 1;
        while i < 300 {
            fill_rectangle_x(
                &mut vga,
                i,
                j,
                i + 20,
                j + 20,
                0,
                (((j / 21 * 15) + i / 21) & 0xFF) as u8,
            );
            i += 21;
        }
        j += 21;
    }

    loop {
        if vga.draw_frame() {
            return Ok(()); // quit
        }
        sleep(14).await; // target 70 fps
    }
}
