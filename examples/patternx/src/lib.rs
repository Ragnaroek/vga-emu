#[cfg(feature = "web")]
pub mod web;

#[cfg(feature = "sdl2")]
use vga::backend_sdl2::Sdl2RenderBackend;
#[cfg(feature = "sdl3")]
use vga::backend_sdl3::Sdl3RenderBackend;
#[cfg(feature = "web")]
use vga::backend_web::WebCanvasRenderBackend;
//Rectangle example from https://github.com/jagregory/abrash-black-book/blob/master/src/chapter-48.md (LISTING 48.2)
use vga::{RenderBackend, SCReg, VGA, set_vertical_display_end};
use vga::{
    VGABuilder,
    util::{fill_pattern_x, sleep},
};

static PATT_TABLE: [[u8; 16]; 16] = [
    [10, 0, 10, 0, 0, 10, 0, 10, 10, 0, 10, 0, 0, 10, 0, 10],
    [9, 0, 0, 0, 0, 9, 0, 0, 0, 0, 9, 0, 0, 0, 0, 9],
    [5, 0, 0, 0, 0, 0, 5, 0, 5, 0, 0, 0, 0, 0, 5, 0],
    [14, 0, 0, 14, 0, 14, 14, 0, 0, 14, 14, 0, 14, 0, 0, 14],
    [15, 15, 15, 1, 15, 15, 1, 1, 15, 1, 1, 1, 1, 1, 1, 1],
    [12, 12, 12, 12, 6, 6, 6, 12, 6, 6, 6, 12, 6, 6, 6, 12],
    [
        80, 80, 80, 80, 80, 80, 80, 80, 80, 80, 80, 80, 80, 80, 80, 15,
    ],
    [
        78, 78, 78, 78, 80, 80, 80, 80, 82, 82, 82, 82, 84, 84, 84, 84,
    ],
    [
        78, 80, 82, 84, 80, 82, 84, 78, 82, 84, 78, 80, 84, 78, 80, 82,
    ],
    [
        78, 80, 82, 84, 78, 80, 82, 84, 78, 80, 82, 84, 78, 80, 82, 84,
    ],
    [0, 1, 2, 3, 4, 5, 6, 7, 8, 9, 10, 11, 12, 13, 14, 15],
    [0, 1, 2, 3, 0, 1, 2, 3, 0, 1, 2, 3, 0, 1, 2, 3],
    [14, 14, 9, 9, 14, 9, 9, 14, 9, 9, 14, 14, 9, 14, 14, 9],
    [15, 8, 8, 8, 15, 15, 15, 8, 15, 15, 15, 8, 15, 8, 8, 8],
    [3, 3, 3, 3, 3, 7, 7, 3, 3, 7, 7, 3, 3, 3, 3, 3],
    [0, 0, 0, 0, 0, 64, 0, 0, 0, 0, 0, 0, 0, 0, 0, 89],
];

pub async fn start_patternx(arg: String) -> Result<(), String> {
    let builder = VGABuilder::new()
        .video_mode(0x13)
        .title(format!("VGA PatternX Example ({})", arg))
        .fullscreen(false);

    match arg.as_str() {
        #[cfg(feature = "sdl3")]
        "sdl3" | "" => run_patternx(builder.build(Sdl3RenderBackend::new())?).await,
        #[cfg(not(feature = "sdl3"))]
        "sdl3" => return Err("sdl3 backend not available (build with --features sdl3)".to_string()),

        #[cfg(feature = "sdl2")]
        "sdl2" => run_patternx(builder.build(Sdl2RenderBackend::new())?).await,
        #[cfg(not(feature = "sdl2"))]
        "sdl2" => return Err("sdl2 backend not available (build with --features sdl2)".to_string()),

        #[cfg(feature = "web")]
        "web" => run_patternx(builder.build(WebCanvasRenderBackend::new())?).await,
        #[cfg(not(feature = "web"))]
        "web" => {
            return Err("web backend not available (build with --features web)".to_string());
        }

        other => return Err(format!("unknown backend: {}", other)),
    }
}

async fn run_patternx<R: RenderBackend>(mut vga: VGA<R>) -> Result<(), String> {
    //enable Mode X
    let mem_mode = vga.get_sc_data(SCReg::MemoryMode);
    vga.set_sc_data(SCReg::MemoryMode, (mem_mode & !0x08) | 0x04); //turn off chain 4 & odd/even
    set_vertical_display_end(&mut vga, 480);

    for j in 0..4 {
        for i in 0..4 {
            fill_pattern_x(
                &mut vga,
                i * 80,
                j * 60,
                i * 80 + 80,
                j * 60 + 60,
                0,
                &PATT_TABLE[j * 4 + i],
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
