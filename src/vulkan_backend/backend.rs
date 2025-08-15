use crate::{
    constants::MAX_FRAMES_IN_FLIGHT,
    vulkan_backend::{
        backend_event::VBackendEvent,
        device::{VDevice, VPhysicalDevice, config::VPhysicalDeviceConfig},
        instance::{VInstance, VInstanceConfig},
        memory::VMemoryManager,
        rendering::{BasicRenderingSystem, VRenderResult, VRenderer, info::VRenderInfo},
        surface::VSurface,
        swapchain::VSwapchain,
    },
    window::Window,
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
    pub fn new(window: &Window) -> Self {
        let v_instance = VInstance::new(&window, VInstanceConfig::default());
        let v_surface = VSurface::new(&window, &v_instance);
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
            &window,
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

    pub fn recreate_swapchain(&mut self, window: &Window) {
        self.v_device.wait_till_idle();
        self.v_swapchain.destroy(&self.v_device);
        self.v_swapchain = VSwapchain::new(
            window,
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

    pub fn check_render_issues(&mut self, window: &Window, result: VRenderResult) {
        match result {
            VRenderResult::RecreateSwapchain => {
                self.recreate_swapchain(window);
                self.emit_update_framebuffers_event();
            }
            _ => {}
        }
    }

    pub fn render(&self, render: impl Fn(VRenderInfo) -> ()) -> VRenderResult {
        self.v_renderer
            .render(&self.v_device, &self.v_swapchain, render)
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
