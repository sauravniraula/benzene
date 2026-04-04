use ash::vk;
use std::time::Duration;
use std::time::Instant;
use winit::window::Window;

use crate::core::ecs::entities::Entity;
use crate::core::ecs::types::{CursorMovedEvent, KeyboardInputEvent};
use crate::core::gpu::model::Model;
use crate::core::gpu::scene_render::RecordableScene;
use crate::log;
use crate::vulkan_backend::backend_event::VBackendEvent;
use crate::{
    core::{
        assets::{AssetStore, MaterialHandle, MeshHandle, TextureHandle},
        gpu::{
            materials_manager::MaterialsManager, scene_render::SceneRenderer, texture::ImageTexture,
        },
        scene::Scene,
    },
    vulkan_backend::{
        backend::VBackend, descriptor::VDescriptorWriteBatch, frame::context::VFrameRenderContext,
    },
};

pub struct GameEngine {
    v_backend: VBackend,
    scene_renderer: SceneRenderer,
    materials_manager: MaterialsManager,
    assets: AssetStore,
    active_scene: Option<Scene>,
    last_frame_instant: Instant,
    frame_count: usize,
    fps: usize,
    pub frame_time: Duration,
}

impl GameEngine {
    pub fn new(window: &Window) -> Self {
        let v_backend = VBackend::new(window);
        let scene_renderer = SceneRenderer::new(&v_backend);
        let materials_manager = MaterialsManager::new(&v_backend.v_device);

        let mut engine = Self {
            v_backend,
            scene_renderer,
            materials_manager,
            assets: AssetStore::new(),
            active_scene: None,
            last_frame_instant: Instant::now(),
            frame_count: 0,
            fps: 0,
            frame_time: Duration::new(0, 0),
        };

        engine.init();
        engine
    }

    fn init(&mut self) {
        let default_texture = ImageTexture::empty(&self.v_backend, vk::Format::R8G8B8A8_SRGB);
        let default_texture_handle = self.assets.insert_texture(default_texture);
        let sampler_layout = self.scene_renderer.get_image_sampler_layout();
        let default_material = self
            .materials_manager
            .allocate_material(&self.v_backend.v_device, sampler_layout);
        self.assets.default_material = default_material;

        let texture = self
            .assets
            .get_texture(default_texture_handle)
            .expect("default texture should exist");

        let mut batch_writer = VDescriptorWriteBatch::new();
        self.materials_manager
            .get_set_at(default_material)
            .queue_image(
                &mut batch_writer,
                vk::DescriptorType::COMBINED_IMAGE_SAMPLER,
                0,
                &texture.image_view,
                &texture.sampler,
                vk::ImageLayout::SHADER_READ_ONLY_OPTIMAL,
            );
        batch_writer.flush(&self.v_backend.v_device);
    }

    pub fn create_scene(&self) -> Scene {
        Scene::new(&self.v_backend, &self.scene_renderer)
    }

    pub fn get_active_scene(&mut self) -> &mut Scene {
        self.active_scene.as_mut().expect("no active scene")
    }

    pub fn set_active_scene(&mut self, scene: Scene) {
        self.active_scene = Some(scene);
    }

    pub fn enable_shadow_for_spot_light_3d(&mut self, entity: Entity) {
        let scene = self.active_scene.as_mut().expect("No active scene");
        scene.shadow_mapping.add_spot_light(&self.v_backend, entity);
    }

    pub fn disable_shadow_for_spot_light_3d(&mut self, entity: Entity) {
        let scene = self.active_scene.as_mut().expect("No active scene");
        scene
            .shadow_mapping
            .remove_spot_light(&self.v_backend, &entity);
    }

    pub fn load_mesh_from_obj(&mut self, obj_path: &str) -> MeshHandle {
        let mesh = Model::from_obj(&self.v_backend, obj_path);
        self.assets.insert_mesh(mesh)
    }

    pub fn load_texture_from_image(&mut self, image_path: &str) -> TextureHandle {
        let texture = ImageTexture::new(&self.v_backend, image_path, vk::Format::R8G8B8A8_SRGB);
        self.assets.insert_texture(texture)
    }

    pub fn create_material_from_texture(&mut self, texture: TextureHandle) -> MaterialHandle {
        let texture = self
            .assets
            .get_texture(texture)
            .expect("invalid texture handle passed to create_material_from_texture");
        let sampler_layout = self.scene_renderer.get_image_sampler_layout();
        let material = self
            .materials_manager
            .allocate_material(&self.v_backend.v_device, sampler_layout);

        let mut batch_writer = VDescriptorWriteBatch::new();
        self.materials_manager.get_set_at(material).queue_image(
            &mut batch_writer,
            vk::DescriptorType::COMBINED_IMAGE_SAMPLER,
            0,
            &texture.image_view,
            &texture.sampler,
            vk::ImageLayout::SHADER_READ_ONLY_OPTIMAL,
        );
        batch_writer.flush(&self.v_backend.v_device);
        material
    }

    pub fn unload_texture(&mut self, texture: TextureHandle) {
        if let Some(texture) = self.assets.remove_texture(texture) {
            texture.destroy(&self.v_backend);
        }
    }

    pub fn emit_update_framebuffers(&mut self, window: &Window) {
        self.v_backend.recreate_swapchain(window);
        let event = VBackendEvent::UpdateFramebuffers(
            &self.v_backend.v_device,
            &self.v_backend.v_swapchain,
        );
        self.scene_renderer.handle_backend_event(&event);
        if let Some(scene) = &mut self.active_scene {
            scene.handle_backend_event(&event);
        }
    }

    pub fn handle_keyboard_input(&mut self, event: &KeyboardInputEvent) {
        if let Some(scene) = &mut self.active_scene {
            scene.handle_keyboard_input(event);
        }
    }

    pub fn handle_cursor_moved(&mut self, event: &CursorMovedEvent) {
        if let Some(scene) = &mut self.active_scene {
            scene.handle_cursor_moved(event);
        }
    }

    pub fn reset_cursor_tracking(&mut self) {
        if let Some(scene) = &mut self.active_scene {
            scene.reset_cursor_tracking();
        }
    }

    pub fn pre_render(&mut self) {
        log!("Game Engine pre render");

        let current_instant = Instant::now();
        let dt = current_instant.duration_since(self.last_frame_instant);
        self.last_frame_instant = current_instant;

        self.fps = (1.0 / dt.as_secs_f64()) as usize;
        self.frame_count += 1;
        self.frame_time = dt;
        log!(format!("FPS: {}", self.fps));

        if let Some(scene) = &mut self.active_scene {
            scene.pre_render(&self.v_backend, dt.as_secs_f32());
        }
    }

    pub fn render(&mut self, window: &Window) {
        log!("Game Engine render");

        let render_result = self.v_backend.render(|info| self.render_scene(&info));

        if let Some(event) = self.v_backend.check_render_issues(window, render_result) {
            self.scene_renderer.handle_backend_event(&event);
            if let Some(scene) = &mut self.active_scene {
                scene.handle_backend_event(&event);
            }

            if let VBackendEvent::UpdateFramebuffers(..) = event {
                log!("Update framebuffer, frame: {}", self.frame_count);
            }
        }
    }

    fn render_scene(&self, ctx: &VFrameRenderContext) {
        if let Some(scene) = &self.active_scene {
            log!("Scene render started");
            let recordables: [&dyn RecordableScene; 1] = [scene];
            self.scene_renderer.render(
                &self.v_backend.v_device,
                &self.assets,
                &self.materials_manager,
                ctx,
                &recordables,
            );
            log!("Scene render end");
        } else {
            log!("No active scene found to render");
        }
    }

    pub fn destroy(mut self) {
        self.v_backend.v_device.wait_till_idle();
        if let Some(scene) = &mut self.active_scene {
            scene.destroy(&self.v_backend);
        }
        self.assets.destroy(&self.v_backend);
        self.scene_renderer.destroy(&self.v_backend.v_device);
        self.materials_manager.destroy(&self.v_backend.v_device);
        self.v_backend.destroy();
    }
}
