use crate::core::{
    device::{VDevice, VPhysicalDevice, config::VPhysicalDeviceConfig},
    instance::{VInstance, VInstanceConfig},
    memory::VMemoryManager,
    rendering::{BasicRenderingSystem, VRenderer},
    surface::VSurface,
    swapchain::VSwapchain,
    vertex_input::Vertex,
    window::VWindow,
};

pub struct VBackend {
    pub v_instance: VInstance,
    pub v_surface: VSurface,
    pub v_physical_device: VPhysicalDevice,
    pub v_device: VDevice,
    pub v_memory_manager: VMemoryManager,
    pub v_swapchain: VSwapchain,
    pub v_renderer: VRenderer,
    pub basic_rendering_system: BasicRenderingSystem<Vertex>,
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
        let v_device = VDevice::new(&v_instance, &v_surface, &v_physical_device);
        let v_memory_manager: VMemoryManager = VMemoryManager::new(&v_device);
        let v_swapchain = VSwapchain::new(
            &v_window,
            &v_instance,
            &v_surface,
            &v_physical_device,
            &v_device,
        );
        let v_renderer = VRenderer::new(&v_device, &v_swapchain);
        let basic_rendering_system = BasicRenderingSystem::<Vertex>::new(&v_device, &v_swapchain);

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
