use std::{cell::Cell, sync::Arc};

use crate::{
    commands::{create_command_buffer, create_command_pool},
    images::transition_image_layout,
    vcontext::Vcontext,
};

pub struct RenderLoop {
    // vcontext: Arc<Vcontext>,
    // command_pool: ash::vk::CommandPool,
    // command_buffer: ash::vk::CommandBuffer,
    // present_complete_semaphore: ash::vk::Semaphore,
    // render_finished_semaphore: ash::vk::Semaphore,
    // draw_fence: ash::vk::Fence,
    // image_index: Cell<u32>,
}

impl RenderLoop {
    pub fn new(vcontext: Arc<Vcontext>) -> Self {
        let command_pool = create_command_pool(&vcontext.device, vcontext.graphics_queue_index);
        let command_buffer = create_command_buffer(&vcontext.device, command_pool);

        // let present_complete_semaphore = unsafe {
        //     vcontext
        //         .device
        //         .create_semaphore(&ash::vk::SemaphoreCreateInfo::default(), None)
        //         .expect("unable to create semaphore")
        // };
        // let render_finished_semaphore = unsafe {
        //     vcontext
        //         .device
        //         .create_semaphore(&ash::vk::SemaphoreCreateInfo::default(), None)
        //         .expect("unable to create semaphore")
        // };
        // let draw_fence = unsafe {
        //     vcontext
        //         .device
        //         .create_fence(
        //             &ash::vk::FenceCreateInfo::default().flags(ash::vk::FenceCreateFlags::SIGNALED),
        //             None,
        //         )
        //         .expect("unable to create fence")
        // };

        Self {
            // vcontext,
            // command_pool,
            // command_buffer,
            // present_complete_semaphore,
            // render_finished_semaphore,
            // draw_fence,
            // image_index: Cell::new(0),
        }
    }

    // pub fn draw(&self) {
    //     unsafe {
    //         let fences = [self.draw_fence];
    //         self.vcontext
    //             .device
    //             .wait_for_fences(&fences, true, u64::MAX)
    //             .expect("unable to wait for fence");
    //         self.vcontext
    //             .device
    //             .reset_fences(&fences)
    //             .expect("unable to reset fences");
    //     }

    //     let (image_index, acquire_success) = unsafe {
    //         self.vcontext
    //             .swapchain_device
    //             .acquire_next_image(
    //                 self.vcontext.swapchain,
    //                 u64::MAX,
    //                 self.present_complete_semaphore,
    //                 ash::vk::Fence::null(),
    //             )
    //             .expect("unable to acquire image")
    //     };

    //     let cmd_begin_info = ash::vk::CommandBufferBeginInfo::default();
    //     unsafe {
    //         self.vcontext
    //             .device
    //             .begin_command_buffer(self.command_buffer, &cmd_begin_info)
    //             .expect("unable to begin command buffer")
    //     };

    //     transition_image_layout(
    //         &self.vcontext.device,
    //         self.command_buffer,
    //         self.vcontext.swapchain_images[image_index as usize],
    //         ash::vk::ImageLayout::UNDEFINED,
    //         ash::vk::ImageLayout::COLOR_ATTACHMENT_OPTIMAL,
    //         ash::vk::AccessFlags2::empty(),
    //         ash::vk::AccessFlags2::COLOR_ATTACHMENT_WRITE,
    //         ash::vk::PipelineStageFlags2::COLOR_ATTACHMENT_OUTPUT,
    //         ash::vk::PipelineStageFlags2::COLOR_ATTACHMENT_OUTPUT,
    //     );

    //     let clear_value = ash::vk::ClearValue {
    //         color: ash::vk::ClearColorValue {
    //             float32: [1.0, 1.0, 1.0, 1.0],
    //         },
    //     };
    //     let color_attachments = [ash::vk::RenderingAttachmentInfo::default()
    //         .image_view(self.vcontext.swapchain_image_views[image_index as usize])
    //         .image_layout(ash::vk::ImageLayout::COLOR_ATTACHMENT_OPTIMAL)
    //         .load_op(ash::vk::AttachmentLoadOp::CLEAR)
    //         .store_op(ash::vk::AttachmentStoreOp::STORE)
    //         .clear_value(clear_value)];
    //     let rendering_info = ash::vk::RenderingInfo::default()
    //         .color_attachments(&color_attachments)
    //         .render_area(
    //             ash::vk::Rect2D::default()
    //                 .offset(ash::vk::Offset2D { x: 0, y: 0 })
    //                 .extent(self.vcontext.surface_extent),
    //         )
    //         .layer_count(1);

    //     unsafe {
    //         self.vcontext
    //             .device
    //             .cmd_begin_rendering(self.command_buffer, &rendering_info)
    //     };
    //     unsafe {
    //         self.vcontext.device.cmd_bind_pipeline(
    //             self.command_buffer,
    //             ash::vk::PipelineBindPoint::GRAPHICS,
    //             self.vcontext.pipeline,
    //         )
    //     };

    //     let viewports = [ash::vk::Viewport {
    //         x: 0_f32,
    //         y: 0_f32,
    //         height: self.vcontext.surface_extent.height as f32,
    //         width: self.vcontext.surface_extent.width as f32,
    //         min_depth: 0_f32,
    //         max_depth: 1_f32,
    //     }];
    //     let scissors = [ash::vk::Rect2D {
    //         offset: ash::vk::Offset2D { x: 0, y: 0 },
    //         extent: self.vcontext.surface_extent,
    //     }];

    //     unsafe {
    //         self.vcontext
    //             .device
    //             .cmd_set_viewport(self.command_buffer, 0, &viewports);
    //         self.vcontext
    //             .device
    //             .cmd_set_scissor(self.command_buffer, 0, &scissors);
    //         self.vcontext
    //             .device
    //             .cmd_draw(self.command_buffer, 3, 1, 0, 0);
    //         self.vcontext.device.cmd_end_rendering(self.command_buffer);
    //     }

    //     transition_image_layout(
    //         &self.vcontext.device,
    //         self.command_buffer,
    //         self.vcontext.swapchain_images[image_index as usize],
    //         ash::vk::ImageLayout::COLOR_ATTACHMENT_OPTIMAL,
    //         ash::vk::ImageLayout::PRESENT_SRC_KHR,
    //         ash::vk::AccessFlags2::COLOR_ATTACHMENT_WRITE,
    //         ash::vk::AccessFlags2::empty(),
    //         ash::vk::PipelineStageFlags2::COLOR_ATTACHMENT_OUTPUT,
    //         ash::vk::PipelineStageFlags2::BOTTOM_OF_PIPE,
    //     );

    //     unsafe {
    //         self.vcontext
    //             .device
    //             .end_command_buffer(self.command_buffer)
    //             .expect("unable to end command buffer");
    //     }

    //     let submit_wait_semaphores = [self.present_complete_semaphore];
    //     let submit_signal_semaphores = [self.render_finished_semaphore];
    //     let command_buffers = [self.command_buffer];

    //     let submits = [ash::vk::SubmitInfo::default()
    //         .wait_semaphores(&submit_wait_semaphores)
    //         .wait_dst_stage_mask(&[ash::vk::PipelineStageFlags::COLOR_ATTACHMENT_OUTPUT])
    //         .command_buffers(&command_buffers)
    //         .signal_semaphores(&submit_signal_semaphores)];

    //     unsafe {
    //         self.vcontext
    //             .device
    //             .queue_submit(self.vcontext.graphics_queue, &submits, self.draw_fence)
    //             .expect("unable to submit to queue");
    //     }

    //     let present_wait_semaphore = [self.render_finished_semaphore];
    //     let swapchains = [self.vcontext.swapchain];
    //     let image_indices = [image_index];
    //     let present_info = ash::vk::PresentInfoKHR::default()
    //         .wait_semaphores(&present_wait_semaphore)
    //         .swapchains(&swapchains)
    //         .image_indices(&image_indices);
    //     unsafe {
    //         self.vcontext
    //             .swapchain_device
    //             .queue_present(self.vcontext.graphics_queue, &present_info)
    //             .expect("unable to present queue");
    //     }
    // }
}
