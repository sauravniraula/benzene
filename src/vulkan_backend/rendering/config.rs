use crate::vulkan_backend::pipeline::VPipelineInfo;
use ash::vk;

pub struct VRenderingSystemConfig<'a> {
    pub pipeline_infos: &'a Vec<VPipelineInfo>,
    pub color_format: Option<vk::Format>,
    pub depth_format: Option<vk::Format>,
    pub color_final_layout: Option<vk::ImageLayout>,
    pub depth_final_layout: Option<vk::ImageLayout>,
    pub dynamic_viewport: bool,
}
