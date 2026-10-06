use crate::{InputMonitoring, RenderBackend, VGABuilder, VGAEmu};
use std::sync::{RwLock, RwLockWriteGuard};

pub struct TestRenderBackend {
    input_monitoring: RwLock<InputMonitoring>,
}

impl TestRenderBackend {
    pub fn new() -> TestRenderBackend {
        TestRenderBackend {
            input_monitoring: RwLock::new(InputMonitoring::new()),
        }
    }
}

impl RenderBackend for TestRenderBackend {
    fn init(&mut self, _: usize, _: usize, _: VGABuilder) -> Result<(), String> {
        Ok(())
    }

    fn draw_frame(&mut self, _: &mut VGAEmu) -> bool {
        false
    }

    fn input_monitoring<'a>(&'a mut self) -> RwLockWriteGuard<'a, InputMonitoring> {
        self.input_monitoring
            .write()
            .expect("write lock InputMonitoring")
    }
}
