use wasm_bindgen::prelude::*;

use super::start_palette;

#[wasm_bindgen]
pub fn start_palette_web() {
    console_error_panic_hook::set_once();

    vga::util::spawn_async(async move {
        start_palette()
            .await
            .expect("palette demo finished without error");
    });
}
