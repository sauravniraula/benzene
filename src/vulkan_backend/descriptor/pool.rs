use ash::vk;
use std::collections::HashMap;

use crate::vulkan_backend::{descriptor::config::VDescriptorPoolConfig, device::VDevice};

pub struct VDescriptorPool {
    pub pool: vk::DescriptorPool,
}

impl VDescriptorPool {
    pub fn new(v_device: &VDevice, config: &VDescriptorPoolConfig) -> Self {
        let mut descriptor_type_counts: HashMap<vk::DescriptorType, u32> = HashMap::new();
        let mut total_sets: usize = 0;

        for set_config in &config.sets {
            total_sets += set_config.count;
            for binding in &set_config.layout.bindings {
                let entry = descriptor_type_counts
                    .entry(binding.descriptor_type)
                    .or_insert(0);
                *entry += (binding.count as u32) * (set_config.count as u32);
            }
        }

        let pool_sizes: Vec<vk::DescriptorPoolSize> = descriptor_type_counts
            .into_iter()
            .map(|(ty, total)| {
                vk::DescriptorPoolSize::default()
                    .ty(ty)
                    .descriptor_count(total)
            })
            .collect();

        let pool_info = vk::DescriptorPoolCreateInfo::default()
            .pool_sizes(&pool_sizes)
            .max_sets(total_sets as u32);

        let pool = unsafe {
            v_device
                .device
                .create_descriptor_pool(&pool_info, None)
                .expect("failed to create descriptor pool")
        };

        Self { pool }
    }

    pub fn destroy(&self, v_device: &VDevice) {
        unsafe {
            v_device.device.destroy_descriptor_pool(self.pool, None);
        }
    }
}
