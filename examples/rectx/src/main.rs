use vgarectx::start_rectx;

fn main() {
    vga::util::spawn_async(async move {
        start_rectx()
            .await
            .expect("rectx demo finished without error");
    });
}
