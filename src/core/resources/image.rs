use ash::vk::{self, Extent3D};

use crate::vulkan_backend::{
    backend::VBackend,
    memory::image::{config::VImageConfig, VImage},
};

pub struct Image {
    pub v_image: VImage,
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
            &v_backend.v_device,
            &v_backend.v_physical_device,
            &v_backend.v_memory_manager,
            VImageConfig::image_2d(
                image_extent,
                image_size,
                vk::ImageUsageFlags::TRANSFER_DST | vk::ImageUsageFlags::SAMPLED,
                v_backend.v_device.buffer_sharing_mode,
                Some(v_backend.v_device.buffer_queue_family_indices.clone()),
                vk::MemoryPropertyFlags::DEVICE_LOCAL,
                vk::Format::R8G8B8A8_SRGB,
            ),
        );

        v_image.copy_to_image(&v_backend.v_device, &v_backend.v_physical_device, &v_backend.v_memory_manager, image_rgba.as_ptr(), image_size);

        Self { v_image }
    }

    pub fn destroy(&self, v_backend: &VBackend) {
        self.v_image.destroy(&v_backend.v_device, &v_backend.v_memory_manager);
    }
}
