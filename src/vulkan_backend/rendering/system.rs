use crate::vulkan_backend::{
    backend_event::VBackendEvent,
    device::VDevice,
    pipeline::VPipelineInfo,
    rendering::{VRenderingSystemConfig, info::VRenderInfo},
    swapchain::VSwapchain,
};
use ash::vk::{self, Extent2D, Offset2D, Rect2D};

pub struct VRenderingSystem {
    pub attachments_count: usize,
    pub render_pass: vk::RenderPass,
    pub pipelines: Vec<vk::Pipeline>,
    pub render_area: Rect2D,
    pub framebuffers: Vec<vk::Framebuffer>,
}

impl VRenderingSystem {
    pub fn new(
        v_device: &VDevice,
        v_swapchain: &VSwapchain,
        config: VRenderingSystemConfig,
    ) -> Self {
        let attachment_ref_1 = vk::AttachmentReference::default()
            .attachment(0)
            .layout(vk::ImageLayout::COLOR_ATTACHMENT_OPTIMAL);
        let attachment_refs = [attachment_ref_1];
        let subpass_1 = vk::SubpassDescription::default().color_attachments(&attachment_refs);
        let subpasses = [subpass_1];

        let attachment_1 = vk::AttachmentDescription::default()
            .samples(vk::SampleCountFlags::TYPE_1)
            .format(v_swapchain.format)
            .load_op(vk::AttachmentLoadOp::CLEAR)
            .final_layout(vk::ImageLayout::PRESENT_SRC_KHR);

        let attachments = [attachment_1];
        let attachments_count = attachments.len();
        let render_pass_info = vk::RenderPassCreateInfo::default()
            .subpasses(&subpasses)
            .attachments(&attachments);

        let render_pass = unsafe {
            v_device
                .device
                .create_render_pass(&render_pass_info, None)
                .expect("failed to create render pass")
        };

        let framebuffers = VRenderingSystem::create_framebuffers(
            v_device,
            attachments_count as u32,
            render_pass,
            &v_swapchain.image_views,
            v_swapchain.image_extent,
        );

        let mut pipeline_create_infos = Vec::new();

        let mut vertex_input_states = Vec::new();
        let mut input_assembly_states = Vec::new();
        let mut shader_stages = Vec::new();
        let mut rasterization_stages = Vec::new();
        let mut multisampling_stages = Vec::new();
        let mut color_blend_stages = Vec::new();
        let mut dynamic_states = Vec::new();
        let mut viewport_states = Vec::new();

        for info in config.pipeline_infos {
            vertex_input_states.push(info.get_vertex_input_stage());
            input_assembly_states.push(info.get_input_assembly_stage());
            shader_stages.push(info.get_shader_stages());
            rasterization_stages.push(info.get_rasterization_stage());
            multisampling_stages.push(info.get_multisampling_stage());
            color_blend_stages.push(info.get_color_blend_stage());
            dynamic_states.push(info.get_dynamic_state());
            viewport_states.push(info.get_viewport_state());
        }

        for i in 0..config.pipeline_infos.len() {
            let pipeline_create_info = vk::GraphicsPipelineCreateInfo::default()
                .render_pass(render_pass)
                .vertex_input_state(&vertex_input_states[i])
                .input_assembly_state(&input_assembly_states[i])
                .stages(&shader_stages[i])
                .rasterization_state(&rasterization_stages[i])
                .multisample_state(&multisampling_stages[i])
                .color_blend_state(&color_blend_stages[i])
                .dynamic_state(&dynamic_states[i])
                .viewport_state(&viewport_states[i])
                .layout(config.pipeline_infos[i].layout);
            pipeline_create_infos.push(pipeline_create_info);
        }

        let pipelines = unsafe {
            Vec::from(
                v_device
                    .device
                    .create_graphics_pipelines(
                        vk::PipelineCache::null(),
                        &pipeline_create_infos,
                        None,
                    )
                    .expect("failed to create pipelines"),
            )
        };

        Self {
            attachments_count,
            render_pass,
            pipelines,
            render_area: Rect2D {
                offset: Offset2D { x: 0, y: 0 },
                extent: v_swapchain.image_extent,
            },
            framebuffers,
        }
    }

    pub fn create_framebuffers(
        v_device: &VDevice,
        attachments_count: u32,
        render_pass: vk::RenderPass,
        image_views: &Vec<vk::ImageView>,
        image_extent: Extent2D,
    ) -> Vec<vk::Framebuffer> {
        (0..image_views.len())
            .map(|i| {
                let info = vk::FramebufferCreateInfo::default()
                    .attachment_count(attachments_count)
                    .attachments(&image_views[i..i + 1])
                    .render_pass(render_pass)
                    .width(image_extent.width)
                    .height(image_extent.height)
                    .layers(1);
                unsafe {
                    v_device
                        .device
                        .create_framebuffer(&info, None)
                        .expect("failed to create framebuffer")
                }
            })
            .collect()
    }

    pub fn start(&self, v_device: &VDevice, info: &VRenderInfo) {
        let mut clear_values = [vk::ClearValue::default()];
        clear_values[0].color = vk::ClearColorValue {
            float32: [0.0, 0.0, 0.0, 1.0],
        };
        let begin_info = vk::RenderPassBeginInfo::default()
            .render_pass(self.render_pass)
            .clear_values(&clear_values)
            .framebuffer(self.framebuffers[info.image_index])
            .render_area(self.render_area);

        unsafe {
            v_device.device.cmd_begin_render_pass(
                info.command_buffer,
                &begin_info,
                vk::SubpassContents::INLINE,
            );

            let viewport = vk::Viewport::default()
                .height(self.render_area.extent.height as f32)
                .width(self.render_area.extent.width as f32)
                .x(0f32)
                .y(0f32)
                .min_depth(0f32)
                .max_depth(1f32);
            v_device
                .device
                .cmd_set_viewport(info.command_buffer, 0, &[viewport]);
            v_device
                .device
                .cmd_set_scissor(info.command_buffer, 0, &[self.render_area]);
        };
    }

    pub fn end(&self, v_device: &VDevice, info: &VRenderInfo) {
        unsafe {
            v_device.device.cmd_end_render_pass(info.command_buffer);
        }
    }

    pub fn handle_backend_event(&mut self, event: &VBackendEvent) {
        match event {
            VBackendEvent::UpdateFramebuffers(v_device, v_swapchain) => {
                self.destroy_framebuffers(v_device);

                let new_image_extent = v_swapchain.image_extent;
                self.framebuffers = VRenderingSystem::create_framebuffers(
                    v_device,
                    self.attachments_count as u32,
                    self.render_pass,
                    &v_swapchain.image_views,
                    new_image_extent,
                );
                self.render_area = Rect2D {
                    offset: self.render_area.offset,
                    extent: new_image_extent,
                }
            }
            _ => {}
        }
    }

    pub fn destroy_pipeline_infos(v_device: &VDevice, infos: Vec<VPipelineInfo>) {
        for info in infos.iter() {
            info.destroy(v_device);
        }
    }

    pub fn destroy_framebuffers(&self, v_device: &VDevice) {
        unsafe {
            for &framebuffer in self.framebuffers.iter() {
                v_device.device.destroy_framebuffer(framebuffer, None);
            }
        }
    }

    pub fn destroy(&self, v_device: &VDevice) {
        self.destroy_framebuffers(v_device);
        unsafe {
            for &pipeline in self.pipelines.iter() {
                v_device.device.destroy_pipeline(pipeline, None);
            }
            v_device.device.destroy_render_pass(self.render_pass, None);
        }
    }
}
