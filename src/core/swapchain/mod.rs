use crate::core::{device::VPhysicalDevice, surface::VSurface, window::VWindow};
use ash::{khr, vk};

pub struct VSwapchain {
    pub swapchain_device: khr::swapchain::Device,
    pub swapchain: vk::SwapchainKHR,
    pub image_count: u32,
    pub image_extent: vk::Extent2D,
    pub format: vk::Format,
}

impl VSwapchain {
    pub fn new(
        v_window: &VWindow,
        v_instance: &super::instance::VInstance,
        v_surface: &VSurface,
        v_physical_device: &VPhysicalDevice,
        v_device: &super::device::VDevice,
    ) -> Self {
        let swapchain_device = khr::swapchain::Device::new(&v_instance.instance, &v_device.device);

        let image_count = v_physical_device.select_swapchain_image_count();
        let image_extent = VSwapchain::select_image_extent(&v_window, &v_physical_device);
        let surface_format = v_physical_device.select_surface_format();
        let create_info = vk::SwapchainCreateInfoKHR::default()
            .surface(v_surface.surface)
            .image_usage(vk::ImageUsageFlags::COLOR_ATTACHMENT)
            .pre_transform(v_physical_device.surface_capabilities.current_transform)
            .composite_alpha(vk::CompositeAlphaFlagsKHR::OPAQUE)
            .image_array_layers(1)
            .min_image_count(image_count)
            .image_extent(image_extent)
            .image_format(surface_format.format)
            .image_color_space(surface_format.color_space);

        let swapchain = unsafe {
            swapchain_device
                .create_swapchain(&create_info, None)
                .expect("failed to create swapchain")
        };

        Self {
            swapchain_device,
            swapchain,
            image_count,
            image_extent,
            format: surface_format.format,
        }
    }

    pub fn select_image_extent(
        v_window: &VWindow,
        v_physical_device: &VPhysicalDevice,
    ) -> vk::Extent2D {
        let actual_extent = v_window.window.get_framebuffer_size();
        let width = (actual_extent.0 as u32).clamp(
            v_physical_device
                .surface_capabilities
                .min_image_extent
                .width,
            v_physical_device
                .surface_capabilities
                .max_image_extent
                .width,
        );
        let height = (actual_extent.1 as u32).clamp(
            v_physical_device
                .surface_capabilities
                .min_image_extent
                .height,
            v_physical_device
                .surface_capabilities
                .max_image_extent
                .height,
        );
        vk::Extent2D {
            width: width,
            height: height,
        }
    }
}
