use std::ffi::CString;

use ash::khr::surface;
use ash::vk::{self, PresentModeKHR, QueueFlags, SurfaceCapabilitiesKHR, SurfaceFormatKHR};

use crate::entities::Window;

#[derive(Debug)]
pub struct ComputeDevice {
    pub pdevice: vk::PhysicalDevice,
    name: CString,
    dtype: vk::PhysicalDeviceType,
    memory: u32,
    queue_flags: Vec<QueueFlags>,
    supported_extensions: Vec<CString>,
    surface_capabilities: SurfaceCapabilitiesKHR,
    surface_formats: Vec<SurfaceFormatKHR>,
    present_modes: Vec<PresentModeKHR>,
}

impl ComputeDevice {
    pub fn new(
        pdevice: vk::PhysicalDevice,
        name: CString,
        dtype: vk::PhysicalDeviceType,
        memory: u32,
        queue_flags: Vec<QueueFlags>,
        supported_extensions: Vec<CString>,
        surface_capabilities: SurfaceCapabilitiesKHR,
        surface_formats: Vec<SurfaceFormatKHR>,
        present_modes: Vec<PresentModeKHR>,
    ) -> Self {
        Self {
            pdevice,
            name,
            dtype,
            memory,
            queue_flags,
            supported_extensions,
            surface_capabilities,
            surface_formats,
            present_modes,
        }
    }

    pub fn select_device_and_queue(devices: Vec<ComputeDevice>) -> Option<(ComputeDevice, u32)> {
        for each in devices {
            if each.dtype != vk::PhysicalDeviceType::DISCRETE_GPU {
                continue;
            }
            if !each
                .supported_extensions
                .contains(&c"VK_KHR_swapchain".to_owned())
            {
                continue;
            }
            if each.surface_formats.is_empty() || each.present_modes.is_empty() {
                continue;
            }
            let count = each.queue_flags.len();
            let mut index = 0;
            loop {
                if index == count {
                    break;
                }
                if each.queue_flags[index].contains(vk::QueueFlags::GRAPHICS) {
                    return Some((each, index as u32));
                }
                index += 1;
            }
        }
        None
    }

    pub fn select_present_queue(
        &self,
        surface_loader: &surface::Instance,
        surface: vk::SurfaceKHR,
    ) -> Option<u32> {
        let count = self.queue_flags.len();
        let mut index = 0u32;
        loop {
            if index as usize == count {
                break;
            }
            let supports_surface = unsafe {
                surface_loader
                    .get_physical_device_surface_support(self.pdevice, index, surface)
                    .expect("failed to check if any device queue family supports surface")
            };
            if supports_surface {
                return Some(index);
            }
            index += 1;
        }
        None
    }

    pub fn select_surface_format(&self) -> SurfaceFormatKHR {
        for each in self.surface_formats.iter() {
            if each.format == vk::Format::B8G8R8A8_SRGB
                && each.color_space == vk::ColorSpaceKHR::SRGB_NONLINEAR
            {
                return *each;
            }
        }
        return self.surface_formats[0];
    }

    pub fn select_present_mode(&self) -> PresentModeKHR {
        for each in self.present_modes.iter() {
            if *each == PresentModeKHR::MAILBOX {
                return PresentModeKHR::MAILBOX;
            }
        }
        return PresentModeKHR::FIFO;
    }

    pub fn select_image_extent(&self, window: &Window) -> vk::Extent2D {
        let actual_extent = window.get_framebuffer_size();
        let width = actual_extent.0.clamp(
            self.surface_capabilities.min_image_extent.width,
            self.surface_capabilities.max_image_extent.width,
        );
        let height = actual_extent.1.clamp(
            self.surface_capabilities.min_image_extent.height,
            self.surface_capabilities.max_image_extent.height,
        );
        vk::Extent2D {
            width: width,
            height: height,
        }
    }

    pub fn select_swapchain_image_count(&self) -> u32 {
        let mut image_count = self.surface_capabilities.min_image_count;
        if self.surface_capabilities.max_image_count > image_count {
            image_count += 1;
        }
        image_count
    }

    pub fn get_current_transform(&self) -> vk::SurfaceTransformFlagsKHR {
        self.surface_capabilities.current_transform
    }
}

impl std::fmt::Display for ComputeDevice {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "{:?}: {:?}\nMemory: {:?} MB\n",
            self.dtype, self.name, self.memory,
        )
    }
}
