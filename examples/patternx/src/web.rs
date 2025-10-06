use wasm_bindgen::prelude::*;

use super::start_patternx;

#[wasm_bindgen]
pub fn start_patternx_web() {
    console_error_panic_hook::set_once();

    vga::util::spawn_async(async move {
        start_patternx()
            .await
            .expect("patternx demo finished without error");
    });
}
