use ash::vk;

pub struct VPhysicalDeviceConfig {
    pub required_extensions: Vec<String>,
    pub required_queue_flags: Vec<vk::QueueFlags>,
}

impl VPhysicalDeviceConfig {
    pub fn default() -> Self {
        Self {
            required_extensions: vec!["VK_KHR_swapchain".into()],
            required_queue_flags: vec![vk::QueueFlags::GRAPHICS],
        }
    }
}
