use ash::vk;
use std::time::Duration;

pub struct VRenderInfo {
    pub command_buffer: vk::CommandBuffer,
    pub image_index: usize,
    pub frame_index: usize,
    pub duration: Duration,
}
