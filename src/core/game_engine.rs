use glfw::{Action, Key, WindowEvent};
use nalgebra::Matrix4;

use crate::{
    core::scene::Scene,
    vulkan_backend::{backend::VBackend, rendering::info::VRenderInfo},
    window::{Window, WindowConfig},
};

pub struct GlobalUniform {
    pub model: Matrix4<f32>,
    pub view: Matrix4<f32>,
    pub projection: Matrix4<f32>,
}

pub struct GameEngine {
    window: Window,
    v_backend: VBackend,
    active_scene: Option<Scene>,
}

impl GameEngine {
    pub fn new() -> Self {
        let window = Window::new(WindowConfig::default());
        let v_backend = VBackend::new(&window);

        let engine = Self {
            window,
            v_backend,
            active_scene: None,
        };
        engine
    }

    pub fn create_scene(&self) -> Scene {
        Scene::new(&self.v_backend)
    }

    pub fn set_active_scene(&mut self, scene: Scene) {
        self.active_scene = Some(scene);
    }

    pub fn run(&mut self) {
        self.window.pwindow.set_key_polling(true);
        while !self.window.pwindow.should_close() {
            self.window.glfwi.poll_events();
            self.handle_window_events();

            let render_result = self.v_backend.render(|info| self.render(&info));
            self.v_backend
                .check_render_issues(&self.window, render_result);
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
        }
    }

    fn render(&self, info: &VRenderInfo) {
        if let Some(scene) = &self.active_scene {
            scene.update(info.frame_index, self.v_backend.v_swapchain.image_extent);
            let recordables: [&dyn crate::vulkan_backend::rendering::Recordable; 1] = [scene];
            self.v_backend.basic_rendering_system.render(
                &self.v_backend.v_device,
                info,
                &recordables,
            );
        }
    }

    pub fn destroy(&self) {
        self.v_backend.v_device.wait_till_idle();
        if let Some(scene) = &self.active_scene {
            scene.destroy(&self.v_backend);
        }
        self.v_backend.destroy();
    }
}
