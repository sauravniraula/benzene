use ash::vk;
use glfw::{self, WindowEvent, fail_on_errors};
use std::{io::Result, ptr::null};

pub struct VWindow {
    pub glfwi: glfw::Glfw,
    pub window: glfw::PWindow,
    pub receiver: glfw::GlfwReceiver<(f64, WindowEvent)>,
}

impl VWindow {
    pub fn new(config: super::VWindowConfig) -> Result<Self> {
        let mut glfwi = glfw::init(fail_on_errors!()).expect("failed to initialize glfw");

        glfwi.window_hint(glfw::WindowHint::ClientApi(glfw::ClientApiHint::NoApi));
        glfwi.window_hint(glfw::WindowHint::Resizable(false));

        let (window, receiver) = glfwi
            .create_window(
                config.width,
                config.height,
                &config.title,
                glfw::WindowMode::Windowed,
            )
            .expect("failed to create window");

        Ok(Self {
            glfwi,
            window,
            receiver,
        })
    }

    pub fn get_surface(
        &self,
        instance: vk::Instance,
        allocation_callback: Option<vk::AllocationCallbacks>,
    ) -> Option<vk::SurfaceKHR> {
        let callback_pointer = match allocation_callback {
            Some(callback) => &callback,
            None => null(),
        };
        let surface_khr: *mut vk::SurfaceKHR = &mut vk::SurfaceKHR::null();
        let result = self
            .window
            .create_window_surface(instance, callback_pointer, surface_khr);
        if result == vk::Result::SUCCESS {
            return Some(unsafe { *surface_khr });
        }
        None
    }
}
