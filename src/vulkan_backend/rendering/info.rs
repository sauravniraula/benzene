use ash::vk;

pub struct VRenderInfo {
    pub command_buffer: vk::CommandBuffer,
    pub image_index: usize,
    pub frame_index: usize,
}
