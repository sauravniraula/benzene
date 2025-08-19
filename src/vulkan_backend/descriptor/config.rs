use ash::vk;

#[derive(Clone)]
pub struct VDescriptorBindingConfig {
    pub binding: u32,
    pub count: u32,
    pub descriptor_type: vk::DescriptorType,
    pub shader_stage: vk::ShaderStageFlags,
}

#[derive(Clone)]
pub struct VDescriptorLayoutConfig {
    pub bindings: Vec<VDescriptorBindingConfig>,
}

pub struct VDescriptorPoolSetConfig {
    pub layout: VDescriptorLayoutConfig,
    pub count: usize,
}

pub struct VDescriptorPoolConfig {
    pub sets: Vec<VDescriptorPoolSetConfig>,
}
