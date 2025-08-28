use ash::vk;
// use std::collections::HashMap; // no longer used with explicit type counts

use crate::vulkan_backend::{descriptor::config::VDescriptorPoolConfig, device::VDevice};

pub struct VDescriptorPool {
    pub pool: vk::DescriptorPool,
}

impl VDescriptorPool {
    pub fn new(v_device: &VDevice, config: VDescriptorPoolConfig) -> Self {
        let pool_sizes: Vec<vk::DescriptorPoolSize> = config
            .types
            .iter()
            .map(|t| {
                vk::DescriptorPoolSize::default()
                    .ty(t.descriptor_type)
                    .descriptor_count(t.count)
            })
            .collect();

        let pool_info = vk::DescriptorPoolCreateInfo::default()
            .pool_sizes(&pool_sizes)
            .max_sets(config.max_sets);

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
