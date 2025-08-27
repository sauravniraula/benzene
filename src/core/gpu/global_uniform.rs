use ash::vk;
use memoffset::offset_of;
use nalgebra::{Matrix4, Vector4};

use crate::vulkan_backend::{
    backend::VBackend,
    descriptor::{VDescriptorSets, VDescriptorWriteBatch},
    memory::VUniformBuffer,
};

pub struct GlobalUniformObject {
    pub view: Matrix4<f32>,
    pub projection: Matrix4<f32>,
    pub ambient_color: Vector4<f32>,
}

pub struct GlobalUniform {
    count: usize,
    uniform_buffers: Vec<VUniformBuffer<GlobalUniformObject>>,
}

impl GlobalUniform {
    pub fn new(v_backend: &VBackend, count: usize) -> Self {
        let uniform_buffers: Vec<VUniformBuffer<_>> = (0..count)
            .map(|_| {
                let mut u = VUniformBuffer::new(
                    &v_backend.v_device,
                    &v_backend.v_physical_device,
                    &v_backend.v_memory_manager,
                );
                u.v_buffer.v_memory.map(
                    &v_backend.v_device,
                    &v_backend.v_memory_manager,
                    0,
                    u.v_buffer.config.size,
                );
                u
            })
            .collect();

        Self {
            count,
            uniform_buffers,
        }
    }

    pub fn queue_descriptor_writes(
        &self,
        sets: &VDescriptorSets,
        batch: &mut VDescriptorWriteBatch,
    ) {
        sets.queue_buffer_all(
            batch,
            0,
            vk::DescriptorType::UNIFORM_BUFFER,
            &self
                .uniform_buffers
                .iter()
                .map(|e| &e.v_buffer)
                .collect::<Vec<_>>(),
        );
    }

    pub fn update_view(&mut self, v_backend: &VBackend, index: usize, value: &Matrix4<f32>) {
        self.uniform_buffers[index].copy_region(
            &v_backend.v_device,
            &v_backend.v_physical_device,
            &v_backend.v_memory_manager,
            offset_of!(GlobalUniformObject, view) as u64,
            size_of::<Matrix4<f32>>() as u64,
            value as *const Matrix4<f32> as *const u8,
        );
    }

    pub fn update_projection(&mut self, v_backend: &VBackend, index: usize, value: &Matrix4<f32>) {
        self.uniform_buffers[index].copy_region(
            &v_backend.v_device,
            &v_backend.v_physical_device,
            &v_backend.v_memory_manager,
            offset_of!(GlobalUniformObject, projection) as u64,
            size_of::<Matrix4<f32>>() as u64,
            value as *const Matrix4<f32> as *const u8,
        );
    }

    pub fn update_ambient_color(
        &mut self,
        v_backend: &VBackend,
        index: usize,
        value: &Vector4<f32>,
    ) {
        self.uniform_buffers[index].copy_region(
            &v_backend.v_device,
            &v_backend.v_physical_device,
            &v_backend.v_memory_manager,
            offset_of!(GlobalUniformObject, ambient_color) as u64,
            size_of::<Vector4<f32>>() as u64,
            value as *const Vector4<f32> as *const u8,
        );
    }

    pub fn destroy(&self, v_backend: &VBackend) {
        for each in self.uniform_buffers.iter() {
            each.destroy(&v_backend.v_device, &v_backend.v_memory_manager);
        }
    }
}
