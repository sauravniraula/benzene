use crate::{core::gpu::texture::ImageTexture, vulkan_backend::backend::VBackend};

pub struct Material3D {
    pub color_texture: Option<ImageTexture>,
}

impl Material3D {
    pub fn new() -> Self {
        Self {
            color_texture: None,
        }
    }

    pub fn with_color_texture(color_texture: ImageTexture) -> Self {
        Self {
            color_texture: Some(color_texture),
        }
    }

    pub fn destroy(&self, v_backend: &VBackend) {
        if let Some(tex) = &self.color_texture {
            tex.destroy(v_backend);
        }
    }
}
