use crate::core::pipeline::VPipelineInfo;

pub struct VRenderingSystemConfig<'a> {
    pub pipeline_infos: Vec<VPipelineInfo<'a>>,
}
