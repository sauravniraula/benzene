use crate::core::pipeline::VPipelineInfo;
use ash::vk;

pub struct VRenderingSystemConfig {
    pub binding_descriptions: Vec<vk::VertexInputBindingDescription>,
    pub attribute_descriptions: Vec<vk::VertexInputAttributeDescription>,
    pub pipeline_infos: Vec<VPipelineInfo>,
}
