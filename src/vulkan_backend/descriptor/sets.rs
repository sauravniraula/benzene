use ash::vk;

use crate::vulkan_backend::device::VDevice;
use crate::vulkan_backend::{
    descriptor::{VDescriptorPool, VDescriptorSetLayout, VDescriptorWriteBatch},
    memory::{
        VBuffer,
        image::{image_view::VImageView, sampler::VSampler},
    },
};

pub struct VDescriptorSet {
    pub set: vk::DescriptorSet,
}

impl VDescriptorSet {
    pub fn new(
        v_device: &VDevice,
        v_pool: &VDescriptorPool,
        v_layout: &VDescriptorSetLayout,
    ) -> Self {
        let alloc_info = vk::DescriptorSetAllocateInfo::default()
            .descriptor_pool(v_pool.pool)
            .set_layouts(std::slice::from_ref(&v_layout.layout));

        let sets = unsafe {
            v_device
                .device
                .allocate_descriptor_sets(&alloc_info)
                .expect("failed to allocate descriptor sets")
        };

        Self { set: sets[0] }
    }

    pub fn queue_buffer(
        &self,
        batch: &mut VDescriptorWriteBatch,
        descriptor_type: vk::DescriptorType,
        binding: u32,
        v_buffer: &VBuffer,
    ) {
        batch.queue_buffer(
            self.set,
            descriptor_type,
            binding,
            v_buffer.buffer,
            0,
            vk::WHOLE_SIZE,
        );
    }

    // pub fn queue_buffer_all_sets(
    //     &self,
    //     batch: &mut VDescriptorWriteBatch,
    //     binding: u32,
    //     descriptor_type: vk::DescriptorType,
    //     v_buffers: &[&VBuffer],
    // ) {
    //     let mut index = 0;
    //     loop {
    //         if index == self.count {
    //             break;
    //         }
    //         self.queue_buffer(batch, index, binding, descriptor_type, v_buffers[index]);
    //         index += 1;
    //     }
    // }

    pub fn queue_image(
        &self,
        batch: &mut VDescriptorWriteBatch,
        descriptor_type: vk::DescriptorType,
        binding: u32,
        v_image_view: &VImageView,
        v_sampler: &VSampler,
        image_layout: vk::ImageLayout,
    ) {
        batch.queue_image(
            self.set,
            descriptor_type,
            binding,
            v_image_view.image_view,
            v_sampler.sampler,
            image_layout,
        );
    }

    // pub fn queue_image_all_sets(
    //     &self,
    //     batch: &mut VDescriptorWriteBatch,
    //     binding: u32,
    //     descriptor_type: vk::DescriptorType,
    //     v_image_views: &[&VImageView],
    //     v_samplers: &[&VSampler],
    //     image_layout: vk::ImageLayout,
    // ) {
    //     assert!(
    //         v_image_views.len() == self.count && v_samplers.len() == self.count,
    //         "image views and samplers must match descriptor set count",
    //     );
    //     let mut index = 0;
    //     loop {
    //         if index == self.count {
    //             break;
    //         }
    //         self.queue_image(
    //             batch,
    //             index,
    //             binding,
    //             descriptor_type,
    //             v_image_views[index],
    //             v_samplers[index],
    //             image_layout,
    //         );
    //         index += 1;
    //     }
    // }
}
