use ash::vk::{self, Extent2D};
use nalgebra::{Matrix4, Point3, Vector3};

use crate::{
    app::renderable_app::RenderableApp,
    constants::MAX_FRAMES_IN_FLIGHT,
    core::{
        backend::VBackend,
        game_objects::models::VModel,
        memory::{VBuffer, VBufferConfig, VBufferState},
    },
};

pub struct UniformBufferObject {
    model: Matrix4<f32>,
    view: Matrix4<f32>,
    proj: Matrix4<f32>,
}

pub struct VApp {
    pub triangle: VModel,
    pub uniform_buffers: Vec<VBuffer>,
}

impl VApp {
    pub fn new(v_backend: &VBackend) -> Self {
        // Triangle Model
        let triangle = VModel::new(&v_backend);

        // Uniforms
        let uniform_buffers = VApp::create_uniform_buffers(v_backend);

        Self {
            triangle,
            uniform_buffers,
        }
    }

    pub fn create_uniform_buffers(v_backend: &VBackend) -> Vec<VBuffer> {
        let uniform_buffers: Vec<VBuffer> = (0..MAX_FRAMES_IN_FLIGHT)
            .map(|_| {
                let mut buffer = VBuffer::new(
                    v_backend,
                    VBufferConfig {
                        size: size_of::<UniformBufferObject>() as u64,
                        usage: vk::BufferUsageFlags::UNIFORM_BUFFER,
                        sharing_mode: v_backend.v_device.buffer_sharing_mode,
                        queue_families: Some(
                            v_backend.v_device.buffer_queue_family_indices.clone(),
                        ),
                        memory_property: vk::MemoryPropertyFlags::HOST_VISIBLE
                            | vk::MemoryPropertyFlags::HOST_COHERENT,
                    },
                );
                buffer.map_memory(v_backend);
                buffer
            })
            .collect();
        uniform_buffers
    }

    pub fn update_uniform_buffer(&self, image_extent: Extent2D, current_frame: u32) {
        let ubo_model =
            Matrix4::<f32>::new_rotation(Vector3::<f32>::new(0.0, 0.0, 0.01_f32.to_radians()));
        let ubo_view = Matrix4::<f32>::look_at_lh(
            &Point3::<f32>::new(2.0, 2.0, 2.0),
            &Point3::<f32>::new(0.0, 0.0, 0.0),
            &Vector3::<f32>::new(0.0, 0.0, 1.0),
        );
        let ubo_proj = Matrix4::<f32>::new_perspective(
            (image_extent.width as f32) / (image_extent.height as f32),
            45_f32.to_radians(),
            0.1,
            10.0,
        );
        let ubo = UniformBufferObject {
            model: ubo_model,
            view: ubo_view,
            proj: ubo_proj,
        };

        if let VBufferState::MAPPED(address) = self.uniform_buffers[current_frame as usize].state {
            unsafe {
                let src = &ubo as *const UniformBufferObject as *const u8;
                std::ptr::copy_nonoverlapping(src, address, size_of::<UniformBufferObject>());
            }
        }
    }

    pub fn destroy(&self, v_backend: &VBackend) {
        self.triangle.destroy(v_backend);
    }
}

impl RenderableApp for VApp {
    fn render_app(
        &self,
        v_backend: &VBackend,
        command_buffer: vk::CommandBuffer,
        image_index: usize,
    ) {
        v_backend.basic_rendering_system.render(
            &v_backend.v_device,
            command_buffer,
            image_index,
            &[self.triangle.v_buffer.buffer],
            self.triangle.i_buffer.buffer,
            self.triangle.indices.len() as u32,
        );
    }
}
