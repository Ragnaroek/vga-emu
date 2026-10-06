use std::env;

use vgaball::start_ball;

fn main() {
    let arg = match env::args().nth(1) {
        Some(arg) => arg,
        None => "".to_string(),
    };

    vga::util::spawn_async(async move {
        start_ball(arg)
            .await
            .expect("ball demo finished without error");
    });
}
