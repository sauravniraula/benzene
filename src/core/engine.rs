use ash::vk;
use std::time::Instant;
use std::{collections::HashMap, time::Duration};
use winit::application::ApplicationHandler;
use winit::event::WindowEvent;
use winit::event_loop::EventLoop;
use winit::window::Window;

use crate::core::ecs::entities::game_object::GameObject;
use crate::core::gpu::scene_render::RecordableScene;
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
    window: Option<Window>,
    v_backend: Option<VBackend>,
    scene_renderer: Option<SceneRenderer>,
    materials_manager: Option<MaterialsManager>,

    // Resources
    textures: HashMap<Id, ImageTexture>,

    // State
    active_scene: Option<Scene>,
    last_frame_instant: Instant,

    // Debug
    frame_count: usize,
    fps: usize,
}

impl GameEngine {
    pub fn new() -> Self {
        let mut engine = Self {
            window: None,
            v_backend: None,
            scene_renderer: None,
            materials_manager: None,
            textures: HashMap::new(),
            active_scene: None,
            last_frame_instant: Instant::now(),
            frame_count: 0,
            fps: 0,
        };

        // Initialize Event Loop
        let event_loop = EventLoop::new().unwrap();
        let _ = event_loop.run_app(&mut engine);

        engine
    }

    pub fn init(&mut self) {
        let v_backend = self.v_backend.as_ref().unwrap();
        let materials_manager = self.materials_manager.as_mut().unwrap();
        let scene_renderer = self.scene_renderer.as_ref().unwrap();

        let default_texture = ImageTexture::empty(v_backend, vk::Format::R8G8B8A8_SRGB);
        let default_texture_id = get_random_id();
        self.textures.insert(default_texture_id, default_texture);
        let default_material_index = materials_manager.allocate_material(
            &v_backend.v_device,
            scene_renderer.get_image_sampler_layout(),
        );
        let default_material = Material3D {
            manager_index: default_material_index,
        };
        let mut batch_writer = VDescriptorWriteBatch::new();
        default_material.queue_descriptor_writes(
            materials_manager,
            &self.textures[&default_texture_id],
            &mut batch_writer,
        );
        batch_writer.flush(&v_backend.v_device);
    }

    pub fn create_scene(&self) -> Scene {
        Scene::new(
            self.v_backend.as_ref().unwrap(),
            self.scene_renderer.as_ref().unwrap(),
        )
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
            .add_spot_light(self.v_backend.as_ref().unwrap(), *entity.get_id());

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
            .remove_spot_light(self.v_backend.as_ref().unwrap(), entity.get_id());
        // self.scene_render
        //     .v_shadow_rendering_system
        //     .remove_framebuffer(&self.v_backend.v_device, entity.get_id());
    }

    pub fn get_structure_3d_from_obj(&self, obj_path: &str) -> Structure3D {
        Structure3D::from_obj(self.v_backend.as_ref().unwrap(), obj_path)
    }

    pub fn load_texture_from_image(&mut self, image_path: &str) -> Id {
        let texture = ImageTexture::new(
            self.v_backend.as_ref().unwrap(),
            image_path,
            vk::Format::R8G8B8A8_SRGB,
        );
        let id = get_random_id();
        self.textures.insert(id, texture);
        id
    }

    pub fn get_material_3d_from_texture(&mut self, texture: Id) -> Material3D {
        let texture = self
            .textures
            .get(&texture)
            .expect("invalid texture id passed to get_material_3d_from_texture");
        let allocated_sets_index = self.materials_manager.as_mut().unwrap().allocate_material(
            &self.v_backend.as_ref().unwrap().v_device,
            self.scene_renderer
                .as_ref()
                .unwrap()
                .get_image_sampler_layout(),
        );

        let mut batch_writer = VDescriptorWriteBatch::new();

        let material = Material3D {
            manager_index: allocated_sets_index,
        };

        material.queue_descriptor_writes(
            self.materials_manager.as_ref().unwrap(),
            texture,
            &mut batch_writer,
        );
        batch_writer.flush(&self.v_backend.as_ref().unwrap().v_device);
        material
    }

    pub fn unload_texture(&mut self, texture: Id) {
        if let Some(tex) = self.textures.remove(&texture) {
            tex.destroy(self.v_backend.as_ref().unwrap());
        }
    }

    fn handle_window_events(&mut self) {
        // let window_messages: Vec<(f64, WindowEvent)> =
        //     glfw::flush_messages(&self.window.receiver).collect();
        // for (_, event) in window_messages {
        //     match event {
        //         glfw::WindowEvent::Key(Key::Escape, _, Action::Press, _) => {
        //             self.window.pwindow.set_should_close(true);
        //         }
        //         _ => {}
        //     }
        //     if let Some(scene) = &mut self.active_scene {
        //         scene.handle_window_event(&event);
        //     }
        // }
    }

    fn pre_render(&mut self) {
        let current_instant = Instant::now();
        let dt = current_instant.duration_since(self.last_frame_instant);
        self.last_frame_instant = current_instant;

        // Frame Count and FPS
        self.fps = (1.0 / dt.as_secs_f64()) as usize;
        self.frame_count += 1;
        // println!("FPS: {}", self.fps);

        // Pre-render the scene
        if let Some(scene) = &mut self.active_scene {
            scene.pre_render(self.v_backend.as_ref().unwrap(), dt.as_secs_f32());
        }
    }

    fn render(&mut self) {
        // render
        let render_result = self
            .v_backend
            .as_ref()
            .unwrap()
            .render(|info| self.render_scene(&info));

        // check render result
        if let Some(event) = self
            .v_backend
            .as_mut()
            .unwrap()
            .check_render_issues(self.window.as_ref().unwrap(), render_result)
        {
            self.scene_renderer
                .as_mut()
                .unwrap()
                .handle_backend_event(&event);
            if let Some(scene) = &mut self.active_scene {
                scene.handle_backend_event(&event);
            }

            //* Debug */
            match event {
                VBackendEvent::UpdateFramebuffers(..) => {
                    println!("Update framebuffer, frame: {}", self.frame_count);
                }
                _ => (),
            }
        }
    }

    fn render_scene(&self, ctx: &VFrameRenderContext) {
        if let Some(scene) = &self.active_scene {
            let recordables: [&dyn RecordableScene; 1] = [scene];
            self.scene_renderer.as_ref().unwrap().render(
                &self.v_backend.as_ref().unwrap().v_device,
                self.materials_manager.as_ref().unwrap(),
                ctx,
                &recordables,
            );
        }
    }

    pub fn destroy(mut self) {
        self.v_backend.as_ref().unwrap().v_device.wait_till_idle();
        if let Some(scene) = &self.active_scene {
            scene.destroy(self.v_backend.as_ref().unwrap());
        }
        // Destroy all engine-owned textures
        for (_, tex) in self.textures.drain() {
            tex.destroy(self.v_backend.as_ref().unwrap());
        }
        self.scene_renderer
            .as_ref()
            .unwrap()
            .destroy(&self.v_backend.as_ref().unwrap().v_device);
        self.materials_manager
            .as_ref()
            .unwrap()
            .destroy(&self.v_backend.as_ref().unwrap().v_device);
        self.v_backend.as_ref().unwrap().destroy();
    }
}

impl ApplicationHandler for GameEngine {
    fn resumed(&mut self, event_loop: &winit::event_loop::ActiveEventLoop) {
        self.window = Some(
            event_loop
                .create_window(Window::default_attributes())
                .expect("failed to create window"),
        );

        self.v_backend = Some(VBackend::new(self.window.as_ref().unwrap()));
        self.scene_renderer = Some(SceneRenderer::new(self.v_backend.as_ref().unwrap()));
        self.materials_manager = Some(MaterialsManager::new(
            &self.v_backend.as_ref().unwrap().v_device,
        ));

        self.init();
    }

    fn window_event(
        &mut self,
        event_loop: &winit::event_loop::ActiveEventLoop,
        window_id: winit::window::WindowId,
        event: winit::event::WindowEvent,
    ) {
        match event {
            WindowEvent::RedrawRequested => {
                self.pre_render();
                self.render();

                self.window.as_ref().unwrap().request_redraw();
            }
            _ => (),
        }
    }
}
