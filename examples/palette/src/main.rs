use vgapalette::start_palette;

fn main() {
    vga::util::spawn_async(async move {
        start_palette()
            .await
            .expect("palette demo finished without error");
    });
}
