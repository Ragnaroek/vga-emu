use vgaball::start_ball;

fn main() {
    vga::util::spawn_async(async move {
        start_ball()
            .await
            .expect("ball demo finished without error");
    });
}
