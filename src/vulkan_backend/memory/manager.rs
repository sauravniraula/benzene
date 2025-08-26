use ash::vk;

use crate::vulkan_backend::device::{VDevice, VPhysicalDevice};

pub struct VMemoryManager {
    transfer_command_pool: vk::CommandPool,
    graphics_command_pool: vk::CommandPool,
}

impl VMemoryManager {
    pub fn new(v_device: &VDevice) -> Self {
        let transfer_command_pool = unsafe {
            v_device
                .device
                .create_command_pool(
                    &vk::CommandPoolCreateInfo::default()
                        .queue_family_index(v_device.transfer_queue_family_index),
                    None,
                )
                .expect("failed to create command pool for VMemoryManager")
        };
        let graphics_command_pool = unsafe {
            v_device
                .device
                .create_command_pool(
                    &vk::CommandPoolCreateInfo::default()
                        .queue_family_index(v_device.graphics_queue_family_index),
                    None,
                )
                .expect("failed to create command pool for VMemoryManager")
        };

        Self {
            transfer_command_pool,
            graphics_command_pool,
        }
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

    pub fn map_memory(
        &self,
        v_device: &VDevice,
        memory: vk::DeviceMemory,
        offset: u64,
        size: u64,
    ) -> *mut u8 {
        unsafe {
            v_device
                .device
                .map_memory(memory, offset, size, vk::MemoryMapFlags::empty())
                .expect("failed to map memory") as *mut u8
        }
    }

    pub fn unmap_memory(&self, v_device: &VDevice, memory: vk::DeviceMemory) {
        unsafe { v_device.device.unmap_memory(memory) };
    }

    pub fn run_single_cmd_submit(
        &self,
        v_device: &VDevice,
        graphics: bool,
        func: impl Fn(vk::CommandBuffer) -> (),
    ) {
        let pool = if graphics {
            self.graphics_command_pool
        } else {
            self.transfer_command_pool
        };

        let queue = if graphics {
            v_device.graphics_queue
        } else {
            v_device.transfer_queue
        };

        let alloc_info = vk::CommandBufferAllocateInfo::default()
            .command_pool(pool)
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

            func(command_buffers[0]);

            v_device
                .device
                .end_command_buffer(command_buffers[0])
                .expect("failed to end command buffer on VMemoryManager");

            let submit_infos = [vk::SubmitInfo::default().command_buffers(&command_buffers)];
            v_device
                .device
                .queue_submit(queue, &submit_infos, vk::Fence::null())
                .expect("failed to submit command buffer on VMemoryManager");
            v_device
                .device
                .queue_wait_idle(queue)
                .expect("failed to wait transfer queue to be idle");
            v_device.device.free_command_buffers(pool, &command_buffers);
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
                .destroy_command_pool(self.transfer_command_pool, None);
            v_device
                .device
                .destroy_command_pool(self.graphics_command_pool, None);
        }
    }
}
