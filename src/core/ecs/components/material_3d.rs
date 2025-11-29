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
        let set = manager.get_set_at(self.manager_index);
        set.queue_image(
            batch,
            vk::DescriptorType::COMBINED_IMAGE_SAMPLER,
            0,
            &texture.image_view,
            &texture.sampler,
            vk::ImageLayout::SHADER_READ_ONLY_OPTIMAL,
        );
    }
}
