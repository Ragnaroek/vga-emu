use vgapatternx::start_patternx;

fn main() {
    vga::util::spawn_async(async move {
        start_patternx()
            .await
            .expect("patternx demo finished without error");
    });
}
