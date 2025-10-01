use wasm_bindgen::prelude::*;
use wasm_bindgen_futures::spawn_local;

use super::start_ball;

#[wasm_bindgen]
pub fn start_ball_web() {
    console_error_panic_hook::set_once();

    spawn_local(async move {
        start_ball()
            .await
            .expect("ball demo finished without error");
    });
}
