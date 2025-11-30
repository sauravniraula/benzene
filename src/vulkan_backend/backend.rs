use crate::{
    constants::MAX_FRAMES_IN_FLIGHT,
    vulkan_backend::{
        backend_event::VBackendEvent,
        device::{VDevice, VPhysicalDevice, config::VPhysicalDeviceConfig},
        frame::{VFrameRenderResult, VFrameRenderer, context::VFrameRenderContext},
        instance::{VInstance, VInstanceConfig},
        memory::VMemoryManager,
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
    pub v_frame_renderer: VFrameRenderer,
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
            &v_memory_manager,
        );
        let v_frame_renderer = VFrameRenderer::new(&v_device, MAX_FRAMES_IN_FLIGHT);

        Self {
            v_instance,
            v_surface,
            v_physical_device,
            v_device,
            v_memory_manager,
            v_swapchain,
            v_frame_renderer,
        }
    }

    pub fn recreate_swapchain(&mut self, window: &Window) {
        self.v_device.wait_till_idle();
        self.v_swapchain
            .destroy(&self.v_device, &self.v_memory_manager);
        self.v_swapchain = VSwapchain::new(
            window,
            &self.v_instance,
            &self.v_surface,
            &self.v_physical_device,
            &self.v_device,
            &self.v_memory_manager,
        );
    }

    pub fn check_render_issues<'a>(
        &'a mut self,
        window: &Window,
        result: VFrameRenderResult,
    ) -> Option<VBackendEvent<'a>> {
        match result {
            VFrameRenderResult::RecreateSwapchain => {
                self.recreate_swapchain(window);
                Some(VBackendEvent::UpdateFramebuffers(
                    &self.v_device,
                    &self.v_swapchain,
                ))
            }
            _ => None,
        }
    }

    pub fn render(&self, render: impl Fn(VFrameRenderContext) -> ()) -> VFrameRenderResult {
        self.v_frame_renderer
            .render(&self.v_device, &self.v_swapchain, render)
    }

    pub fn destroy(&self) {
        self.v_frame_renderer.destroy(&self.v_device);
        self.v_swapchain
            .destroy(&self.v_device, &self.v_memory_manager);
        self.v_memory_manager.destroy(&self.v_device);
        self.v_device.destroy();
        self.v_surface.destroy();
        self.v_instance.destroy();
    }
}
