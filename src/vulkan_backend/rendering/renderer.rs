use std::{
    cell::Cell,
    u64,
};

use ash::vk;

use crate::vulkan_backend::{
    device::VDevice,
    rendering::{VRenderResult, info::VRenderInfo},
    swapchain::VSwapchain,
};

pub struct VRenderer {
    pub command_pool: vk::CommandPool,
    pub command_buffers: Vec<vk::CommandBuffer>,
    pub ready_to_submit_semaphores: Vec<vk::Semaphore>,
    pub ready_to_present_semaphores: Vec<vk::Semaphore>,
    pub buffer_free_fences: Vec<vk::Fence>,
    pub max_frames: usize,
    pub frame_index: Cell<usize>,
}

impl VRenderer {
    pub fn new(v_device: &VDevice, max_frames: usize) -> Self {
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
                        .command_buffer_count(max_frames as u32),
                )
                .expect("failed to allocate command buffers")
        };

        let ready_to_submit_semaphores = unsafe {
            (0..max_frames)
                .map(|_| {
                    v_device
                        .device
                        .create_semaphore(&vk::SemaphoreCreateInfo::default(), None)
                        .expect("failed to create semaphore")
                })
                .collect()
        };
        let ready_to_present_semaphores = unsafe {
            (0..max_frames)
                .map(|_| {
                    v_device
                        .device
                        .create_semaphore(&vk::SemaphoreCreateInfo::default(), None)
                        .expect("failed to create semaphore")
                })
                .collect()
        };

        let buffer_free_fences = unsafe {
            (0..max_frames)
                .map(|_| {
                    v_device
                        .device
                        .create_fence(
                            &vk::FenceCreateInfo::default().flags(vk::FenceCreateFlags::SIGNALED),
                            None,
                        )
                        .expect("failed to create fence")
                })
                .collect()
        };

        Self {
            command_pool,
            command_buffers,
            ready_to_submit_semaphores,
            ready_to_present_semaphores,
            buffer_free_fences,
            max_frames,
            frame_index: Cell::new(0),
        }
    }

    pub fn render(
        &self,
        v_device: &VDevice,
        v_swapchain: &VSwapchain,
        render: impl Fn(VRenderInfo) -> (),
    ) -> VRenderResult {
        let frame_index = self.frame_index.get();

        match self.start_draw(v_device, v_swapchain, frame_index) {
            Ok((command_buffer, image_index)) => {
                render(VRenderInfo {
                    command_buffer,
                    image_id: v_swapchain.image_ids[image_index],
                    frame_index,
                });

                let end_draw_result = self.end_draw(
                    v_device,
                    v_swapchain,
                    command_buffer,
                    image_index,
                    frame_index,
                );
                if let VRenderResult::Ok = end_draw_result {
                    self.frame_index
                        .replace((frame_index + 1) % self.max_frames);
                }
                end_draw_result
            }
            Err(err) => err,
        }
    }

    pub fn start_draw(
        &self,
        v_device: &VDevice,
        v_swapchain: &VSwapchain,
        frame_index: usize,
    ) -> Result<(vk::CommandBuffer, usize), VRenderResult> {
        unsafe {
            v_device
                .device
                .wait_for_fences(
                    &self.buffer_free_fences[frame_index..frame_index + 1],
                    true,
                    u64::MAX,
                )
                .expect("failed to wait for fence");
        }

        let image_acquire_result = unsafe {
            v_swapchain.swapchain_device.acquire_next_image(
                v_swapchain.swapchain,
                u64::MAX,
                self.ready_to_submit_semaphores[frame_index],
                vk::Fence::null(),
            )
        };

        match image_acquire_result {
            Ok((image_index, is_suboptimal)) => {
                if is_suboptimal {
                    return Err(VRenderResult::RecreateSwapchain);
                }
                unsafe {
                    v_device
                        .device
                        .reset_fences(&self.buffer_free_fences[frame_index..frame_index + 1])
                        .expect("failed to reset fence");

                    v_device
                        .device
                        .begin_command_buffer(
                            self.command_buffers[frame_index],
                            &vk::CommandBufferBeginInfo::default(),
                        )
                        .expect("failed to start command buffer")
                };
                Ok((self.command_buffers[frame_index], image_index as usize))
            }
            _ => return Err(VRenderResult::RecreateSwapchain),
        }
    }

    pub fn end_draw(
        &self,
        v_device: &VDevice,
        v_swapchain: &VSwapchain,
        command_buffer: vk::CommandBuffer,
        image_index: usize,
        frame_index: usize,
    ) -> VRenderResult {
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
                        .wait_dst_stage_mask(&[vk::PipelineStageFlags::COLOR_ATTACHMENT_OUTPUT])
                        .wait_semaphores(
                            &self.ready_to_submit_semaphores[frame_index..frame_index + 1],
                        )
                        .signal_semaphores(
                            &self.ready_to_present_semaphores[frame_index..frame_index + 1],
                        )],
                    self.buffer_free_fences[frame_index],
                )
                .expect("failed to submit command buffer");

            let queue_present_result = v_swapchain.swapchain_device.queue_present(
                v_device.present_queue,
                &vk::PresentInfoKHR::default()
                    .swapchains(&[v_swapchain.swapchain])
                    .image_indices(&[image_index as u32])
                    .wait_semaphores(
                        &self.ready_to_present_semaphores[frame_index..frame_index + 1],
                    ),
            );
            return match queue_present_result {
                Ok(is_suboptimal) => {
                    if is_suboptimal {
                        return VRenderResult::RecreateSwapchain;
                    }
                    return VRenderResult::Ok;
                }
                Err(_) => VRenderResult::RecreateSwapchain,
            };
        };
    }

    pub fn destroy(&self, v_device: &VDevice) {
        unsafe {
            for &each_semaphore in self.ready_to_submit_semaphores.iter() {
                v_device.device.destroy_semaphore(each_semaphore, None);
            }
            for &each_semaphore in self.ready_to_present_semaphores.iter() {
                v_device.device.destroy_semaphore(each_semaphore, None);
            }
            for &each_fence in self.buffer_free_fences.iter() {
                v_device.device.destroy_fence(each_fence, None);
            }

            v_device
                .device
                .free_command_buffers(self.command_pool, &self.command_buffers);
            v_device
                .device
                .destroy_command_pool(self.command_pool, None);
        }
    }
}
