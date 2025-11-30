use crate::shared::types::Id;
use ash::vk;

pub struct VFrameRenderContext {
    pub index: usize,
    pub cmd: vk::CommandBuffer,
    pub image_id: Id,
}
