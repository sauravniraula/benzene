use ash::vk;
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
            0,
            vk::DescriptorType::UNIFORM_BUFFER,
            &[&self.uniform_buffer.v_buffer],
        );
    }

    pub fn upload(&mut self, v_backend: &VBackend, data: &PointLightUniformObject) {
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
