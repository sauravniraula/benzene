use ash::vk;

pub struct VPipelineInfoConfig {
    pub binding_descriptions: Vec<vk::VertexInputBindingDescription>,
    pub attribute_descriptions: Vec<vk::VertexInputAttributeDescription>,
    pub vertex_shader_file: Option<String>,
    pub fragment_shader_file: Option<String>,
}
