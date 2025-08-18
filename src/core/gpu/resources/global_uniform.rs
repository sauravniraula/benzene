use ash::vk;
use nalgebra::Matrix4;

use crate::{
    constants::MAX_FRAMES_IN_FLIGHT,
    vulkan_backend::{backend::VBackend, descriptor::VDescriptorSets, memory::VUniformBuffer},
};

pub struct GlobalUniformObject {
    pub view: Matrix4<f32>,
    pub projection: Matrix4<f32>,
}

pub struct GlobalUniform {
    uniform_buffers: Vec<VUniformBuffer<GlobalUniformObject>>,
    sets: VDescriptorSets,
}

impl GlobalUniform {
    pub fn new(v_backend: &VBackend, sets: VDescriptorSets) -> Self {
        let uniform_buffers: Vec<VUniformBuffer<_>> = (0..MAX_FRAMES_IN_FLIGHT)
            .map(|_| {
                let mut u = VUniformBuffer::new(v_backend);
                u.v_buffer.map_memory(v_backend);
                u
            })
            .collect();

        Self {
            uniform_buffers,
            sets,
        }
    }

    pub fn bind_buffers(&self, v_backend: &VBackend) {
        self.sets.bind_all(
            &v_backend.v_device,
            self.uniform_buffers.iter().map(|e| &e.v_buffer).collect(),
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

    pub fn descriptor_set(&self, frame_index: usize) -> vk::DescriptorSet {
        self.sets.sets[frame_index]
    }

    pub fn destroy(&self, v_backend: &VBackend) {
        for each in self.uniform_buffers.iter() {
            each.destroy(v_backend);
        }
    }
}
