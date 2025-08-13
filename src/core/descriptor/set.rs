use ash::vk;

use crate::core::{
    descriptor::{VDescriptorLayout, VDescriptorPool},
    device::VDevice,
    memory::VBuffer,
};

pub struct VDescriptorSets {
    pub sets: Vec<vk::DescriptorSet>,
    pub count: usize,
}

impl VDescriptorSets {
    pub fn new(
        v_device: &VDevice,
        v_pool: &VDescriptorPool,
        v_layout: &VDescriptorLayout,
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

    pub fn bind_all(&self, v_device: &VDevice, buffers: Vec<&VBuffer>) {
        assert!(
            self.count == buffers.len(),
            "length of descriptor set is not same as length of provided buffers"
        );

        let mut buffer_infos: Vec<vk::DescriptorBufferInfo> = vec![];
        let mut writes: Vec<vk::WriteDescriptorSet> = vec![];
        for i in 0..self.count {
            buffer_infos.push(
                vk::DescriptorBufferInfo::default()
                    .buffer(buffers[i].buffer)
                    .offset(0)
                    .range(vk::WHOLE_SIZE),
            );
        }
        for i in 0..self.count {
            writes.push(
                vk::WriteDescriptorSet::default()
                    .dst_set(self.sets[i])
                    .dst_binding(0)
                    .dst_array_element(0)
                    .descriptor_type(vk::DescriptorType::UNIFORM_BUFFER)
                    .descriptor_count(1)
                    .buffer_info(&buffer_infos[i..i + 1]),
            );
        }

        unsafe { v_device.device.update_descriptor_sets(&writes, &[]) };
    }
}
