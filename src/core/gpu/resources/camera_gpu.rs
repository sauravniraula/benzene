use ash::vk;

use crate::{
    constants::MAX_FRAMES_IN_FLIGHT,
    core::game_objects::camera::CameraUniform,
    vulkan_backend::{
        backend::VBackend,
        descriptor::VDescriptorSets,
        memory::VUniformBuffer,
    },
};

pub struct CameraGpu {
    uniform_buffers: Vec<VUniformBuffer<CameraUniform>>,
    descriptor_sets: VDescriptorSets,
}

impl CameraGpu {
    pub fn new_with_sets(v_backend: &VBackend, descriptor_sets: VDescriptorSets) -> Self {
        let uniform_buffers: Vec<VUniformBuffer<_>> = (0..MAX_FRAMES_IN_FLIGHT)
            .map(|_| {
                let mut u = VUniformBuffer::new(v_backend);
                u.v_buffer.map_memory(v_backend);
                u
            })
            .collect();

        let this = Self { uniform_buffers, descriptor_sets };
        this
    }

    pub fn bind_buffers(&self, v_backend: &VBackend) {
        self.descriptor_sets.bind_all(
            &v_backend.v_device,
            self.uniform_buffers.iter().map(|e| &e.v_buffer).collect(),
        );
    }

    pub fn upload(&mut self, frame_index: usize, data: &CameraUniform) {
        self.uniform_buffers[frame_index].copy(data);
    }

    pub fn descriptor_set(&self, frame_index: usize) -> vk::DescriptorSet {
        self.descriptor_sets.sets[frame_index]
    }

    pub fn destroy(&self, v_backend: &VBackend) {
        for each in self.uniform_buffers.iter() {
            each.destroy(v_backend);
        }
    }
}


