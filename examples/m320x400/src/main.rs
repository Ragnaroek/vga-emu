use std::env;

use vgam320x400::start_m320x400;

fn main() {
    let arg = match env::args().nth(1) {
        Some(arg) => arg,
        None => "".to_string(),
    };

    vga::util::spawn_async(async move {
        start_m320x400(arg)
            .await
            .expect("m320x400 demo finished without error");
    });
}
