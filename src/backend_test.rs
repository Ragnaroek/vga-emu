use crate::{InputMonitoring, VGABuilder, VGAEmu};
use std::sync::{RwLock, RwLockWriteGuard};

pub struct RenderContext {
    input_monitoring: RwLock<InputMonitoring>,
}

impl RenderContext {
    pub fn init(_: usize, _: usize, _: VGABuilder) -> Result<RenderContext, String> {
        Ok(RenderContext {
            input_monitoring: RwLock::new(InputMonitoring::new()),
        })
    }

    pub fn draw_frame(&mut self, _: &VGAEmu) -> bool {
        false
    }

    pub fn input_monitoring<'a>(&'a mut self) -> RwLockWriteGuard<'a, InputMonitoring> {
        self.input_monitoring
            .write()
            .expect("write lock InputMonitoring")
    }
}
