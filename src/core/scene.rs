use crate::{
    constants::MAX_FRAMES_IN_FLIGHT,
    core::game_objects::camera::Camera,
    core::gpu::{model::Model, resources::camera_gpu::CameraGpu},
    core::rendering::{scene_render::SceneRender, recordable::{Drawable, Recordable}},
    core::resources::primitives::{cube::Cube, plane::Plane},
    vulkan_backend::{backend::VBackend, rendering::RecordContext},
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
        let models = vec![Plane::create_model(v_backend), Cube::create_model(v_backend)];

        Self { camera, camera_gpu, models }
    }

    pub fn handle_window_event(&mut self, event: &WindowEvent) {
        self.camera.handle_window_event(event);
    }

    pub fn update(&mut self, frame_index: usize, image_extent: vk::Extent2D, dt: f32) {
        self.camera.update(frame_index, image_extent, dt);
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
    fn record(&self, ctx: &RecordContext) {
        unsafe {
            ctx.v_device.device.cmd_bind_descriptor_sets(
                ctx.cmd,
                vk::PipelineBindPoint::GRAPHICS,
                ctx.pipeline_layout,
                0,
                &[self.camera_gpu.descriptor_set(ctx.frame_index)],
                &[],
            );

            for model in self.models.iter() {
                model.draw(ctx.v_device, ctx.cmd);
            }
        }
    }
}
