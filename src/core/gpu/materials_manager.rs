use crate::vulkan_backend::{
    descriptor::{
        VDescriptorPool, VDescriptorSet, VDescriptorSetLayout,
        config::{VDescriptorPoolConfig, VDescriptorPoolTypeConfig},
    },
    device::VDevice,
};
use ash::vk;

pub struct MaterialsManager {
    descriptor_pool: VDescriptorPool,
    descriptor_sets: Vec<VDescriptorSet>,
}

impl MaterialsManager {
    pub fn new(v_device: &VDevice) -> Self {
        Self {
            descriptor_pool: VDescriptorPool::new(
                v_device,
                VDescriptorPoolConfig {
                    types: vec![VDescriptorPoolTypeConfig {
                        descriptor_type: vk::DescriptorType::COMBINED_IMAGE_SAMPLER,
                        count: 100,
                    }],
                    max_sets: 100,
                },
            ),
            descriptor_sets: vec![],
        }
    }

    pub fn get_set_at(&self, index: usize) -> &VDescriptorSet {
        return &self.descriptor_sets[index];
    }

    pub fn allocate_material(
        &mut self,
        v_device: &VDevice,
        v_layout: &VDescriptorSetLayout,
    ) -> usize {
        self.descriptor_sets.push(VDescriptorSet::new(
            v_device,
            &self.descriptor_pool,
            v_layout,
        ));
        return self.descriptor_sets.len() - 1;
    }

    pub fn destroy(&self, v_device: &VDevice) {
        self.descriptor_pool.destroy(v_device);
    }
}
