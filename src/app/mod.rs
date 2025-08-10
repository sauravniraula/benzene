use ash::vk;

use crate::core::{backend::VBackend, game_objects::models::VModel};

pub struct VApp<'a> {
    pub v_backend: &'a VBackend,
    pub triangle: VModel,
}

impl<'a> VApp<'a> {
    pub fn new(v_backend: &'a VBackend) -> Self {
        // Triangle Model
        let triangle = VModel::new(&v_backend);

        Self {
            v_backend,
            triangle,
        }
    }

    pub fn render(&self, command_buffer: vk::CommandBuffer, image_index: usize) {
        self.v_backend.basic_rendering_system.render(
            &self.v_backend.v_device,
            command_buffer,
            image_index,
            self.triangle.vertices.len() as u32,
            &[self.triangle.v_buffer.buffer],
        );
    }

    pub fn destroy(&self) {
        [&self.triangle].map(|each_model| {
            each_model.destroy(&self.v_backend);
        });
    }
}
