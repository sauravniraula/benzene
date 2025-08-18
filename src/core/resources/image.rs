use crate::vulkan_backend::backend::VBackend;

pub struct Image {}

impl Image {
    pub fn new(v_backend: &VBackend, image_path: &str) -> Self {
        let image = image::open(image_path).expect("failed to open image texture");
        let image_rgba = image.to_rgba8();

        Self {}
    }
}
