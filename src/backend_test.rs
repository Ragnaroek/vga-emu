use crate::{InputMonitoring, VGABuilder, VGAEmu};

pub struct RenderContext {
    input_monitoring: InputMonitoring,
}

impl RenderContext {
    pub fn init(_: usize, _: usize, _: VGABuilder) -> Result<RenderContext, String> {
        Ok(RenderContext {
            input_monitoring: InputMonitoring::new(),
        })
    }

    pub fn draw_frame(&mut self, _: &VGAEmu) -> bool {
        false
    }

    pub fn input_monitoring(&mut self) -> &mut InputMonitoring {
        &mut self.input_monitoring
    }
}
