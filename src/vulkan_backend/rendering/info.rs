use ash::vk;
use crate::core::ecs::types::Id;

pub struct VRenderInfo {
    pub command_buffer: vk::CommandBuffer,
    pub image_id: Id,
    pub frame_index: usize,
}
