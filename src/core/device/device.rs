use std::ffi::CString;

use crate::core::{device::VPhysicalDevice, instance::VInstance};
use ash::Device;
use ash::vk;

pub struct VDevice {
    pub device: Device,
    pub graphics_queue_family_index: u32,
    pub present_queue_family_index: u32,
    pub unique_queue_family_indices: Vec<u32>,
    pub graphics_queue: vk::Queue,
    pub present_queue: vk::Queue,
}

impl VDevice {
    pub fn new(v_instance: &VInstance, v_physical_device: &VPhysicalDevice) -> Self {
        let graphics_queue_family_index =
            v_physical_device.get_queue_family(vk::QueueFlags::GRAPHICS);
        let present_queue_family_index = v_physical_device
            .get_present_queue_family_index()
            .expect("failed to find present queue");

        let mut unique_queue_family_indices = vec![graphics_queue_family_index];
        if graphics_queue_family_index != present_queue_family_index {
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
        let present_queue = unsafe { device.get_device_queue(present_queue_family_index, 0) };

        Self {
            device,
            graphics_queue_family_index,
            present_queue_family_index,
            unique_queue_family_indices,
            graphics_queue,
            present_queue,
        }
    }
}
