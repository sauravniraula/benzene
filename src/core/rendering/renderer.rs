use std::u64;

use ash::vk;

use crate::core::{device::VDevice, swapchain::VSwapchain};

pub struct VRenderer<'a> {
    pub command_pool: vk::CommandPool,
    pub command_buffers: Vec<vk::CommandBuffer>,
    pub ready_to_submit_s: vk::Semaphore,
    pub ready_to_present_s: vk::Semaphore,
    pub buffer_free_f: vk::Fence,
    v_device: &'a VDevice,
    v_swapchain: &'a VSwapchain,
}

impl<'a> VRenderer<'a> {
    pub fn new(v_device: &'a VDevice, v_swapchain: &'a VSwapchain) -> Self {
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
            v_device,
            v_swapchain,
        }
    }

    pub fn render(&self, render: impl Fn(vk::CommandBuffer, usize) -> ()) {
        let (command_buffer, image_index) = self.start_draw();
        render(command_buffer, image_index);
        self.end_draw(command_buffer, image_index);
    }

    pub fn start_draw(&self) -> (vk::CommandBuffer, usize) {
        unsafe {
            self.v_device
                .device
                .wait_for_fences(&[self.buffer_free_f], true, u64::MAX)
                .expect("failed to wait for fence");

            self.v_device
                .device
                .reset_fences(&[self.buffer_free_f])
                .expect("failed to reset fence");
        }

        let (image_index, _) = unsafe {
            self.v_swapchain
                .swapchain_device
                .acquire_next_image(
                    self.v_swapchain.swapchain,
                    u64::MAX,
                    self.ready_to_submit_s,
                    vk::Fence::null(),
                )
                .expect("failed to acquire next image")
        };
        unsafe {
            self.v_device
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

    pub fn end_draw(&self, command_buffer: vk::CommandBuffer, image_index: usize) {
        unsafe {
            self.v_device
                .device
                .end_command_buffer(command_buffer)
                .expect("failed to end command buffer");

            self.v_device
                .device
                .queue_submit(
                    self.v_device.graphics_queue,
                    &[vk::SubmitInfo::default()
                        .command_buffers(&[command_buffer])
                        .wait_semaphores(&[self.ready_to_submit_s])
                        .signal_semaphores(&[self.ready_to_present_s])],
                    self.buffer_free_f,
                )
                .expect("failed to submit command buffer");

            self.v_swapchain
                .swapchain_device
                .queue_present(
                    self.v_device.present_queue,
                    &vk::PresentInfoKHR::default()
                        .swapchains(&[self.v_swapchain.swapchain])
                        .image_indices(&[image_index as u32])
                        .wait_semaphores(&[self.ready_to_present_s]),
                )
                .expect("failed to present image");
        };
    }
}
