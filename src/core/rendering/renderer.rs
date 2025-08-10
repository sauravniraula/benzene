use std::u64;

use ash::vk;

use crate::core::{device::VDevice, swapchain::VSwapchain};

pub struct VRenderer {
    pub command_pool: vk::CommandPool,
    pub command_buffers: Vec<vk::CommandBuffer>,
    pub ready_to_submit_s: vk::Semaphore,
    pub ready_to_present_s: vk::Semaphore,
    pub buffer_free_f: vk::Fence,
}

impl VRenderer {
    pub fn new(v_device: &VDevice, v_swapchain: &VSwapchain) -> Self {
        let command_pool = unsafe {
            v_device
                .device
                .create_command_pool(
                    &vk::CommandPoolCreateInfo::default()
                        .flags(vk::CommandPoolCreateFlags::RESET_COMMAND_BUFFER)
                        .queue_family_index(v_device.graphics_queue_family_index),
                    None,
                )
                .expect("failed to create command pool")
        };

        let command_buffers = unsafe {
            v_device
                .device
                .allocate_command_buffers(
                    &vk::CommandBufferAllocateInfo::default()
                        .command_pool(command_pool)
                        .command_buffer_count(v_swapchain.image_count),
                )
                .expect("failed to allocate command buffers")
        };

        let ready_to_submit_s = unsafe {
            v_device
                .device
                .create_semaphore(&vk::SemaphoreCreateInfo::default(), None)
                .expect("failed to create semaphore")
        };
        let ready_to_present_s = unsafe {
            v_device
                .device
                .create_semaphore(&vk::SemaphoreCreateInfo::default(), None)
                .expect("failed to create semaphore")
        };

        let buffer_free_f = unsafe {
            v_device
                .device
                .create_fence(
                    &vk::FenceCreateInfo::default().flags(vk::FenceCreateFlags::SIGNALED),
                    None,
                )
                .expect("failed to create fence")
        };

        Self {
            command_pool,
            command_buffers,
            ready_to_submit_s,
            ready_to_present_s,
            buffer_free_f,
        }
    }

    pub fn render(
        &self,
        v_device: &VDevice,
        v_swapchain: &VSwapchain,
        render: impl Fn(vk::CommandBuffer, usize) -> (),
    ) {
        let (command_buffer, image_index) = self.start_draw(v_device, v_swapchain);
        render(command_buffer, image_index);
        self.end_draw(v_device, v_swapchain, command_buffer, image_index);
    }

    pub fn start_draw(
        &self,
        v_device: &VDevice,
        v_swapchain: &VSwapchain,
    ) -> (vk::CommandBuffer, usize) {
        unsafe {
            v_device
                .device
                .wait_for_fences(&[self.buffer_free_f], true, u64::MAX)
                .expect("failed to wait for fence");

            v_device
                .device
                .reset_fences(&[self.buffer_free_f])
                .expect("failed to reset fence");
        }

        let (image_index, _) = unsafe {
            v_swapchain
                .swapchain_device
                .acquire_next_image(
                    v_swapchain.swapchain,
                    u64::MAX,
                    self.ready_to_submit_s,
                    vk::Fence::null(),
                )
                .expect("failed to acquire next image")
        };
        unsafe {
            v_device
                .device
                .begin_command_buffer(
                    self.command_buffers[image_index as usize],
                    &vk::CommandBufferBeginInfo::default(),
                )
                .expect("failed to start command buffer")
        };
        (
            self.command_buffers[image_index as usize],
            image_index as usize,
        )
    }

    pub fn end_draw(
        &self,
        v_device: &VDevice,
        v_swapchain: &VSwapchain,
        command_buffer: vk::CommandBuffer,
        image_index: usize,
    ) {
        unsafe {
            v_device
                .device
                .end_command_buffer(command_buffer)
                .expect("failed to end command buffer");

            v_device
                .device
                .queue_submit(
                    v_device.graphics_queue,
                    &[vk::SubmitInfo::default()
                        .command_buffers(&[command_buffer])
                        .wait_semaphores(&[self.ready_to_submit_s])
                        .signal_semaphores(&[self.ready_to_present_s])],
                    self.buffer_free_f,
                )
                .expect("failed to submit command buffer");

            v_swapchain
                .swapchain_device
                .queue_present(
                    v_device.present_queue,
                    &vk::PresentInfoKHR::default()
                        .swapchains(&[v_swapchain.swapchain])
                        .image_indices(&[image_index as u32])
                        .wait_semaphores(&[self.ready_to_present_s]),
                )
                .expect("failed to present image");
        };
    }

    pub fn destroy(&self, v_device: &VDevice) {
        unsafe {
            v_device
                .device
                .free_command_buffers(self.command_pool, &self.command_buffers);
            v_device
                .device
                .destroy_command_pool(self.command_pool, None);

            v_device.device.destroy_fence(self.buffer_free_f, None);
            [self.ready_to_submit_s, self.ready_to_present_s].map(|each| {
                v_device.device.destroy_semaphore(each, None);
            });
        }
    }
}
