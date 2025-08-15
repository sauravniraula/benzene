use ash::vk::Extent2D;
use nalgebra::{Matrix4, Point3, Vector3};

use crate::{
    constants::MAX_FRAMES_IN_FLIGHT,
    vulkan_backend::{
        backend::VBackend,
        descriptor::{VDescriptorPool, VDescriptorSets},
        memory::VUniformBuffer,
    },
};

pub struct CameraUniform {
    view: Matrix4<f32>,
    projection: Matrix4<f32>,
}
pub struct Camera {
    uniform_buffers: Vec<VUniformBuffer<CameraUniform>>,
}

impl Camera {
    pub fn new(v_backend: &VBackend, v_descriptor_pool: &VDescriptorPool) -> Self {
        let uniform_buffers: Vec<VUniformBuffer<_>> = (0..MAX_FRAMES_IN_FLIGHT)
            .map(|_| {
                let mut u = VUniformBuffer::new(v_backend);
                u.v_buffer.map_memory(v_backend);
                u
            })
            .collect();

        let descriptor_sets = VDescriptorSets::new(
            &v_backend.v_device,
            v_descriptor_pool,
            &v_backend.basic_rendering_system.descriptor_layouts[0],
            MAX_FRAMES_IN_FLIGHT,
        );
        descriptor_sets.bind_all(
            &v_backend.v_device,
            uniform_buffers.iter().map(|e| &e.v_buffer).collect(),
        );

        Self { uniform_buffers }
    }

    pub fn update_uniform_buffer(&self, frame_index: usize, image_extent: Extent2D) {
        let view = Matrix4::<f32>::look_at_rh(
            &Point3::<f32>::new(3.0, 1.0, 3.0),
            &Point3::<f32>::new(0.0, 0.0, 0.0),
            &Vector3::<f32>::new(0.0, 1.0, 0.0),
        );
        let mut projection = Matrix4::<f32>::new_perspective(
            (image_extent.width as f32) / (image_extent.height as f32),
            45_f32.to_radians(),
            1.0,
            100.0,
        );
        projection[(1, 1)] *= -1.0;
        let gu = CameraUniform { view, projection };
        self.uniform_buffers[frame_index].copy(&gu);
    }
}
