use crate::{
    core::{
        camera::Camera,
        gpu::{
            global_uniform::{GlobalUniform, GlobalUniformObject},
            model::Model,
            recordable::{Drawable, RecordContext, Recordable},
            scene_render::SceneRender,
            texture::ImageTexture,
        },
    },
    vulkan_backend::{
        backend::VBackend,
        descriptor::{VDescriptorSets, VDescriptorWriteBatch},
    },
};
use ash::vk;
use glfw::WindowEvent;
use nalgebra::Matrix4;

pub struct Scene {
    // Descriptor Sets
    single_sets: VDescriptorSets,

    camera: Option<Camera>,
    global_uniform: GlobalUniform,
    models: Vec<Model>,
    texture: ImageTexture,
    last_extent: Option<vk::Extent2D>,
}

impl Scene {
    pub fn new(v_backend: &VBackend, scene_render: &SceneRender) -> Self {
        let single_sets = scene_render.get_descriptor_set(&v_backend.v_device);
        let global_uniform = GlobalUniform::new(v_backend);

        let texture = ImageTexture::new(
            v_backend,
            "assets/textures/cracked-dirt512x512.jpg",
            vk::Format::R8G8B8A8_SRGB,
        );
        {
            let mut batch = VDescriptorWriteBatch::new();
            global_uniform.queue_descriptor_writes(&single_sets, &mut batch);
            texture.queue_descriptor_writes(&single_sets, &mut batch);

            batch.flush(&v_backend.v_device);
        }

        let mut scene = Self {
            single_sets,
            camera: None,
            global_uniform,
            models: Vec::new(),
            texture,
            last_extent: Some(v_backend.v_swapchain.image_extent),
        };
        let (view, projection) = (Matrix4::identity(), Matrix4::identity());
        let uniform = GlobalUniformObject { view, projection };
        scene.global_uniform.upload_all(&uniform);

        scene
    }

    pub fn handle_window_event(&mut self, event: &WindowEvent) {
        if let Some(camera) = &mut self.camera {
            camera.handle_window_event(event);
        }
    }

    pub fn update(&mut self, image_extent: vk::Extent2D, dt: f32) {
        let extent_changed = match self.last_extent {
            Some(prev) => prev.width != image_extent.width || prev.height != image_extent.height,
            None => true,
        };
        if let Some(camera) = &mut self.camera {
            camera.update(image_extent, dt);
            if extent_changed || camera.take_dirty() {
                let (view, projection) = camera.view_projection(image_extent);
                let uniform = GlobalUniformObject { view, projection };
                self.global_uniform.upload_all(&uniform);
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
        self.texture.destroy(v_backend);
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
                &[self.single_sets.sets[ctx.frame_index]],
                &[],
            );

            for model in self.models.iter() {
                model.draw(ctx.v_device, ctx.cmd);
            }
        }
    }
}
