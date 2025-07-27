use vulkan_engine::core::{
    device::{VDevice, VPhysicalDevice, config::VPhysicalDeviceConfig},
    instance::{VInstance, VInstanceConfig},
    surface::VSurface,
    swapchain::VSwapchain,
    window::{VWindow, VWindowConfig},
};

fn main() {
    let v_window = VWindow::new(VWindowConfig::default()).expect("failed to create Vwindow");

    let v_instance =
        VInstance::new(&v_window, VInstanceConfig::default()).expect("failed to create VInstance");

    let v_surface = VSurface::new(&v_window, &v_instance);

    let mut v_physical_devices = VPhysicalDevice::get_compatible_devices(
        &v_instance,
        &v_surface,
        VPhysicalDeviceConfig::default(),
    );
    let v_physical_device = v_physical_devices.remove(0);

    let v_device = VDevice::new(&v_instance, &v_physical_device);

    let v_swapchain = VSwapchain::new(
        &v_window,
        &v_instance,
        &v_surface,
        &v_physical_device,
        &v_device,
    );
}
