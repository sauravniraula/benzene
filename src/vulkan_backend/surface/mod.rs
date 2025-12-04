use ash::khr;
use ash::vk;
use winit::raw_window_handle::HasDisplayHandle;
use winit::raw_window_handle::HasWindowHandle;
use winit::window::Window;

use crate::vulkan_backend::device::VPhysicalDevice;
use crate::vulkan_backend::instance::VInstance;

pub struct VSurface {
    pub surface: vk::SurfaceKHR,
    pub surface_instance: khr::surface::Instance,
}

impl VSurface {
    pub fn new(window: &Window, v_instance: &VInstance) -> Self {
        let surface = unsafe {
            ash_window::create_surface(
                &v_instance.entry,
                &v_instance.instance,
                window.display_handle().unwrap().as_raw(),
                window.window_handle().unwrap().as_raw(),
                None,
            )
            .expect("failed to create surface")
        };

        let surface_instance = khr::surface::Instance::new(&v_instance.entry, &v_instance.instance);

        Self {
            surface,
            surface_instance,
        }
    }

    pub fn get_surface_capabilities(
        &self,
        v_physical_device: &VPhysicalDevice,
    ) -> vk::SurfaceCapabilitiesKHR {
        unsafe {
            self.surface_instance
                .get_physical_device_surface_capabilities(
                    v_physical_device.physical_device,
                    self.surface,
                )
                .expect("failed to get surface capabilities")
        }
    }

    pub fn destroy(&self) {
        unsafe {
            self.surface_instance.destroy_surface(self.surface, None);
        }
    }
}
