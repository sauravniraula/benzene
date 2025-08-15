use ash::vk;

use crate::vulkan_backend::device::VDevice;

pub struct VDescriptorLayout {
    pub layout: vk::DescriptorSetLayout,
    pub binding: u32,
    pub count: u32,
}

impl VDescriptorLayout {
    pub fn new(v_device: &VDevice) -> Self {
        let layout_bindings = [vk::DescriptorSetLayoutBinding::default()
            .binding(0)
            .descriptor_count(1)
            .descriptor_type(vk::DescriptorType::UNIFORM_BUFFER)
            .stage_flags(vk::ShaderStageFlags::VERTEX)];

        let layout_info = vk::DescriptorSetLayoutCreateInfo::default().bindings(&layout_bindings);

        let layout = unsafe {
            v_device
                .device
                .create_descriptor_set_layout(&layout_info, None)
                .expect("failed to create descriptor set layout")
        };

        Self {
            layout,
            binding: 0,
            count: 1,
        }
    }

    pub fn destroy(&self, v_device: &VDevice) {
        unsafe {
            v_device
                .device
                .destroy_descriptor_set_layout(self.layout, None);
        }
    }
}
