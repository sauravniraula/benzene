use ash::vk;
use memoffset::offset_of;
use nalgebra::Vector4;

use crate::vulkan_backend::{
    backend::VBackend,
    descriptor::{VDescriptorSets, VDescriptorWriteBatch},
    memory::VUniformBuffer,
};

pub struct PointLightUniformObject {
    pub points: [Vector4<f32>; 16],
    pub colors: [Vector4<f32>; 16],
}

pub struct PointLightUniform {
    uniform_buffer: VUniformBuffer<PointLightUniformObject>,
}

impl PointLightUniform {
    pub fn new(v_backend: &VBackend) -> Self {
        let mut uniform_buffer = VUniformBuffer::new(
            &v_backend.v_device,
            &v_backend.v_physical_device,
            &v_backend.v_memory_manager,
        );
        uniform_buffer.v_buffer.v_memory.map(
            &v_backend.v_device,
            &v_backend.v_memory_manager,
            0,
            uniform_buffer.v_buffer.config.size,
        );

        Self { uniform_buffer }
    }

    pub fn queue_descriptor_writes(
        &self,
        sets: &VDescriptorSets,
        batch: &mut VDescriptorWriteBatch,
    ) {
        sets.queue_buffer_all(
            batch,
            None,
            vk::DescriptorType::UNIFORM_BUFFER,
            &[&self.uniform_buffer.v_buffer],
        );
    }

    pub fn update(
        &mut self,
        v_backend: &VBackend,
        index: usize,
        point: &Vector4<f32>,
        color: &Vector4<f32>,
    ) {
        let vec4_size = size_of::<Vector4<f32>>();

        self.uniform_buffer.copy_region(
            &v_backend.v_device,
            &v_backend.v_physical_device,
            &v_backend.v_memory_manager,
            (offset_of!(PointLightUniformObject, points) + index * vec4_size) as u64,
            vec4_size as u64,
            point as *const Vector4<f32> as *const u8,
        );
        self.uniform_buffer.copy_region(
            &v_backend.v_device,
            &v_backend.v_physical_device,
            &v_backend.v_memory_manager,
            (offset_of!(PointLightUniformObject, colors) + index * vec4_size) as u64,
            vec4_size as u64,
            color as *const Vector4<f32> as *const u8,
        );
    }

    pub fn update_all(&mut self, v_backend: &VBackend, data: &PointLightUniformObject) {
        self.uniform_buffer.copy_region(
            &v_backend.v_device,
            &v_backend.v_physical_device,
            &v_backend.v_memory_manager,
            0,
            size_of::<PointLightUniformObject>() as u64,
            data as *const PointLightUniformObject as *const u8,
        );
    }

    pub fn destroy(&self, v_backend: &VBackend) {
        self.uniform_buffer
            .destroy(&v_backend.v_device, &v_backend.v_memory_manager);
    }
}
