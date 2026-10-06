use std::env;

use vgapatternx::start_patternx;

fn main() {
    let arg = match env::args().nth(1) {
        Some(arg) => arg,
        None => "".to_string(),
    };

    vga::util::spawn_async(async move {
        start_patternx(arg)
            .await
            .expect("patternx demo finished without error");
    });
}
