use ash::vk::{self, Extent3D};

use crate::vulkan_backend::{
    backend::VBackend,
    memory::{config::VImageConfig, image::VImage},
};

pub struct Image {
    v_image: VImage,
}

impl Image {
    pub fn new(v_backend: &VBackend, image_path: &str) -> Self {
        let image = image::open(image_path).expect("failed to open image texture");
        let image_rgba = image.to_rgba8();
        let image_extent = Extent3D {
            width: image.width(),
            height: image.height(),
            depth: 1,
        };
        let image_size = image_extent.width as u64 * image_extent.height as u64 * 4;

        let v_image = VImage::new(
            v_backend,
            VImageConfig {
                extent: image_extent,
                size: image_size,
                usage: vk::ImageUsageFlags::TRANSFER_DST | vk::ImageUsageFlags::SAMPLED,
                sharing_mode: v_backend.v_device.buffer_sharing_mode,
                queue_families: Some(v_backend.v_device.buffer_queue_family_indices.clone()),
                memory_property: vk::MemoryPropertyFlags::DEVICE_LOCAL,
            },
        );

        v_image.copy_to_image(v_backend, image_rgba.as_ptr(), image_size);

        Self { v_image }
    }

    pub fn destroy(&self, v_backend: &VBackend) {
        self.v_image.destroy(v_backend);
    }
}
