use crate::entities::CommandBufferState;
use ash::{Device, vk};

pub fn get_command_buffer_states(
    device: &Device,
    command_pool: vk::CommandPool,
    count: u32,
) -> Vec<CommandBufferState> {
    unsafe {
        let info = vk::CommandBufferAllocateInfo::default()
            .command_buffer_count(count)
            .command_pool(command_pool)
            .level(vk::CommandBufferLevel::PRIMARY);

        let mut command_buffers = device
            .allocate_command_buffers(&info)
            .expect("failed to allocate command buffers");

        let sem_info = vk::SemaphoreCreateInfo::default();
        let fen_info = vk::FenceCreateInfo::default().flags(vk::FenceCreateFlags::SIGNALED);

        let mut states: Vec<CommandBufferState> = vec![];
        loop {
            states.push(CommandBufferState {
                buffer: command_buffers.remove(0),
                image_available_sem: device
                    .create_semaphore(&sem_info, None)
                    .expect("failed to create semaphore"),
                render_finished_sem: device
                    .create_semaphore(&sem_info, None)
                    .expect("failed to create semaphore"),
                in_flight_fen: device
                    .create_fence(&fen_info, None)
                    .expect("failed to create fence"),
            });
            if command_buffers.len() == 0 {
                break;
            }
        }
        return states;
    }
}
