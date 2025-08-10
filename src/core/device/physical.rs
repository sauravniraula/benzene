use ash::vk;

use crate::core::{device::config::VPhysicalDeviceConfig, instance::VInstance, surface::VSurface};

pub struct VPhysicalDevice {
    pub physical_device: vk::PhysicalDevice,
    pub surface_formats: Vec<vk::SurfaceFormatKHR>,
    pub present_modes: Vec<vk::PresentModeKHR>,
    pub queue_families: Vec<vk::QueueFamilyProperties>,
    pub surface_capabilities: vk::SurfaceCapabilitiesKHR,
    pub memory_properties: vk::PhysicalDeviceMemoryProperties,
    pub score: usize,
    pub required_extensions: Vec<String>,
}

impl VPhysicalDevice {
    pub fn get_queue_family_index(&self, flag: vk::QueueFlags) -> Option<u32> {
        for i in 0..self.queue_families.len() {
            if self.queue_families[i].queue_flags.contains(flag) {
                return Some(i as u32);
            }
        }
        None
    }

    pub fn get_transfer_queue_family_index(&self) -> Option<u32> {
        for i in 0..self.queue_families.len() {
            if self.queue_families[i]
                .queue_flags
                .contains(vk::QueueFlags::TRANSFER)
                && !self.queue_families[i]
                    .queue_flags
                    .contains(vk::QueueFlags::GRAPHICS)
            {
                return Some(i as u32);
            }
        }
        None
    }

    pub fn get_present_queue_family_index(&self, v_surface: &VSurface) -> Option<u32> {
        for i in 0..self.queue_families.len() {
            let supports_surface = unsafe {
                v_surface
                    .surface_instance
                    .get_physical_device_surface_support(
                        self.physical_device,
                        i as u32,
                        v_surface.surface,
                    )
                    .expect("failed to get device surface support")
            };
            if supports_surface {
                return Some(i as u32);
            }
        }
        None
    }

    pub fn select_surface_format(&self) -> vk::SurfaceFormatKHR {
        for each in self.surface_formats.iter() {
            if each.format == vk::Format::B8G8R8A8_SRGB
                && each.color_space == vk::ColorSpaceKHR::SRGB_NONLINEAR
            {
                return *each;
            }
        }
        return self.surface_formats[0];
    }

    pub fn select_present_mode(&self) -> vk::PresentModeKHR {
        for each in self.present_modes.iter() {
            if *each == vk::PresentModeKHR::MAILBOX {
                return vk::PresentModeKHR::MAILBOX;
            }
        }
        return vk::PresentModeKHR::FIFO;
    }

    pub fn select_swapchain_image_count(&self) -> u32 {
        let mut image_count = self.surface_capabilities.min_image_count;
        if self.surface_capabilities.max_image_count > image_count {
            image_count += 1;
        }
        image_count
    }

    pub fn find_memory_type_index(
        &self,
        memory_type: u32,
        properties: vk::MemoryPropertyFlags,
    ) -> Option<u32> {
        for i in 0..self.memory_properties.memory_type_count {
            if memory_type & (1 << i) != 0
                && self.memory_properties.memory_types[i as usize]
                    .property_flags
                    .contains(properties)
            {
                return Some(i);
            }
        }
        None
    }

    pub fn get_compatible_devices(
        v_instance: &VInstance,
        v_surface: &VSurface,
        config: VPhysicalDeviceConfig,
    ) -> Vec<VPhysicalDevice> {
        let physical_devices = unsafe {
            v_instance
                .instance
                .enumerate_physical_devices()
                .expect("failed to list physical devices")
        };

        let mut compatible_physical_devices: Vec<VPhysicalDevice> = vec![];
        for each_device in physical_devices {
            let ext_properties = unsafe {
                v_instance
                    .instance
                    .enumerate_device_extension_properties(each_device)
                    .expect("failed to get device extension properties")
            };
            let supported_device_extensions: Vec<String> = ext_properties
                .iter()
                .map(|each| {
                    each.extension_name_as_c_str()
                        .unwrap()
                        .to_str()
                        .unwrap()
                        .into()
                })
                .collect();
            for each_required_extension in &config.required_extensions {
                if !supported_device_extensions.contains(each_required_extension) {
                    break;
                }
            }

            let surface_formats = unsafe {
                v_surface
                    .surface_instance
                    .get_physical_device_surface_formats(each_device, v_surface.surface)
                    .expect("failed to get device surface formats")
            };
            if surface_formats.is_empty() {
                break;
            }
            let present_modes = unsafe {
                v_surface
                    .surface_instance
                    .get_physical_device_surface_present_modes(each_device, v_surface.surface)
                    .expect("failed to get device present modesl")
            };
            if present_modes.is_empty() {
                break;
            }
            let queue_families = unsafe {
                v_instance
                    .instance
                    .get_physical_device_queue_family_properties(each_device)
            };

            let mut flags_count = 0;
            for each_flag in config.required_queue_flags.iter() {
                for each_family in queue_families.iter() {
                    if each_family.queue_flags.contains(*each_flag) {
                        flags_count += 1;
                        break;
                    }
                }
            }
            if flags_count != config.required_queue_flags.len() {
                break;
            }

            let surface_capabilities = unsafe {
                v_surface
                    .surface_instance
                    .get_physical_device_surface_capabilities(each_device, v_surface.surface)
                    .expect("failed to get device surface capabilities")
            };

            let memory_properties = unsafe {
                v_instance
                    .instance
                    .get_physical_device_memory_properties(each_device)
            };

            let score = surface_formats.len() + present_modes.len() + queue_families.len();
            compatible_physical_devices.push(VPhysicalDevice {
                physical_device: each_device,
                surface_formats,
                present_modes,
                queue_families,
                surface_capabilities,
                memory_properties,
                score,
                required_extensions: config.required_extensions.clone(),
            });
        }

        if compatible_physical_devices.is_empty() {
            panic!("failed to find device with required queue flags")
        }

        compatible_physical_devices.sort_by(|a, b| b.score.cmp(&a.score));
        compatible_physical_devices
    }
}
