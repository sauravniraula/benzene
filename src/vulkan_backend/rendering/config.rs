use crate::vulkan_backend::pipeline::VPipelineInfo;

pub struct VRenderingSystemConfig<'a> {
    pub pipeline_infos: &'a Vec<VPipelineInfo>,
}
