use std::{cell::Cell, sync::Arc};

use crate::{
    commands::{create_command_buffers, create_command_pool},
    constants::MAX_FRAMES_IN_FLIGHT,
    images::transition_image_layout,
    log_info,
    vcontext::Vcontext,
};

pub struct RenderLoop {
    vcontext: Arc<Vcontext>,
    command_pool: ash::vk::CommandPool,
    command_buffers: Vec<ash::vk::CommandBuffer>,
    semaphores_present: Vec<ash::vk::Semaphore>,
    semaphores_render: Vec<ash::vk::Semaphore>,
    fences_draw: Vec<ash::vk::Fence>,
    frame_index: Cell<usize>,
}

impl RenderLoop {
    pub fn new(vcontext: Arc<Vcontext>) -> Self {
        let command_pool = create_command_pool(&vcontext.device, vcontext.graphics_queue_index);
        let command_buffers = create_command_buffers(
            &vcontext.device,
            command_pool,
            vcontext.state.borrow().image_count,
        );

        let semaphores_present: Vec<_> = unsafe {
            (0..MAX_FRAMES_IN_FLIGHT)
                .map(|_| {
                    vcontext
                        .device
                        .create_semaphore(&ash::vk::SemaphoreCreateInfo::default(), None)
                        .expect("unable to create semaphore")
                })
                .collect()
        };
        let semaphores_render = unsafe {
            (0..MAX_FRAMES_IN_FLIGHT)
                .map(|_| {
                    vcontext
                        .device
                        .create_semaphore(&ash::vk::SemaphoreCreateInfo::default(), None)
                        .expect("unable to create semaphore")
                })
                .collect()
        };
        let fences_draw = unsafe {
            (0..MAX_FRAMES_IN_FLIGHT)
                .map(|_| {
                    vcontext
                        .device
                        .create_fence(
                            &ash::vk::FenceCreateInfo::default()
                                .flags(ash::vk::FenceCreateFlags::SIGNALED),
                            None,
                        )
                        .expect("unable to create fence")
                })
                .collect()
        };

        Self {
            vcontext,
            command_pool,
            command_buffers,
            semaphores_present,
            semaphores_render,
            fences_draw,
            frame_index: Cell::new(0),
        }
    }

    pub fn draw(&self) -> bool {
        let frame_index = self.frame_index.get();
        let vcontext_state = self.vcontext.state.borrow();
        let surface_extent = vcontext_state.surface_capabilities.current_extent;

        let fences = &self.fences_draw[frame_index..frame_index + 1];

        unsafe {
            self.vcontext
                .device
                .wait_for_fences(&fences, true, u64::MAX)
                .expect("unable to wait for fence")
        };

        let image_acquire_result = unsafe {
            self.vcontext.swapchain_device.acquire_next_image(
                vcontext_state.swapchain,
                u64::MAX,
                self.semaphores_present[frame_index],
                ash::vk::Fence::null(),
            )
        };

        match image_acquire_result {
            Ok((image_index, is_suboptimal)) => {
                if is_suboptimal {
                    return false;
                }

                unsafe {
                    self.vcontext
                        .device
                        .reset_fences(&fences)
                        .expect("unable to reset fences")
                };

                let command_buffer = self.command_buffers[image_index as usize];

                let cmd_begin_info = ash::vk::CommandBufferBeginInfo::default();
                unsafe {
                    self.vcontext
                        .device
                        .begin_command_buffer(command_buffer, &cmd_begin_info)
                        .expect("unable to begin command buffer")
                };

                transition_image_layout(
                    &self.vcontext.device,
                    command_buffer,
                    vcontext_state.swapchain_images[image_index as usize],
                    ash::vk::ImageLayout::UNDEFINED,
                    ash::vk::ImageLayout::COLOR_ATTACHMENT_OPTIMAL,
                    ash::vk::AccessFlags2::empty(),
                    ash::vk::AccessFlags2::COLOR_ATTACHMENT_WRITE,
                    ash::vk::PipelineStageFlags2::COLOR_ATTACHMENT_OUTPUT,
                    ash::vk::PipelineStageFlags2::COLOR_ATTACHMENT_OUTPUT,
                );

                let clear_value = ash::vk::ClearValue {
                    color: ash::vk::ClearColorValue {
                        float32: [1.0, 1.0, 1.0, 1.0],
                    },
                };
                let color_attachments = [ash::vk::RenderingAttachmentInfo::default()
                    .image_view(vcontext_state.swapchain_image_views[image_index as usize])
                    .image_layout(ash::vk::ImageLayout::COLOR_ATTACHMENT_OPTIMAL)
                    .load_op(ash::vk::AttachmentLoadOp::CLEAR)
                    .store_op(ash::vk::AttachmentStoreOp::STORE)
                    .clear_value(clear_value)];
                let rendering_info = ash::vk::RenderingInfo::default()
                    .color_attachments(&color_attachments)
                    .render_area(
                        ash::vk::Rect2D::default()
                            .offset(ash::vk::Offset2D { x: 0, y: 0 })
                            .extent(surface_extent),
                    )
                    .layer_count(1);

                unsafe {
                    self.vcontext
                        .device
                        .cmd_begin_rendering(command_buffer, &rendering_info)
                };
                unsafe {
                    self.vcontext.device.cmd_bind_pipeline(
                        command_buffer,
                        ash::vk::PipelineBindPoint::GRAPHICS,
                        self.vcontext.pipeline,
                    )
                };

                let viewports = [ash::vk::Viewport {
                    x: 0_f32,
                    y: 0_f32,
                    height: surface_extent.height as f32,
                    width: surface_extent.width as f32,
                    min_depth: 0_f32,
                    max_depth: 1_f32,
                }];
                let scissors = [ash::vk::Rect2D {
                    offset: ash::vk::Offset2D { x: 0, y: 0 },
                    extent: surface_extent,
                }];

                unsafe {
                    self.vcontext
                        .device
                        .cmd_set_viewport(command_buffer, 0, &viewports);
                    self.vcontext
                        .device
                        .cmd_set_scissor(command_buffer, 0, &scissors);
                    self.vcontext.device.cmd_draw(command_buffer, 3, 1, 0, 0);
                    self.vcontext.device.cmd_end_rendering(command_buffer);
                }

                transition_image_layout(
                    &self.vcontext.device,
                    command_buffer,
                    vcontext_state.swapchain_images[image_index as usize],
                    ash::vk::ImageLayout::COLOR_ATTACHMENT_OPTIMAL,
                    ash::vk::ImageLayout::PRESENT_SRC_KHR,
                    ash::vk::AccessFlags2::COLOR_ATTACHMENT_WRITE,
                    ash::vk::AccessFlags2::empty(),
                    ash::vk::PipelineStageFlags2::COLOR_ATTACHMENT_OUTPUT,
                    ash::vk::PipelineStageFlags2::BOTTOM_OF_PIPE,
                );

                unsafe {
                    self.vcontext
                        .device
                        .end_command_buffer(command_buffer)
                        .expect("unable to end command buffer");
                }

                let submits = [ash::vk::SubmitInfo::default()
                    .wait_semaphores(&self.semaphores_present[frame_index..frame_index + 1])
                    .wait_dst_stage_mask(&[ash::vk::PipelineStageFlags::COLOR_ATTACHMENT_OUTPUT])
                    .command_buffers(std::slice::from_ref(&command_buffer))
                    .signal_semaphores(&self.semaphores_render[frame_index..frame_index + 1])];

                unsafe {
                    self.vcontext
                        .device
                        .queue_submit(
                            self.vcontext.graphics_queue,
                            &submits,
                            self.fences_draw[frame_index],
                        )
                        .expect("unable to submit to queue");
                }

                let swapchains = [vcontext_state.swapchain];
                let image_indices = [image_index];
                let present_info = ash::vk::PresentInfoKHR::default()
                    .wait_semaphores(&self.semaphores_render[frame_index..frame_index + 1])
                    .swapchains(&swapchains)
                    .image_indices(&image_indices);

                let present_result = unsafe {
                    self.vcontext
                        .swapchain_device
                        .queue_present(self.vcontext.graphics_queue, &present_info)
                };
                match present_result {
                    Ok(is_suboptimal) => {
                        if is_suboptimal {
                            return false;
                        }

                        self.frame_index
                            .set((frame_index + 1) % MAX_FRAMES_IN_FLIGHT);

                        true
                    }
                    _ => false,
                }
            }
            _ => false,
        }
    }
}
