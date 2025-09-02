use crate::vulkan_backend::{
    backend_event::VBackendEvent,
    device::VDevice,
    pipeline::VPipelineInfo,
    rendering::{VRenderingSystemConfig, info::VRenderInfo},
};
use ash::vk::{self, Extent2D, Offset2D, Rect2D};

pub struct VRenderingSystem {
    pub attachments_count: usize,
    pub render_pass: vk::RenderPass,
    pub pipelines: Vec<vk::Pipeline>,
    pub render_area: Rect2D,
    pub framebuffers: Vec<vk::Framebuffer>,
    pub clear_values: Vec<vk::ClearValue>,
    pub viewport: vk::Viewport,
}

impl VRenderingSystem {
    pub fn new(v_device: &VDevice, config: VRenderingSystemConfig) -> Self {
        let mut attachment_descriptions: Vec<vk::AttachmentDescription> = Vec::new();
        let mut attachment_refs: Vec<vk::AttachmentReference> = Vec::new();
        let mut depth_ref_opt: Option<vk::AttachmentReference> = None;

        if let Some(color_format) = config.color_format {
            let color_attachment = vk::AttachmentDescription::default()
                .samples(vk::SampleCountFlags::TYPE_1)
                .format(color_format)
                .load_op(vk::AttachmentLoadOp::CLEAR)
                .store_op(vk::AttachmentStoreOp::STORE)
                .initial_layout(vk::ImageLayout::UNDEFINED)
                .final_layout(
                    config
                        .color_final_layout
                        .unwrap_or(vk::ImageLayout::COLOR_ATTACHMENT_OPTIMAL),
                );

            let color_attachment_ref = vk::AttachmentReference::default()
                .attachment(attachment_descriptions.len() as u32)
                .layout(vk::ImageLayout::COLOR_ATTACHMENT_OPTIMAL);

            attachment_descriptions.push(color_attachment);
            attachment_refs.push(color_attachment_ref);
        }

        if let Some(depth_format) = config.depth_format {
            let depth_attachment = vk::AttachmentDescription::default()
                .format(depth_format)
                .samples(vk::SampleCountFlags::TYPE_1)
                .load_op(vk::AttachmentLoadOp::CLEAR)
                .store_op(vk::AttachmentStoreOp::STORE)
                .stencil_load_op(vk::AttachmentLoadOp::DONT_CARE)
                .stencil_store_op(vk::AttachmentStoreOp::DONT_CARE)
                .initial_layout(vk::ImageLayout::UNDEFINED)
                .final_layout(
                    config
                        .depth_final_layout
                        .unwrap_or(vk::ImageLayout::DEPTH_STENCIL_ATTACHMENT_OPTIMAL),
                );

            let depth_attachment_ref = vk::AttachmentReference::default()
                .attachment(attachment_descriptions.len() as u32)
                .layout(vk::ImageLayout::DEPTH_STENCIL_ATTACHMENT_OPTIMAL);

            attachment_descriptions.push(depth_attachment);
            depth_ref_opt = Some(depth_attachment_ref);
        }

        assert!(
            !attachment_descriptions.is_empty(),
            "VRenderingSystem requires at least one attachment (color or depth)"
        );

        let mut subpass = vk::SubpassDescription::default();
        if !attachment_refs.is_empty() {
            subpass = subpass.color_attachments(&attachment_refs);
        }
        if let Some(depth_ref) = depth_ref_opt.as_ref() {
            subpass = subpass.depth_stencil_attachment(depth_ref);
        }

        let subpasses = [subpass];

        let mut subpass_dependencies: Vec<vk::SubpassDependency> = Vec::new();
        if config.color_format.is_some() {
            subpass_dependencies.push(
                vk::SubpassDependency::default()
                    .src_subpass(vk::SUBPASS_EXTERNAL)
                    .dst_subpass(0)
                    .src_stage_mask(vk::PipelineStageFlags::COLOR_ATTACHMENT_OUTPUT)
                    .dst_stage_mask(vk::PipelineStageFlags::COLOR_ATTACHMENT_OUTPUT)
                    .dst_access_mask(
                        vk::AccessFlags::COLOR_ATTACHMENT_READ
                            | vk::AccessFlags::COLOR_ATTACHMENT_WRITE,
                    ),
            );
        }
        if config.depth_format.is_some() {
            subpass_dependencies.push(
                vk::SubpassDependency::default()
                    .src_subpass(vk::SUBPASS_EXTERNAL)
                    .dst_subpass(0)
                    .src_stage_mask(vk::PipelineStageFlags::LATE_FRAGMENT_TESTS)
                    .dst_stage_mask(vk::PipelineStageFlags::EARLY_FRAGMENT_TESTS)
                    .dst_access_mask(vk::AccessFlags::DEPTH_STENCIL_ATTACHMENT_WRITE),
            );
        }

        let render_pass_info = vk::RenderPassCreateInfo::default()
            .subpasses(&subpasses)
            .attachments(&attachment_descriptions)
            .dependencies(&subpass_dependencies);

        let render_pass = unsafe {
            v_device
                .device
                .create_render_pass(&render_pass_info, None)
                .expect("failed to create render pass")
        };

        let framebuffers = VRenderingSystem::create_framebuffers_raw(
            v_device,
            attachment_descriptions.len() as u32,
            render_pass,
            config.color_image_views,
            config.depth_image_views,
            config.extent,
        );

        let mut pipeline_create_infos = Vec::new();

        let mut vertex_input_states = Vec::new();
        let mut input_assembly_states = Vec::new();
        let mut shader_states = Vec::new();
        let mut rasterization_states = Vec::new();
        let mut depth_stencil_states = Vec::new();
        let mut multisampling_states = Vec::new();
        let mut color_blend_states = Vec::new();
        let mut dynamic_states = Vec::new();
        let mut viewport_states = Vec::new();

        for info in config.pipeline_infos {
            vertex_input_states.push(info.get_vertex_input_state());
            input_assembly_states.push(info.get_input_assembly_state());
            shader_states.push(info.get_shader_states());
            rasterization_states.push(info.get_rasterization_state());
            depth_stencil_states.push(info.get_depth_stencil_state());
            multisampling_states.push(info.get_multisampling_state());
            color_blend_states.push(info.get_color_blend_state());
            dynamic_states.push(info.get_dynamic_state());
            viewport_states.push(info.get_viewport_state());
        }

        for i in 0..config.pipeline_infos.len() {
            let pipeline_create_info = vk::GraphicsPipelineCreateInfo::default()
                .render_pass(render_pass)
                .vertex_input_state(&vertex_input_states[i])
                .input_assembly_state(&input_assembly_states[i])
                .stages(&shader_states[i])
                .rasterization_state(&rasterization_states[i])
                .depth_stencil_state(&depth_stencil_states[i])
                .multisample_state(&multisampling_states[i])
                .color_blend_state(&color_blend_states[i])
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

        let mut clear_values: Vec<vk::ClearValue> = Vec::new();
        if config.color_format.is_some() {
            let mut cv = vk::ClearValue::default();
            cv.color = vk::ClearColorValue {
                float32: [0.18, 0.22, 0.28, 1.0],
            };
            clear_values.push(cv);
        }
        if config.depth_format.is_some() {
            let mut dv = vk::ClearValue::default();
            dv.depth_stencil = vk::ClearDepthStencilValue {
                depth: 1.0,
                stencil: 0,
            };
            clear_values.push(dv);
        }

        let viewport = VRenderingSystem::get_viewport(config.extent);

        Self {
            attachments_count: attachment_descriptions.len(),
            render_pass,
            pipelines,
            render_area: Rect2D {
                offset: Offset2D { x: 0, y: 0 },
                extent: vk::Extent2D {
                    width: config.extent.width,
                    height: config.extent.height,
                },
            },
            framebuffers,
            clear_values,
            viewport,
        }
    }

    pub fn get_viewport(image_extent: Extent2D) -> vk::Viewport {
        vk::Viewport::default()
            .height(image_extent.height as f32)
            .width(image_extent.width as f32)
            .x(0f32)
            .y(0f32)
            .min_depth(0f32)
            .max_depth(1f32)
    }

    pub fn create_framebuffers_raw(
        v_device: &VDevice,
        attachments_count: u32,
        render_pass: vk::RenderPass,
        color_image_views: Option<&[vk::ImageView]>,
        depth_image_views: Option<&[vk::ImageView]>,
        image_extent: Extent2D,
    ) -> Vec<vk::Framebuffer> {
        let color_len = color_image_views.map(|v| v.len()).unwrap_or(0);
        let depth_len = depth_image_views.map(|v| v.len()).unwrap_or(0);
        let framebuffer_count = if color_len > 0 {
            color_len
        } else {
            depth_len.max(1)
        };

        (0..framebuffer_count)
            .map(|i| {
                let mut attachments: Vec<vk::ImageView> = Vec::new();
                if let Some(colors) = color_image_views {
                    attachments.push(colors[i]);
                }
                if let Some(depths) = depth_image_views {
                    let depth_view = if depths.len() == 1 {
                        depths[0]
                    } else {
                        depths[i]
                    };
                    attachments.push(depth_view);
                }

                let info = vk::FramebufferCreateInfo::default()
                    .attachment_count(attachments_count)
                    .attachments(&attachments)
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

    pub fn update_framebuffers(
        &mut self,
        v_device: &VDevice,
        color_image_views: Option<&[vk::ImageView]>,
        depth_image_views: Option<&[vk::ImageView]>,
        image_extent: Extent2D,
    ) {
        self.destroy_framebuffers(v_device);

        self.viewport = VRenderingSystem::get_viewport(image_extent);
        self.framebuffers = VRenderingSystem::create_framebuffers_raw(
            v_device,
            self.attachments_count as u32,
            self.render_pass,
            color_image_views,
            depth_image_views,
            image_extent,
        );
        self.render_area = Rect2D {
            offset: self.render_area.offset,
            extent: image_extent,
        };
    }

    pub fn start(&self, v_device: &VDevice, info: &VRenderInfo) {
        let begin_info = vk::RenderPassBeginInfo::default()
            .render_pass(self.render_pass)
            .clear_values(&self.clear_values)
            .framebuffer(self.framebuffers[info.image_index])
            .render_area(self.render_area);

        unsafe {
            v_device.device.cmd_begin_render_pass(
                info.command_buffer,
                &begin_info,
                vk::SubpassContents::INLINE,
            );

            v_device
                .device
                .cmd_set_viewport(info.command_buffer, 0, &[self.viewport]);
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
                let color_views: Vec<vk::ImageView> = v_swapchain
                    .v_image_views
                    .iter()
                    .map(|v| v.image_view)
                    .collect();
                let depth_views: Vec<vk::ImageView> =
                    vec![v_swapchain.depth_v_image_view.image_view];
                self.update_framebuffers(
                    v_device,
                    Some(&color_views),
                    Some(&depth_views),
                    v_swapchain.image_extent,
                );
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
