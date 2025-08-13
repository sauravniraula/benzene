use std::time::Duration;

use crate::core::backend::VBackend;
use ash::vk;

pub trait RenderableApp {
    fn render_app(
        &mut self,
        v_backend: &VBackend,
        command_buffer: vk::CommandBuffer,
        image_index: usize,
        frame_index: usize,
        duration: Duration,
    );
}
