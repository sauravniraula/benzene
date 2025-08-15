use ash::vk::Extent2D;
use glfw::{Action, Key, WindowEvent};
use nalgebra::{Matrix4, Point3, Vector3};
use std::time::SystemTime;

use crate::{
    constants::MAX_FRAMES_IN_FLIGHT,
    vulkan_backend::{backend::VBackend, memory::VUniformBuffer, rendering::info::VRenderInfo},
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
    start_time: SystemTime,
}

impl GameEngine {
    pub fn new() -> Self {
        let window = Window::new(WindowConfig::default());
        let v_backend = VBackend::new(&window);

        Self {
            window,
            v_backend,
            start_time: SystemTime::now(),
        }
    }

    pub fn run(&mut self) {
        self.window.pwindow.set_key_polling(true);
        while !self.window.pwindow.should_close() {
            self.window.glfwi.poll_events();

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
            let render_result = self.v_backend.render(|info| self.render(info));
            self.v_backend
                .check_render_issues(&self.window, render_result);
        }
    }

    fn render(&self, info: VRenderInfo) {
        // self.update_uniform_buffer(info.frame_index, self.v_backend.v_swapchain.image_extent);
        // self.v_backend.basic_rendering_system.render(
        //     &self.v_backend.v_device,
        //     info.command_buffer,
        //     info.image_index,
        // );
    }

    pub fn destroy(&self) {
        // self.v_backend.v_device.wait_till_idle();
        // self.descriptor_pool.destroy(&self.v_backend.v_device);
        // self.model.destroy(&self.v_backend);
        // for each in self.uniform_buffers.iter() {
        //     each.destroy(&self.v_backend);
        // }
        // self.v_backend.destroy();
    }
}
