use ash::vk;

#[derive(Clone, Copy)]
pub struct CommandBufferState {
    pub buffer: vk::CommandBuffer,
    pub image_available_sem: vk::Semaphore,
    pub render_finished_sem: vk::Semaphore,
    pub in_flight_fen: vk::Fence,
}

impl CommandBufferState {
    pub fn destory(&self, device: &ash::Device) {
        unsafe {
            device.destroy_semaphore(self.image_available_sem, None);
            device.destroy_semaphore(self.render_finished_sem, None);
            device.destroy_fence(self.in_flight_fen, None);
        }
    }
}
