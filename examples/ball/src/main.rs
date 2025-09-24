use vgaball::start_ball;

fn main() -> Result<(), String> {
    let rt = tokio::runtime::Builder::new_current_thread()
        .enable_time()
        .build()
        .map_err(|e| e.to_string())?;

    rt.block_on(async move {
        let local = tokio::task::LocalSet::new();
        local
            .run_until(async move {
                tokio::task::spawn_local(async move {
                    start_ball()
                        .await
                        .expect("ball demo finished without error");
                })
                .await
                .unwrap();
            })
            .await;
    });

    Ok(())
}
