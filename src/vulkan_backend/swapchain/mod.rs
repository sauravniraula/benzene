use crate::vulkan_backend::memory::image::config::VImageConfig;
use crate::vulkan_backend::memory::image::{VImage, image_view::VImageView};
use crate::{
    vulkan_backend::{device::VPhysicalDevice, instance::VInstance, surface::VSurface},
    window::Window,
};
use ash::{
    khr,
    vk::{self, Extent2D},
};

pub struct VSwapchain {
    pub swapchain_device: khr::swapchain::Device,
    pub swapchain: vk::SwapchainKHR,
    pub images: Vec<VImage>,
    pub image_views: Vec<VImageView>,
}

impl VSwapchain {
    pub fn new(
        window: &Window,
        v_instance: &VInstance,
        v_surface: &VSurface,
        v_physical_device: &VPhysicalDevice,
        v_device: &super::device::VDevice,
    ) -> Self {
        let swapchain_device = khr::swapchain::Device::new(&v_instance.instance, &v_device.device);

        let surface_capabilities = v_surface.get_surface_capabilities(v_physical_device);

        let image_count = VSwapchain::select_swapchain_image_count(&surface_capabilities);
        let image_extent = VSwapchain::select_image_extent(&window, &surface_capabilities);
        let surface_format = v_physical_device.select_surface_format();
        let mut create_info = vk::SwapchainCreateInfoKHR::default()
            .surface(v_surface.surface)
            .image_usage(vk::ImageUsageFlags::COLOR_ATTACHMENT | vk::ImageUsageFlags::TRANSFER_DST)
            .pre_transform(surface_capabilities.current_transform)
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

        let v_images: Vec<VImage> = images
            .iter()
            .map(|&img| {
                VImage::from_external(
                    img,
                    VImageConfig::external_color_2d(
                        vk::Extent3D {
                            width: image_extent.width,
                            height: image_extent.height,
                            depth: 1,
                        },
                        vk::ImageUsageFlags::COLOR_ATTACHMENT | vk::ImageUsageFlags::TRANSFER_DST,
                        if v_device.is_graphics_and_present_queue_same {
                            vk::SharingMode::EXCLUSIVE
                        } else {
                            vk::SharingMode::CONCURRENT
                        },
                        if v_device.is_graphics_and_present_queue_same {
                            None
                        } else {
                            Some(vec![
                                v_device.graphics_queue_family_index,
                                v_device.present_queue_family_index,
                            ])
                        },
                        surface_format.format,
                    ),
                )
            })
            .collect();

        let image_views: Vec<VImageView> = v_images
            .iter()
            .map(|v_image| VImageView::new_2d_color(v_device, v_image.image, surface_format.format))
            .collect();

        Self {
            swapchain_device,
            swapchain,
            images: v_images,
            image_views,
        }
    }

    pub fn get_image_extent(&self) -> Extent2D {
        return Extent2D {
            width: self.images[0].config.extent.width,
            height: self.images[0].config.extent.height,
        };
    }

    pub fn select_image_extent(
        window: &Window,
        surface_capabilities: &vk::SurfaceCapabilitiesKHR,
    ) -> vk::Extent2D {
        let actual_extent = window.pwindow.get_framebuffer_size();
        let width = (actual_extent.0 as u32).clamp(
            surface_capabilities.min_image_extent.width,
            surface_capabilities.max_image_extent.width,
        );
        let height = (actual_extent.1 as u32).clamp(
            surface_capabilities.min_image_extent.height,
            surface_capabilities.max_image_extent.height,
        );
        vk::Extent2D {
            width: width,
            height: height,
        }
    }

    pub fn select_swapchain_image_count(surface_capabilities: &vk::SurfaceCapabilitiesKHR) -> u32 {
        let mut image_count = surface_capabilities.min_image_count;
        if surface_capabilities.max_image_count > image_count {
            image_count += 1;
        }
        image_count
    }

    pub fn destroy(&self, v_device: &super::device::VDevice) {
        unsafe {
            for v_image_view in self.image_views.iter() {
                v_image_view.destroy(v_device);
            }
            self.swapchain_device
                .destroy_swapchain(self.swapchain, None);
        }
    }
}
