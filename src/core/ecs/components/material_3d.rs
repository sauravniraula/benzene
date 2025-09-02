use ash::vk;

use crate::{
    core::gpu::materials_manager::MaterialsManager, core::gpu::texture::ImageTexture,
    vulkan_backend::descriptor::VDescriptorWriteBatch,
};

pub struct Material3D {
    pub manager_index: usize,
}

impl Material3D {
    pub fn queue_descriptor_writes(
        &self,
        manager: &MaterialsManager,
        texture: &ImageTexture,
        batch: &mut VDescriptorWriteBatch,
    ) {
        let sets = manager.get_sets_at(self.manager_index);
        let views = (0..sets.count)
            .map(|_| &texture.image_view)
            .collect::<Vec<_>>();
        let samplers = (0..sets.count)
            .map(|_| &texture.sampler)
            .collect::<Vec<_>>();
        sets.queue_image_all_sets(
            batch,
            0,
            vk::DescriptorType::COMBINED_IMAGE_SAMPLER,
            &views,
            &samplers,
            vk::ImageLayout::SHADER_READ_ONLY_OPTIMAL,
        );
    }
}
