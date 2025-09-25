use wasm_bindgen::prelude::*;

use super::start_ball;

#[wasm_bindgen]
pub fn start_ball_web() {
    console_error_panic_hook::set_once();

    vga::util::spawn_async(async move {
        start_ball()
            .await
            .expect("ball demo finished without error");
    });
}
