use crate::core::{
    device::{VDevice, VPhysicalDevice},
    memory::VBuffer,
};
use ash::vk;

pub struct VMemoryManager {}

impl VMemoryManager {
    pub fn allocate_memory(
        v_physical_device: &VPhysicalDevice,
        v_device: &VDevice,
        config: super::VAllocateMemoryConfig,
    ) -> vk::DeviceMemory {
        let memory_type_index = v_physical_device
            .find_memory_type_index(config.memory_type, config.properties)
            .expect("failed to find memory type index");
        let alloc_info = vk::MemoryAllocateInfo::default()
            .allocation_size(config.size)
            .memory_type_index(memory_type_index);

        unsafe {
            v_device
                .device
                .allocate_memory(&alloc_info, None)
                .expect("failed to allocate memory")
        }
    }

    pub fn copy_to_host_visible<T>(v_device: &VDevice, v_buffer: &VBuffer, data: &T) {
        let destination = unsafe {
            v_device
                .device
                .map_memory(
                    v_buffer.memory,
                    0,
                    v_buffer.memory_requirements.size,
                    vk::MemoryMapFlags::empty(),
                )
                .expect("failed to map memory") as *mut T
        };
        let source: *const T = data;
        unsafe {
            std::ptr::copy(source, destination, 1);
            v_device.device.unmap_memory(v_buffer.memory);
        };
    }
}
