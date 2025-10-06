use wasm_bindgen::prelude::*;

use super::start_kite;

#[wasm_bindgen]
pub fn start_kite_web() {
    console_error_panic_hook::set_once();

    vga::util::spawn_async(async move {
        start_kite()
            .await
            .expect("kite demo finished without error");
    });
}
