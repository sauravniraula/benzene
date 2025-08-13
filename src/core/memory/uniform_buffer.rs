use std::marker::PhantomData;

use crate::core::{
    backend::VBackend,
    memory::{VBuffer, VBufferConfig, VBufferState},
};
use ash::vk;

pub struct VUniformBuffer<T> {
    pub v_buffer: VBuffer,
    marker: PhantomData<T>,
}

impl<T> VUniformBuffer<T> {
    pub fn new(v_backend: &VBackend) -> Self {
        let v_buffer = VBuffer::new(
            v_backend,
            VBufferConfig {
                size: size_of::<T>() as u64,
                usage: vk::BufferUsageFlags::UNIFORM_BUFFER,
                sharing_mode: v_backend.v_device.buffer_sharing_mode,
                queue_families: Some(v_backend.v_device.buffer_queue_family_indices.clone()),
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
        if let VBufferState::MAPPED(address) = self.v_buffer.state {
            unsafe {
                let src = data as *const T as *const u8;
                std::ptr::copy_nonoverlapping(src, address, size_of::<T>());
            }
        }
    }

    pub fn destroy(&self, v_backend: &VBackend) {
        self.v_buffer.destroy(v_backend);
    }
}
