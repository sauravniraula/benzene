use ash::vk;

use crate::vulkan_backend::device::VDevice;

pub struct VDescriptorPool {
    pub pool: vk::DescriptorPool,
}

impl VDescriptorPool {
    pub fn new(v_device: &VDevice, count: usize) -> Self {
        let pool_sizes = [vk::DescriptorPoolSize::default()
            .ty(vk::DescriptorType::UNIFORM_BUFFER)
            .descriptor_count(count as u32)];

        let pool_info = vk::DescriptorPoolCreateInfo::default()
            .pool_sizes(&pool_sizes)
            .max_sets(3);

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
