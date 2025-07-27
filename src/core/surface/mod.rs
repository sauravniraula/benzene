use ash::khr;
use ash::vk;

use crate::core::instance::VInstance;
use crate::core::window::VWindow;

pub struct VSurface {
    pub surface: vk::SurfaceKHR,
    pub surface_instance: khr::surface::Instance,
}

impl VSurface {
    pub fn new(v_window: &VWindow, v_instance: &VInstance) -> Self {
        let surface = v_window
            .get_surface(v_instance.instance.handle(), None)
            .expect("failed to get surface");

        let surface_instance = khr::surface::Instance::new(&v_instance.entry, &v_instance.instance);

        Self {
            surface,
            surface_instance,
        }
    }
}
