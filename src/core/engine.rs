use ash::vk;
use std::time::Duration;
use std::{collections::HashMap, time::Instant};
use winit::window::Window;

use crate::core::ecs::entities::game_object::GameObject;
use crate::core::ecs::types::{CursorMovedEvent, KeyboardInputEvent};
use crate::core::gpu::scene_render::RecordableScene;
use crate::log;
use crate::vulkan_backend::backend_event::VBackendEvent;
use crate::{
    core::{
        ecs::components::{Material3D, Structure3D},
        gpu::{
            materials_manager::MaterialsManager, scene_render::SceneRenderer, texture::ImageTexture,
        },
        scene::Scene,
        utils::get_random_id,
    },
    shared::types::Id,
    vulkan_backend::{
        backend::VBackend, descriptor::VDescriptorWriteBatch, frame::context::VFrameRenderContext,
    },
};

pub struct GameEngine {
    // Core
    v_backend: VBackend,
    scene_renderer: SceneRenderer,
    materials_manager: MaterialsManager,

    // Resources
    textures: HashMap<Id, ImageTexture>,

    // State
    active_scene: Option<Scene>,
    last_frame_instant: Instant,
    frame_count: usize,
    fps: usize,
    frame_time: Duration,
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
            textures: HashMap::new(),
            active_scene: None,
            last_frame_instant: Instant::now(),
            frame_count: 0,
            fps: 0,
            frame_time: Duration::new(0, 0),
        };

        engine.init();

        engine
    }

    pub fn init(&mut self) {
        let default_texture = ImageTexture::empty(&self.v_backend, vk::Format::R8G8B8A8_SRGB);
        let default_texture_id = get_random_id();
        self.textures.insert(default_texture_id, default_texture);
        let sampler_layout = self.scene_renderer.get_image_sampler_layout();
        let default_material_index = self
            .materials_manager
            .allocate_material(&self.v_backend.v_device, sampler_layout);
        let default_material = Material3D {
            manager_index: default_material_index,
        };
        let mut batch_writer = VDescriptorWriteBatch::new();
        default_material.queue_descriptor_writes(
            &mut self.materials_manager,
            &self.textures[&default_texture_id],
            &mut batch_writer,
        );
        batch_writer.flush(&self.v_backend.v_device);
    }

    pub fn create_scene(&self) -> Scene {
        Scene::new(&self.v_backend, &self.scene_renderer)
    }

    pub fn get_active_scene(&mut self) -> &mut Scene {
        self.active_scene.as_mut().unwrap()
    }

    pub fn set_active_scene(&mut self, scene: Scene) {
        self.active_scene = Some(scene);
    }

    pub fn enable_shadow_for_spot_light_3d(&mut self, entity: &GameObject) {
        let scene = self.active_scene.as_mut().expect("No active scene");
        scene
            .shadow_mapping
            .add_spot_light(&self.v_backend, *entity.get_id());

        let _shadow_map = scene
            .shadow_mapping
            .spot_light_maps
            .get(entity.get_id())
            .unwrap();
        let _shadow_map_view = scene
            .shadow_mapping
            .spot_light_views
            .get(entity.get_id())
            .unwrap();

        // self.scene_render.v_shadow_rendering_system.add_framebuffer(
        //     &self.v_backend.v_device,
        //     *entity.get_id(),
        //     None,
        //     Some(shadow_map_view),
        //     shadow_map.config.get_extent_2d(),
        //     true,
        //     true,
        // );
    }

    pub fn disable_shadow_for_spot_light_3d(&mut self, entity: &GameObject) {
        let scene = self.active_scene.as_mut().expect("No active scene");
        scene
            .shadow_mapping
            .remove_spot_light(&self.v_backend, entity.get_id());
        // self.scene_render
        //     .v_shadow_rendering_system
        //     .remove_framebuffer(&self.v_backend.v_device, entity.get_id());
    }

    pub fn get_structure_3d_from_obj(&self, obj_path: &str) -> Structure3D {
        Structure3D::from_obj(&self.v_backend, obj_path)
    }

    pub fn load_texture_from_image(&mut self, image_path: &str) -> Id {
        let texture = ImageTexture::new(&self.v_backend, image_path, vk::Format::R8G8B8A8_SRGB);
        let id = get_random_id();
        self.textures.insert(id, texture);
        id
    }

    pub fn get_material_3d_from_texture(&mut self, texture: Id) -> Material3D {
        let texture = self
            .textures
            .get(&texture)
            .expect("invalid texture id passed to get_material_3d_from_texture");
        let sampler_layout = self.scene_renderer.get_image_sampler_layout();
        let allocated_sets_index = self
            .materials_manager
            .allocate_material(&self.v_backend.v_device, sampler_layout);

        let mut batch_writer = VDescriptorWriteBatch::new();

        let material = Material3D {
            manager_index: allocated_sets_index,
        };

        material.queue_descriptor_writes(&mut self.materials_manager, texture, &mut batch_writer);
        batch_writer.flush(&self.v_backend.v_device);
        material
    }

    pub fn unload_texture(&mut self, texture: Id) {
        if let Some(tex) = self.textures.remove(&texture) {
            tex.destroy(&self.v_backend);
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

    pub fn pre_render(&mut self) {
        log!("Game Engine pre render");

        let current_instant = Instant::now();
        let dt = current_instant.duration_since(self.last_frame_instant);
        self.last_frame_instant = current_instant;

        // Frame Count and FPS
        self.fps = (1.0 / dt.as_secs_f64()) as usize;
        self.frame_count += 1;
        self.frame_time = dt;
        log!(format!("FPS: {}", self.fps));

        // Pre-render the scene
        if let Some(scene) = &mut self.active_scene {
            scene.pre_render(&self.v_backend, dt.as_secs_f32());
        }
    }

    pub fn render(&mut self, window: &Window) {
        // render
        log!("Game Engine render");

        let render_result = self.v_backend.render(|info| self.render_scene(&info));

        // check render result
        if let Some(event) = self.v_backend.check_render_issues(window, render_result) {
            self.scene_renderer.handle_backend_event(&event);
            if let Some(scene) = &mut self.active_scene {
                scene.handle_backend_event(&event);
            }

            match event {
                VBackendEvent::UpdateFramebuffers(..) => {
                    log!("Update framebuffer, frame: {}", self.frame_count);
                }
                _ => (),
            }
        }
    }

    fn render_scene(&self, ctx: &VFrameRenderContext) {
        if let Some(scene) = &self.active_scene {
            log!("Scene render started");
            let recordables: [&dyn RecordableScene; 1] = [scene];
            self.scene_renderer.render(
                &self.v_backend.v_device,
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
        if let Some(scene) = &self.active_scene {
            scene.destroy(&self.v_backend);
        }
        // Destroy all engine-owned textures
        for (_, tex) in self.textures.drain() {
            tex.destroy(&self.v_backend);
        }
        self.scene_renderer.destroy(&self.v_backend.v_device);
        self.materials_manager.destroy(&self.v_backend.v_device);
        self.v_backend.destroy();
    }
}
