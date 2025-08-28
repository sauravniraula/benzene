use crate::{core::gpu::texture::ImageTexture, vulkan_backend::backend::VBackend};

pub struct Material3D {
    pub color_texture: Option<ImageTexture>,
    pub manager_index: usize,
}

impl Material3D {
    pub fn destroy(&self, v_backend: &VBackend) {
        if let Some(tex) = &self.color_texture {
            tex.destroy(v_backend);
        }
    }
}
