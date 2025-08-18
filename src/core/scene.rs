use crate::{
    core::{
        game_objects::camera::Camera,
        gpu::{
            model::Model,
            resources::global_uniform::{GlobalUniform, GlobalUniformObject},
        },
        rendering::{
            recordable::{Drawable, Recordable},
            scene_render::SceneRender,
        },
        resources::image::Image,
    },
    vulkan_backend::{backend::VBackend, rendering::RecordContext},
};
use ash::vk;
use glfw::WindowEvent;
use nalgebra::Matrix4;

pub struct Scene {
    camera: Option<Camera>,
    global_uniform: GlobalUniform,
    models: Vec<Model>,
    last_extent: Option<vk::Extent2D>,
}

impl Scene {
    pub fn new(v_backend: &VBackend, scene_render: &SceneRender) -> Self {
        // Initializing Global Uniform
        let global_uniform_sets =
            scene_render.get_global_uniform_descriptor_set(&v_backend.v_device);
        let global_uniform = GlobalUniform::new(v_backend, global_uniform_sets);
        global_uniform.bind_buffers(v_backend);
        let mut scene = Self {
            camera: None,
            global_uniform,
            models: Vec::new(),
            last_extent: Some(v_backend.v_swapchain.image_extent),
        };
        let (view, projection) = (Matrix4::identity(), Matrix4::identity());
        let uniform = GlobalUniformObject { view, projection };
        scene.global_uniform.upload_all(&uniform);

        // Image Texture
        let texture = Image::new(v_backend, "assets/textures/cracked-dirt512x512.jpg");

        scene
    }

    pub fn handle_window_event(&mut self, event: &WindowEvent) {
        if let Some(camera) = &mut self.camera {
            camera.handle_window_event(event);
        }
    }

    pub fn update(&mut self, frame_index: usize, image_extent: vk::Extent2D, dt: f32) {
        let extent_changed = match self.last_extent {
            Some(prev) => prev.width != image_extent.width || prev.height != image_extent.height,
            None => true,
        };
        if let Some(camera) = &mut self.camera {
            camera.update(frame_index, image_extent, dt);
            if extent_changed || camera.take_dirty() {
                let (view, projection) = camera.view_projection(image_extent);
                let uniform = GlobalUniformObject { view, projection };
                self.global_uniform.upload(frame_index, &uniform);
            }
        }
        self.last_extent = Some(image_extent);
    }

    pub fn destroy(&self, v_backend: &VBackend) {
        if let Some(camera) = &self.camera {
            camera.destroy();
        }
        self.global_uniform.destroy(v_backend);
        for each in self.models.iter() {
            each.destroy(v_backend);
        }
    }
}

impl Scene {
    pub fn attach_camera(&mut self, camera: Camera) {
        self.camera = Some(camera);
        if let Some(extent) = self.last_extent {
            if let Some(cam) = &self.camera {
                let (view, projection) = cam.view_projection(extent);
                let uniform = GlobalUniformObject { view, projection };
                self.global_uniform.upload_all(&uniform);
            }
        }
    }
    pub fn detach_camera(&mut self) {
        self.camera = None;
    }
    pub fn add_model(&mut self, model: Model) {
        self.models.push(model);
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
                &[self.global_uniform.descriptor_set(ctx.frame_index)],
                &[],
            );

            for model in self.models.iter() {
                model.draw(ctx.v_device, ctx.cmd);
            }
        }
    }
}
