use ash::vk;

use crate::vulkan_backend::device::VDevice;

pub trait Recordable {
    fn record(
        &self,
        v_device: &VDevice,
        command_buffer: vk::CommandBuffer,
        frame_index: usize,
        pipeline_layouts: &[vk::PipelineLayout],
    );
}

pub trait Drawable {
    fn draw(&self, v_device: &VDevice, command_buffer: vk::CommandBuffer);
}


