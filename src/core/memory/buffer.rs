use ash::vk;

use crate::core::{
    device::{VDevice, VPhysicalDevice},
    memory::{VAllocateMemoryConfig, VBufferConfig, VMemoryManager},
};

pub struct VBuffer {
    pub buffer: vk::Buffer,
    pub memory_requirements: vk::MemoryRequirements,
    pub memory: vk::DeviceMemory,
}

impl VBuffer {
    pub fn new(
        v_physical_device: &VPhysicalDevice,
        v_device: &VDevice,
        config: VBufferConfig,
    ) -> Self {
        let buffer_info = vk::BufferCreateInfo::default()
            .size(config.size)
            .usage(vk::BufferUsageFlags::VERTEX_BUFFER)
            .sharing_mode(vk::SharingMode::EXCLUSIVE);

        let buffer = unsafe {
            v_device
                .device
                .create_buffer(&buffer_info, None)
                .expect("failed to create buffer")
        };

        let memory_requirements = unsafe { v_device.device.get_buffer_memory_requirements(buffer) };

        let memory = VMemoryManager::allocate_memory(
            v_physical_device,
            v_device,
            VAllocateMemoryConfig {
                size: memory_requirements.size,
                memory_type: memory_requirements.memory_type_bits,
                properties: vk::MemoryPropertyFlags::HOST_VISIBLE
                    | vk::MemoryPropertyFlags::HOST_COHERENT,
            },
        );

        unsafe {
            v_device
                .device
                .bind_buffer_memory(buffer, memory, 0)
                .expect("failed to bind buffer memory")
        };

        Self {
            buffer,
            memory_requirements,
            memory,
        }
    }

    pub fn copy_to_buffer<T>(&self, v_device: &VDevice, data: &T) {
        VMemoryManager::copy_to_host_visible(v_device, self, data);
    }
}
