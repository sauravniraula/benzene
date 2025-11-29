use ash::vk;
use glfw::{Action, Key, WindowEvent};
use std::time::Instant;
use std::{collections::HashMap, time::Duration};

use crate::core::ecs::entities::game_object::GameObject;
use crate::{
    core::{
        ecs::{
            components::{Material3D, Structure3D},
            types::Id,
        },
        gpu::{
            materials_manager::MaterialsManager,
            scene_render::{SceneRender, SceneRenderRecordable},
            texture::ImageTexture,
        },
        scene::Scene,
        utils::get_random_id,
    },
    vulkan_backend::{
        backend::VBackend, descriptor::VDescriptorWriteBatch, rendering::info::VRenderInfo,
    },
    window::{Window, WindowConfig},
};

pub struct GameEngine {
    // Core
    window: Window,
    v_backend: VBackend,
    scene_render: SceneRender,
    materials_manager: MaterialsManager,

    // Resources
    textures: HashMap<Id, ImageTexture>,

    // State
    active_scene: Option<Scene>,
    last_frame_instant: Instant,
}

impl GameEngine {
    pub fn new() -> Self {
        let window = Window::new(WindowConfig::default());
        let v_backend = VBackend::new(&window);
        let scene_render = SceneRender::new(&v_backend);
        let materials_manager = MaterialsManager::new(&v_backend.v_device);

        let engine = Self {
            window,
            v_backend,
            scene_render,
            materials_manager,
            textures: HashMap::new(),
            active_scene: None,
            last_frame_instant: Instant::now(),
        };
        engine
    }

    pub fn init(&mut self) {
        let default_texture = ImageTexture::empty(&self.v_backend, vk::Format::R8G8B8A8_SRGB);
        let default_texture_id = get_random_id();
        self.textures.insert(default_texture_id, default_texture);
        let default_material_index = self.materials_manager.allocate_material(
            &self.v_backend.v_device,
            &self.scene_render.descriptor_set_layouts[2],
        );
        let default_material = Material3D {
            manager_index: default_material_index,
        };
        let mut batch_writer = VDescriptorWriteBatch::new();
        default_material.queue_descriptor_writes(
            &self.materials_manager,
            &self.textures[&default_texture_id],
            &mut batch_writer,
        );
        batch_writer.flush(&self.v_backend.v_device);
    }

    pub fn create_scene(&self) -> Scene {
        Scene::new(&self.v_backend, &self.scene_render)
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

        let shadow_map = scene
            .shadow_mapping
            .spot_light_maps
            .get(entity.get_id())
            .unwrap();
        let shadow_map_view = scene
            .shadow_mapping
            .spot_light_views
            .get(entity.get_id())
            .unwrap();

        self.scene_render.v_shadow_rendering_system.add_framebuffer(
            &self.v_backend.v_device,
            *entity.get_id(),
            None,
            Some(shadow_map_view),
            shadow_map.config.get_extent_2d(),
            true,
            true,
        );
    }

    pub fn disable_shadow_for_spot_light_3d(&mut self, entity: &GameObject) {
        let scene = self.active_scene.as_mut().expect("No active scene");
        scene
            .shadow_mapping
            .remove_spot_light(&self.v_backend, entity.get_id());
        self.scene_render
            .v_shadow_rendering_system
            .remove_framebuffer(&self.v_backend.v_device, entity.get_id());
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
        let allocated_sets_index = self.materials_manager.allocate_material(
            &self.v_backend.v_device,
            &self.scene_render.descriptor_set_layouts[2],
        );

        let mut batch_writer = VDescriptorWriteBatch::new();

        let material = Material3D {
            manager_index: allocated_sets_index,
        };

        material.queue_descriptor_writes(&self.materials_manager, texture, &mut batch_writer);
        batch_writer.flush(&self.v_backend.v_device);
        material
    }

    pub fn unload_texture(&mut self, texture: Id) {
        if let Some(tex) = self.textures.remove(&texture) {
            tex.destroy(&self.v_backend);
        }
    }

    pub fn run(&mut self, on_pre_render: &mut impl FnMut(&mut GameEngine, Duration)) {
        self.window.pwindow.set_key_polling(true);
        self.window
            .pwindow
            .set_cursor_mode(glfw::CursorMode::Normal);
        while !self.window.pwindow.should_close() {
            self.window.glfwi.poll_events();
            self.handle_window_events();
            self.pre_render(on_pre_render);
            self.render();
        }
    }

    fn handle_window_events(&mut self) {
        let window_messages: Vec<(f64, WindowEvent)> =
            glfw::flush_messages(&self.window.receiver).collect();
        for (_, event) in window_messages {
            match event {
                glfw::WindowEvent::Key(Key::Escape, _, Action::Press, _) => {
                    self.window.pwindow.set_should_close(true);
                }
                _ => {}
            }
            if let Some(scene) = &mut self.active_scene {
                scene.handle_window_event(&event);
            }
        }
    }

    fn pre_render(&mut self, on_pre_render: &mut impl FnMut(&mut GameEngine, Duration)) {
        let current_instant = Instant::now();
        let dt = current_instant.duration_since(self.last_frame_instant);
        self.last_frame_instant = current_instant;

        on_pre_render(self, dt);

        // Pre-render the scene
        if let Some(scene) = &mut self.active_scene {
            scene.pre_render(&self.v_backend, dt.as_secs_f32());
        }
    }

    fn render(&mut self) {
        // render
        let render_result = self.v_backend.render(|info| self.render_scene(&info));

        // check render result
        if let Some(event) = self
            .v_backend
            .check_render_issues(&self.window, render_result)
        {
            self.scene_render.handle_backend_event(&event);
            if let Some(scene) = &mut self.active_scene {
                scene.handle_backend_event(&event);
            }
        }
    }

    fn render_scene(&self, info: &VRenderInfo) {
        if let Some(scene) = &self.active_scene {
            let recordables: [&dyn SceneRenderRecordable; 1] = [scene];
            self.scene_render.render(
                &self.v_backend.v_device,
                &self.materials_manager,
                info,
                &recordables,
            );
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
        self.scene_render.destroy(&self.v_backend.v_device);
        self.materials_manager.destroy(&self.v_backend.v_device);
        self.v_backend.destroy();
    }
}
