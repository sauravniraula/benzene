use std::ptr::null;

use ash::vk;
use glfw::{Context, Glfw, GlfwReceiver, PWindow, WindowEvent, WindowHint, fail_on_errors};

use crate::entities::WindowInstruction;

pub struct Window {
    glfw: Glfw,
    window: PWindow,
    events: GlfwReceiver<(f64, WindowEvent)>,
}

impl Window {
    pub fn new(width: u32, height: u32, title: &str, mode: glfw::WindowMode) -> Self {
        let mut glfw = glfw::init(fail_on_errors!()).expect("Expected to initialize glfw");

        glfw.window_hint(WindowHint::ClientApi(glfw::ClientApiHint::NoApi));
        glfw.window_hint(WindowHint::Resizable(false));

        let (window, events) = glfw
            .create_window(width, height, title, mode)
            .expect("Expected to create window");

        Self {
            glfw,
            window,
            events,
        }
    }

    pub fn make_current(&mut self) {
        self.window.make_current();
    }

    pub fn set_key_polling(&mut self, should_poll: bool) {
        self.window.set_key_polling(should_poll);
    }

    pub fn get_required_glfw_extensions(&self) -> Option<Vec<String>> {
        self.glfw.get_required_instance_extensions()
    }

    pub unsafe fn get_surface(
        &self,
        instance: vk::Instance,
        allocation_callback: Option<vk::AllocationCallbacks>,
    ) -> Option<vk::SurfaceKHR> {
        let callback_pointer = match allocation_callback {
            Some(callback) => &callback,
            None => null(),
        };
        let surface_khr: *mut vk::SurfaceKHR = &mut vk::SurfaceKHR::null();
        // Uses behind the scene
        // vk::XcbSurfaceCreateInfoKHR
        // ash::khr::xcb_surface
        let result = self
            .window
            .create_window_surface(instance, callback_pointer, surface_khr);
        if result == vk::Result::SUCCESS {
            return Some(unsafe { *surface_khr });
        }
        None
    }

    pub fn get_framebuffer_size(&self) -> (u32, u32) {
        let size = self.window.get_framebuffer_size();
        (size.0 as u32, size.1 as u32)
    }

    pub fn start(
        &mut self,
        on_render_loop: impl Fn(),
        events_handler: impl Fn(WindowEvent) -> WindowInstruction,
    ) {
        while !self.window.should_close() {
            // self.window.swap_buffers();

            on_render_loop();

            self.glfw.poll_events();
            for (_, event) in glfw::flush_messages(&self.events) {
                match events_handler(event) {
                    WindowInstruction::None => (),
                    WindowInstruction::Close => self.window.set_should_close(true),
                }
            }
        }
    }
}
