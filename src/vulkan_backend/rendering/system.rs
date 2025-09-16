use crate::vulkan_backend::memory::image::image_view::VImageView;
use crate::core::ecs::types::Id;
use std::collections::HashMap;
use crate::vulkan_backend::{
    device::VDevice,
    pipeline::VPipelineInfo,
    rendering::{VRenderingSystemConfig, info::VRenderInfo},
};
use ash::vk::{self, Extent2D, Rect2D};

pub struct VRenderingSystem {
    pub attachments_count: usize,
    pub has_color: bool,
    pub has_depth: bool,
    pub render_pass: vk::RenderPass,
    pub pipelines: Vec<vk::Pipeline>,
    pub render_area: Option<Rect2D>,
    pub framebuffers: HashMap<Id, vk::Framebuffer>,
    pub color_clear_value: Option<vk::ClearValue>,
    pub depth_clear_value: Option<vk::ClearValue>,
    pub viewport: Option<vk::Viewport>,
    pub dynamic_viewport: bool,
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
                        .expect("color attachment final layout not provided"),
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
                        .expect("depth attachment final layout not provided"),
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

        let framebuffers: HashMap<Id, vk::Framebuffer> = HashMap::new();

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

        let color_clear_value = if config.color_format.is_some() {
            let mut cv = vk::ClearValue::default();
            cv.color = vk::ClearColorValue {
                float32: [0.18, 0.22, 0.28, 1.0],
            };
            Some(cv)
        } else {
            None
        };
        let depth_clear_value = if config.depth_format.is_some() {
            let mut dv = vk::ClearValue::default();
            dv.depth_stencil = vk::ClearDepthStencilValue {
                depth: 1.0,
                stencil: 0,
            };
            Some(dv)
        } else {
            None
        };
        // No framebuffers yet; viewport/render_area not set
        let viewport: Option<vk::Viewport> = None;

        Self {
            attachments_count: attachment_descriptions.len(),
            has_color: config.color_format.is_some(),
            has_depth: config.depth_format.is_some(),
            render_pass,
            pipelines,
            render_area: None,
            framebuffers,
            color_clear_value,
            depth_clear_value,
            viewport,
            dynamic_viewport: config.dynamic_viewport,
        }
    }

    pub fn set_render_area(&mut self, x: i32, y: i32, width: u32, height: u32) {
        let offset = vk::Offset2D { x, y };
        let extent = vk::Extent2D { width, height };
        self.render_area = Some(vk::Rect2D { offset, extent });
    }

    pub fn set_viewport(
        &mut self,
        x: f32,
        y: f32,
        width: f32,
        height: f32,
        min_depth: f32,
        max_depth: f32,
    ) {
        self.viewport = Some(
            vk::Viewport::default()
                .height(height)
                .width(width)
                .x(x)
                .y(y)
                .min_depth(min_depth)
                .max_depth(max_depth),
        );
    }

    pub fn add_framebuffer(
        &mut self,
        v_device: &VDevice,
        id: Id,
        color_view: Option<&VImageView>,
        depth_view: Option<&VImageView>,
        image_extent: Extent2D,
        update_render_pass: bool,
        update_viewport: bool,
    ) {
        if self.has_color {
            assert!(color_view.is_some(), "render pass expects color attachment, but color_view is None");
        } else {
            assert!(color_view.is_none(), "no color attachment configured; color_view must be None");
        }
        if self.has_depth {
            assert!(depth_view.is_some(), "render pass expects depth attachment, but depth_view is None");
        } else {
            assert!(depth_view.is_none(), "no depth attachment configured; depth_view must be None");
        }

        let mut attachments: Vec<vk::ImageView> = Vec::new();
        if let Some(cv) = color_view {
            attachments.push(cv.image_view);
        }
        if let Some(dv) = depth_view {
            attachments.push(dv.image_view);
        }

        let info = vk::FramebufferCreateInfo::default()
            .attachment_count(self.attachments_count as u32)
            .attachments(&attachments)
            .render_pass(self.render_pass)
            .width(image_extent.width)
            .height(image_extent.height)
            .layers(1);
        let framebuffer = unsafe {
            v_device
                .device
                .create_framebuffer(&info, None)
                .expect("failed to create framebuffer")
        };
        self.framebuffers.insert(id, framebuffer);

        if update_render_pass {
            self.set_render_area(0, 0, image_extent.width, image_extent.height);
        }
        if update_viewport {
            self.set_viewport(
                0f32,
                0f32,
                image_extent.width as f32,
                image_extent.height as f32,
                0f32,
                1f32,
            );
        }
    }

    pub fn start(&self, v_device: &VDevice, command_buffer: vk::CommandBuffer, image_id: &Id) {
        let mut clear_values: Vec<vk::ClearValue> = Vec::new();
        if let Some(cv) = self.color_clear_value.as_ref() {
            clear_values.push(*cv);
        }
        if let Some(dv) = self.depth_clear_value.as_ref() {
            clear_values.push(*dv);
        }

        let render_area = self.render_area.expect("render_area not set");

        let framebuffer = *self
            .framebuffers
            .get(image_id)
            .expect("framebuffer not found for image_id");

        let begin_info = vk::RenderPassBeginInfo::default()
            .render_pass(self.render_pass)
            .clear_values(&clear_values)
            .framebuffer(framebuffer)
            .render_area(render_area);

        unsafe {
            v_device.device.cmd_begin_render_pass(
                command_buffer,
                &begin_info,
                vk::SubpassContents::INLINE,
            );

            if self.dynamic_viewport {
                let viewport = self.viewport.expect("viewport not set");

                v_device
                    .device
                    .cmd_set_viewport(command_buffer, 0, &[viewport]);
                v_device
                    .device
                    .cmd_set_scissor(command_buffer, 0, &[render_area]);
            }
        };
    }

    pub fn end(&self, v_device: &VDevice, info: &VRenderInfo) {
        unsafe {
            v_device.device.cmd_end_render_pass(info.command_buffer);
        }
    }

    pub fn destroy_pipeline_infos(v_device: &VDevice, infos: Vec<VPipelineInfo>) {
        for info in infos.iter() {
            info.destroy(v_device);
        }
    }

    pub fn remove_framebuffer(&mut self, v_device: &VDevice, id: &Id) {
        if let Some(fb) = self.framebuffers.remove(id) {
            unsafe {
                v_device
                    .device
                    .destroy_framebuffer(fb, None);
            }
        }
    }

    pub fn remove_all_framebuffers(&mut self, v_device: &VDevice) {
        unsafe {
            for (_, &framebuffer) in self.framebuffers.iter() {
                v_device.device.destroy_framebuffer(framebuffer, None);
            }
        }
        self.framebuffers.clear();
    }

    pub fn destroy(&self, v_device: &VDevice) {
        unsafe {
            for (_, &framebuffer) in self.framebuffers.iter() {
                v_device.device.destroy_framebuffer(framebuffer, None);
            }
            for &pipeline in self.pipelines.iter() {
                v_device.device.destroy_pipeline(pipeline, None);
            }
            v_device.device.destroy_render_pass(self.render_pass, None);
        }
    }
}
