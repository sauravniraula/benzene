use ash::vk;

use crate::vulkan_backend::{
    descriptor::config::{VDescriptorBindingConfig, VDescriptorLayoutConfig},
    device::VDevice,
};

pub struct VDescriptorSetLayout {
    pub layout: vk::DescriptorSetLayout,
    pub config: VDescriptorLayoutConfig,
}

impl VDescriptorSetLayout {
    pub fn new(v_device: &VDevice, config: VDescriptorLayoutConfig) -> Self {
        let layout_bindings: Vec<vk::DescriptorSetLayoutBinding> = config
            .bindings
            .iter()
            .map(|b: &VDescriptorBindingConfig| {
                vk::DescriptorSetLayoutBinding::default()
                    .binding(b.binding)
                    .descriptor_count(b.count)
                    .descriptor_type(b.descriptor_type)
                    .stage_flags(b.shader_stage)
            })
            .collect();

        let layout_info = vk::DescriptorSetLayoutCreateInfo::default().bindings(&layout_bindings);

        let layout = unsafe {
            v_device
                .device
                .create_descriptor_set_layout(&layout_info, None)
                .expect("failed to create descriptor set layout")
        };

        Self { layout, config }
    }

    pub fn destroy(&self, v_device: &VDevice) {
        unsafe {
            v_device
                .device
                .destroy_descriptor_set_layout(self.layout, None);
        }
    }
}
