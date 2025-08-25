use ash::vk;

pub struct VDescriptorBindingConfig {
    pub binding: u32,
    pub count: u32,
    pub descriptor_type: vk::DescriptorType,
    pub shader_stage: vk::ShaderStageFlags,
}

pub struct VDescriptorLayoutConfig {
    pub bindings: Vec<VDescriptorBindingConfig>,
}


pub struct VDescriptorPoolTypeConfig {
    pub descriptor_type: vk::DescriptorType,
    pub count: u32,
}

pub struct VDescriptorPoolConfig {
    pub types: Vec<VDescriptorPoolTypeConfig>,
    pub max_sets: u32,
}