use glfw::{Action, Key, WindowEvent};
use std::time::Instant;

use crate::{
    core::{
        ModelBuilder,
        ecs::components::Structure3D,
        gpu::{recordable::Recordable, scene_render::SceneRender},
        scene::Scene,
    },
    vulkan_backend::{backend::VBackend, rendering::info::VRenderInfo},
    window::{Window, WindowConfig},
};

pub struct GameEngine {
    window: Window,
    v_backend: VBackend,
    scene_render: SceneRender,
    active_scene: Option<Scene>,
    last_frame_instant: Instant,
}

impl GameEngine {
    pub fn new() -> Self {
        let window = Window::new(WindowConfig::default());
        let v_backend = VBackend::new(&window);
        let scene_render = SceneRender::new(&v_backend);

        let engine = Self {
            window,
            v_backend,
            scene_render,
            active_scene: None,
            last_frame_instant: Instant::now(),
        };
        engine
    }

    pub fn create_scene(&self) -> Scene {
        Scene::new(&self.v_backend, &self.scene_render)
    }

    pub fn set_active_scene(&mut self, scene: Scene) {
        self.active_scene = Some(scene);
    }

    pub fn get_structure_from_model_builder<B: ModelBuilder>(&self) -> Structure3D {
        Structure3D::new(B::create_model(&self.v_backend))
    }

    pub fn get_structure_from_obj(&self, obj_path: &str) -> Structure3D {
        Structure3D::from_obj(&self.v_backend, obj_path)
    }

    pub fn run(&mut self) {
        self.window.pwindow.set_key_polling(true);
        self.window
            .pwindow
            .set_cursor_mode(glfw::CursorMode::Normal);
        while !self.window.pwindow.should_close() {
            self.window.glfwi.poll_events();
            self.handle_window_events();
            self.pre_render();
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

    fn pre_render(&mut self) {
        let current_instant = Instant::now();
        let dt = current_instant.duration_since(self.last_frame_instant);
        self.last_frame_instant = current_instant;

        // Pre-render the scene
        if let Some(scene) = &mut self.active_scene {
            scene.pre_render(dt.as_secs_f32());
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
            let recordables: [&dyn Recordable; 1] = [scene];
            self.scene_render
                .render(&self.v_backend.v_device, info, &recordables);
        }
    }

    pub fn destroy(&self) {
        self.v_backend.v_device.wait_till_idle();
        if let Some(scene) = &self.active_scene {
            scene.destroy(&self.v_backend);
        }
        self.scene_render.destroy(&self.v_backend.v_device);
        self.v_backend.destroy();
    }
}
