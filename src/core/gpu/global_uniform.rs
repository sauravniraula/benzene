use ash::vk;
use nalgebra::{Matrix4, Vector4};

use crate::{
    constants::MAX_FRAMES_IN_FLIGHT,
    vulkan_backend::{
        backend::VBackend,
        descriptor::{VDescriptorSets, VDescriptorWriteBatch},
        memory::VUniformBuffer,
    },
};

pub struct GlobalUniformObject {
    pub view: Matrix4<f32>,
    pub projection: Matrix4<f32>,
    pub point_light: Vector4<f32>,
}

pub struct GlobalUniform {
    uniform_buffers: Vec<VUniformBuffer<GlobalUniformObject>>,
}

impl GlobalUniform {
    pub fn new(v_backend: &VBackend) -> Self {
        let uniform_buffers: Vec<VUniformBuffer<_>> = (0..MAX_FRAMES_IN_FLIGHT)
            .map(|_| {
                let mut u = VUniformBuffer::new(
                    &v_backend.v_device,
                    &v_backend.v_physical_device,
                    &v_backend.v_memory_manager,
                );
                u.v_buffer
                    .map_memory(&v_backend.v_device, &v_backend.v_memory_manager);
                u
            })
            .collect();

        Self { uniform_buffers }
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

    pub fn upload(&mut self, frame_index: usize, data: &GlobalUniformObject) {
        self.uniform_buffers[frame_index].copy(data);
    }

    pub fn upload_all(&mut self, data: &GlobalUniformObject) {
        for frame_index in 0..MAX_FRAMES_IN_FLIGHT {
            self.upload(frame_index, data);
        }
    }

    pub fn destroy(&self, v_backend: &VBackend) {
        for each in self.uniform_buffers.iter() {
            each.destroy(&v_backend.v_device, &v_backend.v_memory_manager);
        }
    }
}
