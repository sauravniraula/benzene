pub fn create_command_pool(device: &ash::Device, queue_family_index: u32) -> ash::vk::CommandPool {
    let info = ash::vk::CommandPoolCreateInfo::default()
        .flags(ash::vk::CommandPoolCreateFlags::RESET_COMMAND_BUFFER)
        .queue_family_index(queue_family_index);

    unsafe {
        device
            .create_command_pool(&info, None)
            .expect("unable to create command pool")
    }
}

pub fn create_command_buffers(
    device: &ash::Device,
    command_pool: ash::vk::CommandPool,
    count: u32,
) -> Vec<ash::vk::CommandBuffer> {
    unsafe {
        device
            .allocate_command_buffers(
                &ash::vk::CommandBufferAllocateInfo::default()
                    .command_pool(command_pool)
                    .command_buffer_count(count)
                    .level(ash::vk::CommandBufferLevel::PRIMARY),
            )
            .expect("unable to create command buffer")
    }
}
