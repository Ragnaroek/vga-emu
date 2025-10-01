use vgam320x400::start_m320x400;

fn main() {
    vga::util::spawn_async(async move {
        start_m320x400()
            .await
            .expect("m320x400 demo finished without error");
    });
}
