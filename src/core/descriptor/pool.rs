use ash::vk;

use crate::core::device::VDevice;

pub struct VDescriptorPool {
    pool: vk::DescriptorPool,
}

impl VDescriptorPool {
    pub fn new(v_device: &VDevice) -> Self {
        let pool_sizes = [vk::DescriptorPoolSize::default()
            .ty(vk::DescriptorType::UNIFORM_BUFFER)
            .descriptor_count(3)];

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
}
