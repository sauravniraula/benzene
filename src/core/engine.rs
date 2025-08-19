use glfw::{Action, Key, WindowEvent};
use std::time::Instant;

use crate::{
    core::{
        gpu::model::Model,
        rendering::{recordable::Recordable, scene_render::SceneRender},
        resources::primitives::ModelBuilder,
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

    pub fn build_model<B: ModelBuilder>(&self) -> Model {
        B::create_model(&self.v_backend)
    }

    pub fn run(&mut self) {
        self.window.pwindow.set_key_polling(true);
        self.window
            .pwindow
            .set_cursor_mode(glfw::CursorMode::Normal);
        while !self.window.pwindow.should_close() {
            let now = Instant::now();
            let dt = (now - self.last_frame_instant).as_secs_f32();
            self.last_frame_instant = now;

            self.window.glfwi.poll_events();
            self.handle_window_events();

            if let Some(scene) = &mut self.active_scene {
                let frame_index = self.v_backend.v_renderer.frame_index.get();
                scene.update(
                    frame_index,
                    self.v_backend.v_swapchain.get_image_extent(),
                    dt,
                );
            }

            let render_result = self.v_backend.render(|info| self.render(&info));
            if let Some(event) = self
                .v_backend
                .check_render_issues(&self.window, render_result)
            {
                self.scene_render.handle_backend_event(&event);
            }
        }
    }

    pub fn handle_window_events(&mut self) {
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

    fn render(&self, info: &VRenderInfo) {
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
