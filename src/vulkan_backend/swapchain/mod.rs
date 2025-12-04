use crate::shared::types::Id;
use crate::vulkan_backend::device::VDevice;
use crate::vulkan_backend::memory::VMemoryManager;
use crate::vulkan_backend::memory::image::config::VImageConfig;
use crate::vulkan_backend::memory::image::{VImage, image_view::VImageView};
use crate::vulkan_backend::{device::VPhysicalDevice, instance::VInstance, surface::VSurface};
use ash::{khr, vk};
use rand;
use winit::window::Window;

pub struct VSwapchain {
    pub swapchain_device: khr::swapchain::Device,
    pub swapchain: vk::SwapchainKHR,
    pub image_extent: vk::Extent2D,
    pub image_ids: Vec<Id>,
    pub v_images: Vec<VImage>,
    pub v_image_views: Vec<VImageView>,
    pub depth_v_image: VImage,
    pub depth_v_image_view: VImageView,
    pub depth_format: vk::Format,
}

impl VSwapchain {
    pub fn new(
        window: &Window,
        v_instance: &VInstance,
        v_surface: &VSurface,
        v_physical_device: &VPhysicalDevice,
        v_device: &VDevice,
        v_memory_manager: &VMemoryManager,
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
            .present_mode(v_physical_device.select_present_mode())
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

        let image_ids: Vec<Id> = (0..images.len()).map(|_| rand::random()).collect();

        let extent_3d = vk::Extent3D {
            width: image_extent.width,
            height: image_extent.height,
            depth: 1,
        };
        let sharing_mode = if v_device.is_graphics_and_present_queue_same {
            vk::SharingMode::EXCLUSIVE
        } else {
            vk::SharingMode::CONCURRENT
        };
        let queue_family_indices = if v_device.is_graphics_and_present_queue_same {
            None
        } else {
            Some(vec![
                v_device.graphics_queue_family_index,
                v_device.present_queue_family_index,
            ])
        };

        let v_images: Vec<VImage> = images
            .iter()
            .map(|&img| {
                VImage::from_external(
                    img,
                    VImageConfig::external_2d(
                        extent_3d,
                        vk::ImageUsageFlags::COLOR_ATTACHMENT | vk::ImageUsageFlags::TRANSFER_DST,
                        sharing_mode,
                        queue_family_indices.clone(),
                        surface_format.format,
                    ),
                )
            })
            .collect();

        let v_image_views: Vec<VImageView> = v_images
            .iter()
            .map(|v_image| {
                VImageView::new_2d(
                    v_device,
                    &v_image,
                    vk::ImageAspectFlags::COLOR,
                    surface_format.format,
                )
            })
            .collect();

        let depth_format = v_physical_device.get_format_for_depth_stencil(v_instance);

        let depth_v_image = VImage::new(
            v_device,
            v_physical_device,
            v_memory_manager,
            VImageConfig::image_2d(
                extent_3d,
                image_extent.height as u64 * image_extent.width as u64 * 4,
                vk::ImageUsageFlags::DEPTH_STENCIL_ATTACHMENT,
                sharing_mode,
                queue_family_indices,
                vk::MemoryPropertyFlags::DEVICE_LOCAL,
                depth_format,
            ),
        );
        let depth_v_image_view = VImageView::new_2d(
            v_device,
            &depth_v_image,
            vk::ImageAspectFlags::DEPTH,
            depth_format,
        );

        Self {
            swapchain_device,
            swapchain,
            image_extent,
            image_ids,
            v_images,
            v_image_views,
            depth_v_image,
            depth_v_image_view,
            depth_format,
        }
    }

    pub fn select_image_extent(
        window: &Window,
        surface_capabilities: &vk::SurfaceCapabilitiesKHR,
    ) -> vk::Extent2D {
        let window_size = window.inner_size();
        let width = (window_size.width as u32).clamp(
            surface_capabilities.min_image_extent.width,
            surface_capabilities.max_image_extent.width,
        );
        let height = (window_size.height as u32).clamp(
            surface_capabilities.min_image_extent.height,
            surface_capabilities.max_image_extent.height,
        );
        let extent = vk::Extent2D {
            width: width,
            height: height,
        };

        extent
    }

    pub fn select_swapchain_image_count(surface_capabilities: &vk::SurfaceCapabilitiesKHR) -> u32 {
        let mut image_count = surface_capabilities.min_image_count;
        if surface_capabilities.max_image_count > image_count {
            image_count += 1;
        }
        image_count
    }

    pub fn destroy(&self, v_device: &VDevice, v_memory_manager: &VMemoryManager) {
        unsafe {
            self.depth_v_image_view.destroy(v_device);
            self.depth_v_image.destroy(v_device, v_memory_manager);
            for v_image_view in self.v_image_views.iter() {
                v_image_view.destroy(v_device);
            }
            self.swapchain_device
                .destroy_swapchain(self.swapchain, None);
        }
    }
}
