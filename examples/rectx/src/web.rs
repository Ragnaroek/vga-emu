use wasm_bindgen::prelude::*;

use super::start_rectx;

#[wasm_bindgen]
pub fn start_rectx_web() {
    console_error_panic_hook::set_once();

    vga::util::spawn_async(async move {
        start_rectx()
            .await
            .expect("rectx demo finished without error");
    });
}
