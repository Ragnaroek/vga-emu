use wasm_bindgen::prelude::*;

use super::start_m320x400;

#[wasm_bindgen]
pub fn start_m320x400_web() {
    console_error_panic_hook::set_once();

    vga::util::spawn_async(async move {
        start_m320x400()
            .await
            .expect("m320x400 demo finished without error");
    });
}
