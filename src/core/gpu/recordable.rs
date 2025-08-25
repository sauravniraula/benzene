use ash::vk;

use crate::vulkan_backend::{
    descriptor::VDescriptorSetLayout,
    device::VDevice,
    pipeline::VPipelineInfo,
};

pub struct RecordContext<'a> {
    pub v_device: &'a VDevice,
    pub cmd: vk::CommandBuffer,
    pub frame_index: usize,
    pub pipeline_infos: &'a Vec<VPipelineInfo>,
    pub descriptor_sets_layouts: &'a Vec<VDescriptorSetLayout>,
}

pub trait Recordable {
    fn record(&self, ctx: &RecordContext);
}

pub trait Drawable {
    fn draw(&self, v_device: &VDevice, command_buffer: vk::CommandBuffer);
}
