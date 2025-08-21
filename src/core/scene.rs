use crate::{
    core::{
        camera::Camera,
        gpu::{
            game_object::GameObject,
            global_uniform::{GlobalUniform, GlobalUniformObject},
            recordable::{RecordContext, Recordable},
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
use nalgebra::{Matrix4, Vector3};

pub struct Scene {
    // Descriptor Sets
    global_uniform_set: VDescriptorSets,
    image_sampler_set: VDescriptorSets,

    camera: Option<Camera>,
    global_uniform: GlobalUniform,
    game_objects: Vec<GameObject>,
    texture: ImageTexture,
    last_extent: Option<vk::Extent2D>,
}

impl Scene {
    pub fn new(v_backend: &VBackend, scene_render: &SceneRender) -> Self {
        let global_uniform_set =
            scene_render.get_global_uniform_descriptor_set(&v_backend.v_device);
        let global_uniform = GlobalUniform::new(v_backend);

        let image_sampler_set = scene_render.get_image_sampler_descriptor_set(&v_backend.v_device);
        let texture = ImageTexture::new(
            v_backend,
            "assets/textures/cracked-dirt512x512.jpg",
            vk::Format::R8G8B8A8_SRGB,
        );
        {
            let mut batch = VDescriptorWriteBatch::new();
            global_uniform.queue_descriptor_writes(&global_uniform_set, &mut batch);
            texture.queue_descriptor_writes(&image_sampler_set, &mut batch);

            batch.flush(&v_backend.v_device);
        }

        let mut scene = Self {
            global_uniform_set,
            image_sampler_set,
            camera: None,
            global_uniform,
            game_objects: Vec::new(),
            texture,
            last_extent: Some(v_backend.v_swapchain.image_extent),
        };
        let (view, projection) = (Matrix4::identity(), Matrix4::identity());
        let light_direction = Vector3::<f32>::new(-1.0, -1.0, -0.5);
        let uniform = GlobalUniformObject {
            view,
            projection,
            light_direction,
        };
        scene.global_uniform.upload_all(&uniform);

        scene
    }

    pub fn handle_window_event(&mut self, event: &WindowEvent) {
        if let Some(camera) = &mut self.camera {
            camera.handle_window_event(event);
        }
    }

    pub fn pre_render(&mut self, image_extent: vk::Extent2D, dt: f32) {
        let extent_changed = match self.last_extent {
            Some(prev) => prev.width != image_extent.width || prev.height != image_extent.height,
            None => true,
        };
        if let Some(camera) = &mut self.camera {
            camera.update(image_extent, dt);
            if extent_changed || camera.take_dirty() {
                let (view, projection) = camera.view_projection(image_extent);
                let light_direction = Vector3::<f32>::new(-1.0, -1.0, -0.5);
                let uniform = GlobalUniformObject {
                    view,
                    projection,
                    light_direction,
                };
                self.global_uniform.upload_all(&uniform);
            }
        }
        self.last_extent = Some(image_extent);

        // Update Game Objects
        for game_object in self.game_objects.iter_mut() {
            game_object.pre_render();
        }
    }

    pub fn destroy(&self, v_backend: &VBackend) {
        if let Some(camera) = &self.camera {
            camera.destroy();
        }
        self.global_uniform.destroy(v_backend);
        for each in self.game_objects.iter() {
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
                let light_direction = Vector3::<f32>::new(-1.0, -1.0, -0.5);
                let uniform = GlobalUniformObject {
                    view,
                    projection,
                    light_direction,
                };
                self.global_uniform.upload_all(&uniform);
            }
        }
    }
    pub fn detach_camera(&mut self) {
        self.camera = None;
    }
    pub fn add_game_object(&mut self, game_object: GameObject) {
        self.game_objects.push(game_object);
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
                &[
                    self.global_uniform_set.sets[ctx.frame_index],
                    self.image_sampler_set.sets[0],
                ],
                &[],
            );

            for game_object in self.game_objects.iter() {
                game_object.record(ctx);
            }
        }
    }
}
