use ash::vk;

use crate::vulkan_backend::device::VDevice;

pub struct RecordContext<'a> {
    pub v_device: &'a VDevice,
    pub cmd: vk::CommandBuffer,
    pub frame_index: usize,
    pub pipeline_layout: vk::PipelineLayout,
}


