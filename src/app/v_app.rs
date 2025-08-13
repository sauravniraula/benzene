use ash::vk::{self, Extent2D};
use nalgebra::{Matrix4, Point3, Vector3};
use std::time::{Duration, SystemTime};

use crate::{
    app::{game_objects::VModel, renderable_app::RenderableApp},
    constants::MAX_FRAMES_IN_FLIGHT,
    core::{
        backend::VBackend,
        descriptor::{VDescriptorPool, VDescriptorSets},
        memory::VUniformBuffer,
    },
};

pub struct UniformBufferObject {
    model: Matrix4<f32>,
    view: Matrix4<f32>,
    proj: Matrix4<f32>,
}

pub struct VApp {
    pub model: VModel,
    pub uniform_buffers: Vec<VUniformBuffer<UniformBufferObject>>,
    descriptor_pool: VDescriptorPool,
    descriptor_sets: VDescriptorSets,
    start_time: SystemTime,
}

impl VApp {
    pub fn new(v_backend: &VBackend) -> Self {
        // Model
        let model = VModel::new(&v_backend);

        // Uniforms
        let uniform_buffers = VApp::create_uniform_buffers(v_backend);

        // Descriptor Pool and Sets
        let descriptor_pool = VDescriptorPool::new(&v_backend.v_device);
        let descriptor_sets = VDescriptorSets::new(
            &v_backend.v_device,
            &descriptor_pool,
            &v_backend.basic_rendering_system.descriptor_layouts[0],
            MAX_FRAMES_IN_FLIGHT,
        );
        descriptor_sets.bind_all(
            &v_backend.v_device,
            uniform_buffers.iter().map(|e| &e.v_buffer).collect(),
        );

        Self {
            model,
            uniform_buffers,
            descriptor_pool,
            descriptor_sets,
            start_time: SystemTime::now(),
        }
    }

    pub fn create_uniform_buffers(
        v_backend: &VBackend,
    ) -> Vec<VUniformBuffer<UniformBufferObject>> {
        let uniform_buffers: Vec<VUniformBuffer<_>> = (0..MAX_FRAMES_IN_FLIGHT)
            .map(|_| {
                let mut u = VUniformBuffer::new(v_backend);
                u.v_buffer.map_memory(v_backend);
                u
            })
            .collect();
        uniform_buffers
    }

    pub fn update_uniform_buffer(&self, image_extent: Extent2D, frame_index: usize) {
        let duration = SystemTime::now()
            .duration_since(self.start_time)
            .expect("failed to get duration");
        let duration_secs = duration.as_secs_f32();

        let mut ubo_model = Matrix4::identity();
        let mut ubo_view = Matrix4::identity();
        let mut ubo_proj = Matrix4::identity();
        ubo_model = Matrix4::<f32>::new_rotation(Vector3::<f32>::new(
            0.0,
            duration_secs * 40_f32.to_radians(),
            0.0,
        ));
        ubo_view = Matrix4::<f32>::look_at_rh(
            &Point3::<f32>::new(0.0, 0.0, 5.0),
            &Point3::<f32>::new(0.0, 0.0, 0.0),
            &Vector3::<f32>::new(0.0, 1.0, 0.0),
        );
        ubo_proj = Matrix4::<f32>::new_perspective(
            (image_extent.width as f32) / (image_extent.height as f32),
            45_f32.to_radians(),
            1.0,
            100.0,
        );
        // ubo_proj = Matrix4::new_orthographic(-1.0, 1.0, 1.0, -1.0, 1.0, 100.0);
        let vk_remap = Matrix4::<f32>::new(
            1.0, 0.0, 0.0, 0.0, 0.0, -1.0, 0.0, 0.0, 0.0, 0.0, 0.5, 0.5, 0.0, 0.0, 0.0, 1.0,
        );
        ubo_proj = vk_remap * ubo_proj;
        let ubo = UniformBufferObject {
            model: ubo_model,
            view: ubo_view,
            proj: ubo_proj,
        };
        self.uniform_buffers[frame_index].copy(&ubo);
    }

    pub fn destroy(&self, v_backend: &VBackend) {
        self.descriptor_pool.destroy(&v_backend.v_device);
        self.model.destroy(v_backend);
        for each in self.uniform_buffers.iter() {
            each.destroy(v_backend);
        }
    }
}

impl RenderableApp for VApp {
    fn render_app(
        &mut self,
        v_backend: &VBackend,
        command_buffer: vk::CommandBuffer,
        image_index: usize,
        frame_index: usize,
        _duration: Duration,
    ) {
        self.update_uniform_buffer(v_backend.v_swapchain.image_extent, frame_index);
        v_backend.basic_rendering_system.render(
            &v_backend.v_device,
            command_buffer,
            image_index,
            &[self.model.v_buffer.buffer],
            self.model.i_buffer.buffer,
            self.model.indices.len() as u32,
            self.descriptor_sets.sets[0],
        );
    }
}
