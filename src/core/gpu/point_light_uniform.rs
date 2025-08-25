use ash::vk;
use nalgebra::Vector4;

use crate::vulkan_backend::{
    backend::VBackend,
    descriptor::{VDescriptorSets, VDescriptorWriteBatch},
    memory::VUniformBuffer,
};

pub struct PointLightUniformObject {
    // std140 requires vec4 alignment/stride for arrays. The w component is used as a flag/intensity.
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
        uniform_buffer
            .v_buffer
            .map_memory(&v_backend.v_device, &v_backend.v_memory_manager);

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

    pub fn upload(&mut self, data: &PointLightUniformObject) {
        self.uniform_buffer.copy(data);
    }

    pub fn destroy(&self, v_backend: &VBackend) {
        self.uniform_buffer
            .destroy(&v_backend.v_device, &v_backend.v_memory_manager);
    }
}
