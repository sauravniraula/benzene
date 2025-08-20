use ash::vk::{self, Extent3D};

use crate::vulkan_backend::descriptor::{VDescriptorSets, VDescriptorWriteBatch};
use crate::vulkan_backend::memory::image::{VImage, VImageConfig};
use crate::vulkan_backend::{
    backend::VBackend,
    memory::image::{image_view::VImageView, sampler::VSampler},
};

pub struct ImageTexture {
    pub image: VImage,
    pub image_view: VImageView,
    pub sampler: VSampler,
}

impl ImageTexture {
    pub fn new(v_backend: &VBackend, image_path: &str, format: vk::Format) -> Self {
        let opened_image = image::open(image_path).expect("failed to open image texture");
        let image_rgba = opened_image.to_rgba8();
        let image_extent = Extent3D {
            width: opened_image.width(),
            height: opened_image.height(),
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

        v_image.copy_to_image(
            &v_backend.v_device,
            &v_backend.v_physical_device,
            &v_backend.v_memory_manager,
            image_rgba.as_ptr(),
            image_size,
        );
        let image_view = VImageView::new_2d(
            &v_backend.v_device,
            &v_image,
            vk::ImageAspectFlags::COLOR,
            format,
        );
        let sampler = VSampler::new(&v_backend.v_device, &v_backend.v_physical_device);
        Self {
            image: v_image,
            image_view,
            sampler,
        }
    }

    pub fn queue_descriptor_writes(
        &self,
        sets: &VDescriptorSets,
        batch: &mut VDescriptorWriteBatch,
    ) {
        let views = (0..sets.count)
            .map(|_| &self.image_view)
            .collect::<Vec<_>>();
        let samplers = (0..sets.count).map(|_| &self.sampler).collect::<Vec<_>>();
        sets.queue_image_all(
            batch,
            1,
            vk::DescriptorType::COMBINED_IMAGE_SAMPLER,
            &views,
            &samplers,
            vk::ImageLayout::SHADER_READ_ONLY_OPTIMAL,
        );
    }

    pub fn destroy(&self, v_backend: &VBackend) {
        self.image_view.destroy(&v_backend.v_device);
        self.sampler.destroy(&v_backend.v_device);
        self.image
            .destroy(&v_backend.v_device, &v_backend.v_memory_manager);
    }
}
