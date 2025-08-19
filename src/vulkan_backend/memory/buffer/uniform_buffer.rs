use std::marker::PhantomData;

use crate::vulkan_backend::{
    device::{VDevice, VPhysicalDevice},
    memory::{VBuffer, VBufferConfig, VMemoryManager, VMemoryState},
};
use ash::vk;

pub struct VUniformBuffer<T> {
    pub v_buffer: VBuffer,
    marker: PhantomData<T>,
}

impl<T> VUniformBuffer<T> {
    pub fn new(
        v_device: &VDevice,
        v_physical_device: &VPhysicalDevice,
        v_memory_manager: &VMemoryManager,
    ) -> Self {
        let v_buffer = VBuffer::new(
            v_device,
            v_physical_device,
            v_memory_manager,
            VBufferConfig {
                size: size_of::<T>() as u64,
                usage: vk::BufferUsageFlags::UNIFORM_BUFFER,
                sharing_mode: v_device.buffer_sharing_mode,
                queue_families: Some(v_device.buffer_queue_family_indices.clone()),
                memory_property: vk::MemoryPropertyFlags::HOST_VISIBLE
                    | vk::MemoryPropertyFlags::HOST_COHERENT,
            },
        );

        Self {
            v_buffer,
            marker: PhantomData,
        }
    }

    pub fn copy(&self, data: &T) {
        if let VMemoryState::MAPPED(address) = self.v_buffer.v_memory.state {
            unsafe {
                let src = data as *const T as *const u8;
                std::ptr::copy_nonoverlapping(src, address, size_of::<T>());
            }
        }
    }

    pub fn destroy(&self, v_device: &VDevice, v_memory_manager: &VMemoryManager) {
        self.v_buffer.destroy(v_device, v_memory_manager);
    }
}


