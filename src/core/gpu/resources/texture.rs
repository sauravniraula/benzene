use ash::vk;

use crate::core::resources::image::Image;
use crate::vulkan_backend::descriptor::{VDescriptorSets, VDescriptorWriteBatch};
use crate::vulkan_backend::{
    backend::VBackend,
    memory::image::{image_view::VImageView, sampler::VSampler},
};

/// Owns an image, its view and sampler for binding as a combined image sampler.
pub struct ImageTexture {
    pub image: Image,
    pub image_view: VImageView,
    pub sampler: VSampler,
}

impl ImageTexture {
    pub fn new(v_backend: &VBackend, image_path: &str, format: vk::Format) -> Self {
        let image = Image::new(v_backend, image_path);
        let image_view = VImageView::new_2d(
            &v_backend.v_device,
            &image.v_image,
            vk::ImageAspectFlags::COLOR,
            format,
        );
        let sampler = VSampler::new(&v_backend.v_device, &v_backend.v_physical_device);
        Self {
            image,
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
        self.image.destroy(v_backend);
    }
}
