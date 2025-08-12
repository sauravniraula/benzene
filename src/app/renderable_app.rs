use crate::core::backend::VBackend;
use ash::vk;

pub trait RenderableApp {
    fn render_app(
        &self,
        v_backend: &VBackend,
        command_buffer: vk::CommandBuffer,
        image_index: usize,
    );
}
