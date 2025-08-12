use ash::vk;

use crate::core::descriptor::VDescriptorLayout;

pub struct VPipelineInfoConfig<'a> {
    pub binding_descriptions: Vec<vk::VertexInputBindingDescription>,
    pub attribute_descriptions: Vec<vk::VertexInputAttributeDescription>,
    pub vertex_shader_file: String,
    pub fragment_shader_file: String,
    pub descriptor_layouts: &'a Vec<VDescriptorLayout>,
}
