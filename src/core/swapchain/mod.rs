use crate::core::{device::VPhysicalDevice, surface::VSurface, window::VWindow};
use ash::{khr, vk};

pub struct VSwapchain {
    pub swapchain_device: khr::swapchain::Device,
    pub swapchain: vk::SwapchainKHR,
    pub image_count: u32,
    pub image_extent: vk::Extent2D,
    pub format: vk::Format,
    pub images: Vec<vk::Image>,
    pub image_views: Vec<vk::ImageView>,
    pub subresource_range: vk::ImageSubresourceRange,
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
        let mut create_info = vk::SwapchainCreateInfoKHR::default()
            .surface(v_surface.surface)
            .image_usage(vk::ImageUsageFlags::COLOR_ATTACHMENT | vk::ImageUsageFlags::TRANSFER_DST)
            .pre_transform(v_physical_device.surface_capabilities.current_transform)
            .composite_alpha(vk::CompositeAlphaFlagsKHR::OPAQUE)
            .image_array_layers(1)
            .min_image_count(image_count)
            .image_extent(image_extent)
            .image_format(surface_format.format)
            .image_color_space(surface_format.color_space);

        let graphics_and_present_queue_family_indices = [
            v_device.graphics_queue_family_index,
            v_device.present_queue_family_index,
        ];
        if !v_device.is_graphics_and_present_queue_same {
            create_info = create_info
                .image_sharing_mode(vk::SharingMode::CONCURRENT)
                .queue_family_indices(&graphics_and_present_queue_family_indices);
        }

        let swapchain = unsafe {
            swapchain_device
                .create_swapchain(&create_info, None)
                .expect("failed to create swapchain")
        };

        let images = unsafe {
            swapchain_device
                .get_swapchain_images(swapchain)
                .expect("failed to get swapchain images")
        };

        let subresource_range = vk::ImageSubresourceRange::default()
            .aspect_mask(vk::ImageAspectFlags::COLOR)
            .level_count(1)
            .layer_count(1);

        let image_views: Vec<vk::ImageView> = images
            .iter()
            .map(|image| unsafe {
                let create_info = vk::ImageViewCreateInfo::default()
                    .image(*image)
                    .subresource_range(subresource_range)
                    .view_type(vk::ImageViewType::TYPE_2D)
                    .format(surface_format.format);

                v_device
                    .device
                    .create_image_view(&create_info, None)
                    .expect("failed to create image view")
            })
            .collect();

        Self {
            swapchain_device,
            swapchain,
            image_count,
            image_extent,
            format: surface_format.format,
            images,
            image_views,
            subresource_range,
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

    pub fn destroy(&self, v_device: &super::device::VDevice) {
        unsafe {
            for &image_view in self.image_views.iter() {
                v_device.device.destroy_image_view(image_view, None);
            }
            self.swapchain_device
                .destroy_swapchain(self.swapchain, None);
        }
    }
}
