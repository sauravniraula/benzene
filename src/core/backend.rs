use crate::{
    app::renderable_app::RenderableApp,
    constants::MAX_FRAMES_IN_FLIGHT,
    core::{
        backend_event::VBackendEvent,
        device::{VDevice, VPhysicalDevice, config::VPhysicalDeviceConfig},
        instance::{VInstance, VInstanceConfig},
        memory::VMemoryManager,
        rendering::{BasicRenderingSystem, VRenderResult, VRenderer},
        surface::VSurface,
        swapchain::VSwapchain,
        window::VWindow,
    },
};

pub struct VBackend {
    pub v_instance: VInstance,
    pub v_surface: VSurface,
    pub v_physical_device: VPhysicalDevice,
    pub v_device: VDevice,
    pub v_memory_manager: VMemoryManager,
    pub v_swapchain: VSwapchain,
    pub v_renderer: VRenderer,
    pub basic_rendering_system: BasicRenderingSystem,
}

impl VBackend {
    pub fn new(v_window: &VWindow) -> Self {
        let v_instance = VInstance::new(&v_window, VInstanceConfig::default());
        let v_surface = VSurface::new(&v_window, &v_instance);
        let mut v_physical_devices = VPhysicalDevice::get_compatible_devices(
            &v_instance,
            &v_surface,
            VPhysicalDeviceConfig::default(),
        );
        let v_physical_device = v_physical_devices.remove(0);

        println!(
            "Selected device: {:?}",
            v_physical_device.properties.device_name_as_c_str()
        );

        let v_device = VDevice::new(&v_instance, &v_surface, &v_physical_device);
        let v_memory_manager: VMemoryManager = VMemoryManager::new(&v_device);
        let v_swapchain = VSwapchain::new(
            &v_window,
            &v_instance,
            &v_surface,
            &v_physical_device,
            &v_device,
        );
        let v_renderer = VRenderer::new(&v_device, MAX_FRAMES_IN_FLIGHT);
        let basic_rendering_system = BasicRenderingSystem::new(&v_device, &v_swapchain);

        Self {
            v_instance,
            v_surface,
            v_physical_device,
            v_device,
            v_memory_manager,
            v_swapchain,
            v_renderer,
            basic_rendering_system,
        }
    }

    pub fn recreate_swapchain(&mut self, v_window: &VWindow) {
        self.v_device.wait_till_idle();
        self.v_swapchain.destroy(&self.v_device);
        self.v_swapchain = VSwapchain::new(
            v_window,
            &self.v_instance,
            &self.v_surface,
            &self.v_physical_device,
            &self.v_device,
        );
    }

    pub fn emit_update_framebuffers_event(&mut self) {
        let event = VBackendEvent::UpdateFramebuffers(&self.v_device, &self.v_swapchain);
        self.basic_rendering_system.handle_backend_event(&event);
    }

    pub fn render(&mut self, v_window: &VWindow, mut apps: Vec<&mut impl RenderableApp>) {
        let render_result = self.v_renderer.render(
            &self.v_device,
            &self.v_swapchain,
            |command_buffer, image_index, frame_index, duration| {
                for app in apps.iter_mut() {
                    app.render_app(self, command_buffer, image_index, frame_index, duration);
                }
            },
        );
        match render_result {
            VRenderResult::RecreateSwapchain => {
                self.recreate_swapchain(v_window);
                self.emit_update_framebuffers_event();
            }
            _ => {}
        }
    }

    pub fn destroy(&self) {
        self.v_renderer.destroy(&self.v_device);
        self.basic_rendering_system.destroy(&self.v_device);
        self.v_swapchain.destroy(&self.v_device);
        self.v_memory_manager.destroy(&self.v_device);
        self.v_device.destroy();
        self.v_surface.destroy();
        self.v_instance.destroy();
    }
}
