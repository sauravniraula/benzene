use ash::vk;

use crate::vulkan_backend::device::VDevice;


pub struct RecordContext<'a> {
    pub v_device: &'a VDevice,
    pub cmd: vk::CommandBuffer,
    pub frame_index: usize,
    pub pipeline_layout: vk::PipelineLayout,
}

pub trait Recordable {
    fn record(&self, ctx: &RecordContext);
}

pub trait Drawable {
    fn draw(&self, v_device: &VDevice, command_buffer: vk::CommandBuffer);
}
