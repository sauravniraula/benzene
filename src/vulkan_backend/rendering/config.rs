use ash::vk;
use crate::vulkan_backend::pipeline::VPipelineInfo;

pub struct VRenderingSystemConfig<'a> {
    pub pipeline_infos: &'a Vec<VPipelineInfo>,
    // Formats corresponding to the provided attachments (required when provided)
    pub color_format: Option<vk::Format>,
    pub depth_format: Option<vk::Format>,
    // Final layouts for attachments
    pub color_final_layout: Option<vk::ImageLayout>,
    pub depth_final_layout: Option<vk::ImageLayout>,
}
