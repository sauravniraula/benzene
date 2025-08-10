use ash::vk;

use crate::core::{
    device::{VDevice, VPhysicalDevice},
    memory::VBuffer,
};

pub struct VMemoryManager {
    command_pool: vk::CommandPool,
}

impl VMemoryManager {
    pub fn new(v_device: &VDevice) -> Self {
        let command_pool = unsafe {
            v_device
                .device
                .create_command_pool(
                    &vk::CommandPoolCreateInfo::default()
                        .queue_family_index(v_device.transfer_queue_family_index),
                    None,
                )
                .expect("failed to create command pool for VMemoryManager")
        };

        Self { command_pool }
    }

    pub fn allocate_memory(
        &self,
        v_physical_device: &VPhysicalDevice,
        v_device: &VDevice,
        config: super::VAllocateMemoryConfig,
    ) -> vk::DeviceMemory {
        let memory_type_index = v_physical_device
            .find_memory_type_index(config.memory_type, config.properties)
            .expect("failed to find memory type index");
        let alloc_info = vk::MemoryAllocateInfo::default()
            .allocation_size(config.size)
            .memory_type_index(memory_type_index);

        unsafe {
            v_device
                .device
                .allocate_memory(&alloc_info, None)
                .expect("failed to allocate memory")
        }
    }

    pub fn copy_to_host_visible(
        &self,
        v_device: &VDevice,
        v_buffer: &VBuffer,
        data: *const u8,
        size: usize,
    ) {
        let destination = unsafe {
            v_device
                .device
                .map_memory(
                    v_buffer.memory,
                    0,
                    v_buffer.memory_requirements.size,
                    vk::MemoryMapFlags::empty(),
                )
                .expect("failed to map memory") as *mut u8
        };
        unsafe {
            std::ptr::copy_nonoverlapping(data, destination, size);
            v_device.device.unmap_memory(v_buffer.memory);
        };
    }

    pub fn copy_to_device_local(
        &self,
        v_device: &VDevice,
        from: &VBuffer,
        to: &VBuffer,
        size: usize,
    ) {
        let alloc_info = vk::CommandBufferAllocateInfo::default()
            .command_pool(self.command_pool)
            .command_buffer_count(1);
        unsafe {
            let command_buffers = v_device
                .device
                .allocate_command_buffers(&alloc_info)
                .expect("failed to allocate command buffers on VMemoryManager");

            v_device
                .device
                .begin_command_buffer(
                    command_buffers[0],
                    &vk::CommandBufferBeginInfo::default()
                        .flags(vk::CommandBufferUsageFlags::ONE_TIME_SUBMIT),
                )
                .expect("failed to begin command buffer on VMemoryManager");

            let copy_regions = [vk::BufferCopy::default().size(size as u64)];
            v_device.device.cmd_copy_buffer(
                command_buffers[0],
                from.buffer,
                to.buffer,
                &copy_regions,
            );
            v_device
                .device
                .end_command_buffer(command_buffers[0])
                .expect("failed to end command buffer on VMemoryManager");

            let submit_infos = [vk::SubmitInfo::default().command_buffers(&command_buffers)];
            v_device
                .device
                .queue_submit(v_device.transfer_queue, &submit_infos, vk::Fence::null())
                .expect("failed to submit command buffer on VMemoryManager");
            v_device
                .device
                .queue_wait_idle(v_device.transfer_queue)
                .expect("failed to wait transfer queue to be idle");
            v_device
                .device
                .free_command_buffers(self.command_pool, &command_buffers);
        };
    }

    pub fn free_memory(&self, v_device: &VDevice, memory: vk::DeviceMemory) {
        unsafe {
            v_device.device.free_memory(memory, None);
        }
    }

    pub fn destroy(&self, v_device: &VDevice) {
        unsafe {
            v_device
                .device
                .destroy_command_pool(self.command_pool, None)
        };
    }
}
