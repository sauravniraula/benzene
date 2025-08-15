use std::ffi::CString;

use crate::vulkan_backend::surface::VSurface;
use crate::vulkan_backend::{device::VPhysicalDevice, instance::VInstance};
use ash::Device;
use ash::vk;

pub struct VDevice {
    pub device: Device,
    pub graphics_queue_family_index: u32,
    pub transfer_queue_family_index: u32,
    pub present_queue_family_index: u32,
    pub unique_queue_family_indices: Vec<u32>,
    pub graphics_queue: vk::Queue,
    pub transfer_queue: vk::Queue,
    pub present_queue: vk::Queue,
    pub is_graphics_and_transfer_queue_same: bool,
    pub is_graphics_and_present_queue_same: bool,

    // For buffers
    pub buffer_sharing_mode: vk::SharingMode,
    pub buffer_queue_family_indices: Vec<u32>,
}

impl VDevice {
    pub fn new(
        v_instance: &VInstance,
        v_surface: &VSurface,
        v_physical_device: &VPhysicalDevice,
    ) -> Self {
        let graphics_queue_family_index = v_physical_device
            .get_queue_family_index(vk::QueueFlags::GRAPHICS)
            .expect("Selected device does not support graphics queue");

        let transfer_queue_family_index = v_physical_device
            .get_transfer_queue_family_index()
            .unwrap_or(graphics_queue_family_index);

        let present_queue_family_index = v_physical_device
            .get_present_queue_family_index(v_surface)
            .expect("failed to find present queue");

        let is_graphics_and_transfer_queue_same =
            graphics_queue_family_index == transfer_queue_family_index;
        let is_graphics_and_present_queue_same =
            graphics_queue_family_index == present_queue_family_index;

        let mut unique_queue_family_indices = vec![graphics_queue_family_index];
        let mut buffer_queue_family_indices = vec![graphics_queue_family_index];
        if !is_graphics_and_transfer_queue_same {
            unique_queue_family_indices.push(transfer_queue_family_index);
            buffer_queue_family_indices.push(transfer_queue_family_index);
        }
        if !is_graphics_and_present_queue_same {
            unique_queue_family_indices.push(present_queue_family_index);
        }
        let queue_infos: Vec<vk::DeviceQueueCreateInfo> = unique_queue_family_indices
            .iter()
            .map(|index| {
                vk::DeviceQueueCreateInfo::default()
                    .queue_family_index(*index)
                    .queue_priorities(&[1.0])
            })
            .collect();

        let p_extenstions: Vec<*const i8> = v_physical_device
            .required_extensions
            .iter()
            .map(|each| CString::new(each.as_str()).unwrap().into_raw() as *const i8)
            .collect();
        let device_info = vk::DeviceCreateInfo::default()
            .queue_create_infos(&queue_infos)
            .enabled_extension_names(&p_extenstions);

        let device = unsafe {
            v_instance
                .instance
                .create_device(v_physical_device.physical_device, &device_info, None)
                .expect("failed to create device")
        };

        let graphics_queue = unsafe { device.get_device_queue(graphics_queue_family_index, 0) };
        let transfer_queue = unsafe { device.get_device_queue(transfer_queue_family_index, 0) };
        let present_queue = unsafe { device.get_device_queue(present_queue_family_index, 0) };

        let buffer_sharing_mode = if buffer_queue_family_indices.len() > 0 {
            vk::SharingMode::CONCURRENT
        } else {
            vk::SharingMode::EXCLUSIVE
        };

        Self {
            device,
            graphics_queue_family_index,
            transfer_queue_family_index,
            present_queue_family_index,
            unique_queue_family_indices,
            graphics_queue,
            transfer_queue,
            present_queue,
            is_graphics_and_transfer_queue_same,
            is_graphics_and_present_queue_same,
            buffer_queue_family_indices,
            buffer_sharing_mode,
        }
    }

    pub fn wait_till_idle(&self) {
        unsafe {
            self.device
                .device_wait_idle()
                .expect("failed to wait device till idle");
        }
    }

    pub fn destroy(&self) {
        unsafe {
            self.device.destroy_device(None);
        }
    }
}
