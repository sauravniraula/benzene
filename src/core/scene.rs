use crate::{
    constants::MAX_FRAMES_IN_FLIGHT,
    core::game_objects::camera::Camera,
    core::gpu::{model::Model, resources::camera_gpu::CameraGpu, scene_render::SceneRender},
    core::resources::primitives::cube::Cube,
    vulkan_backend::{
        backend::VBackend,
        device::VDevice,
        rendering::{Drawable, Recordable},
    },
};
use ash::vk;
use glfw::WindowEvent;

pub struct Scene {
    camera: Camera,
    camera_gpu: CameraGpu,
    models: Vec<Model>,
}

impl Scene {
    pub fn new(v_backend: &VBackend, scene_render: &SceneRender) -> Self {
        let layout = scene_render.get_descriptor_set_layout_at_binding(0);

        let camera = Camera::new();
        let camera_sets = scene_render.allocate_descriptor_sets(&v_backend.v_device, layout, MAX_FRAMES_IN_FLIGHT);
        let camera_gpu = CameraGpu::new_with_sets(v_backend, camera_sets);
        camera_gpu.bind_buffers(v_backend);
        let models = vec![Cube::create_model(v_backend)];

        Self { camera, camera_gpu, models }
    }

    pub fn handle_window_event(&mut self, event: &WindowEvent) {
        self.camera.handle_window_event(event);
    }

    pub fn update(&mut self, frame_index: usize, image_extent: vk::Extent2D) {
        self.camera.update(frame_index, image_extent);
        let uniform = self.camera.get_uniform(image_extent);
        self.camera_gpu.upload(frame_index, &uniform);
    }

    pub fn destroy(&self, v_backend: &VBackend) {
        self.camera.destroy();
        self.camera_gpu.destroy(v_backend);
        for each in self.models.iter() {
            each.destroy(v_backend);
        }
    }
}

impl Recordable for Scene {
    fn record(
        &self,
        v_device: &VDevice,
        command_buffer: vk::CommandBuffer,
        frame_index: usize,
        pipeline_layouts: &[vk::PipelineLayout],
    ) {
        unsafe {
            v_device.device.cmd_bind_descriptor_sets(
                command_buffer,
                vk::PipelineBindPoint::GRAPHICS,
                pipeline_layouts[0],
                0,
                &[self.camera_gpu.descriptor_set(frame_index)],
                &[],
            );

            for model in self.models.iter() {
                model.draw(v_device, command_buffer);
            }
        }
    }
}
