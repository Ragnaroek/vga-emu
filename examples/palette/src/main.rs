use std::env;

use vgapalette::start_palette;

fn main() {
    let arg = match env::args().nth(1) {
        Some(arg) => arg,
        None => "".to_string(),
    };

    vga::util::spawn_async(async move {
        start_palette(arg)
            .await
            .expect("palette demo finished without error");
    });
}
