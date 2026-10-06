use std::env;

use vgakite::start_kite;

fn main() {
    let arg = match env::args().nth(1) {
        Some(arg) => arg,
        None => "".to_string(),
    };

    vga::util::spawn_async(async move {
        start_kite(arg)
            .await
            .expect("kite demo finished without error");
    });
}
