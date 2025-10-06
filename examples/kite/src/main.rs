use vgakite::start_kite;

fn main() {
    vga::util::spawn_async(async move {
        start_kite()
            .await
            .expect("kite demo finished without error");
    });
}
