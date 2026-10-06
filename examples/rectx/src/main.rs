use std::env;

use vgarectx::start_rectx;

fn main() {
    let arg = match env::args().nth(1) {
        Some(arg) => arg,
        None => "".to_string(),
    };

    vga::util::spawn_async(async move {
        start_rectx(arg)
            .await
            .expect("rectx demo finished without error");
    });
}
