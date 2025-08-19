use ash::vk;

use crate::vulkan_backend::device::VDevice;
use crate::vulkan_backend::{
    descriptor::{VDescriptorPool, VDescriptorSetLayout, VDescriptorWriteBatch},
    memory::{
        VBuffer,
        image::{image_view::VImageView, sampler::VSampler},
    },
};

pub struct VDescriptorSets {
    pub sets: Vec<vk::DescriptorSet>,
    pub count: usize,
}

impl VDescriptorSets {
    pub fn new(
        v_device: &VDevice,
        v_pool: &VDescriptorPool,
        v_layout: &VDescriptorSetLayout,
        count: usize,
    ) -> Self {
        let layouts: Vec<vk::DescriptorSetLayout> = (0..count).map(|_| v_layout.layout).collect();
        let alloc_info = vk::DescriptorSetAllocateInfo::default()
            .descriptor_pool(v_pool.pool)
            .set_layouts(&layouts);

        let sets = unsafe {
            v_device
                .device
                .allocate_descriptor_sets(&alloc_info)
                .expect("failed to allocate descriptor sets")
        };

        Self { sets, count }
    }

    pub fn queue_buffer(
        &self,
        batch: &mut VDescriptorWriteBatch,
        set_index: usize,
        binding: u32,
        descriptor_type: vk::DescriptorType,
        v_buffer: &VBuffer,
    ) {
        assert!(set_index < self.count, "set_index out of range");
        batch.queue_buffer(
            self.sets[set_index],
            binding,
            descriptor_type,
            v_buffer.buffer,
            vk::WHOLE_SIZE,
        );
    }

    pub fn queue_buffer_all(
        &self,
        batch: &mut VDescriptorWriteBatch,
        binding: u32,
        descriptor_type: vk::DescriptorType,
        v_buffers: &[&VBuffer],
    ) {
        assert!(
            self.count == v_buffers.len(),
            "length of descriptor set is not same as length of provided buffers",
        );
        for (set, vbuf) in self.sets.iter().copied().zip(v_buffers.iter().copied()) {
            batch.queue_buffer(set, binding, descriptor_type, vbuf.buffer, vk::WHOLE_SIZE);
        }
    }

    pub fn queue_image(
        &self,
        batch: &mut VDescriptorWriteBatch,
        set_index: usize,
        binding: u32,
        descriptor_type: vk::DescriptorType,
        v_image_view: &VImageView,
        v_sampler: &VSampler,
        image_layout: vk::ImageLayout,
    ) {
        assert!(set_index < self.count, "set_index out of range");
        batch.queue_image(
            self.sets[set_index],
            binding,
            descriptor_type,
            v_image_view.image_view,
            v_sampler.sampler,
            image_layout,
        );
    }

    pub fn queue_image_all(
        &self,
        batch: &mut VDescriptorWriteBatch,
        binding: u32,
        descriptor_type: vk::DescriptorType,
        v_image_views: &[&VImageView],
        v_samplers: &[&VSampler],
        image_layout: vk::ImageLayout,
    ) {
        assert!(
            v_image_views.len() == self.count && v_samplers.len() == self.count,
            "image views and samplers must match descriptor set count",
        );
        for ((set, view), sampler) in self
            .sets
            .iter()
            .copied()
            .zip(v_image_views.iter().copied())
            .zip(v_samplers.iter().copied())
        {
            batch.queue_image(
                set,
                binding,
                descriptor_type,
                view.image_view,
                sampler.sampler,
                image_layout,
            );
        }
    }
}
