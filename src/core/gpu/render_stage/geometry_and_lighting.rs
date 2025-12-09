use crate::core::model_push_constant::ModelPushConstant;
use crate::log;
use crate::shared::types::Id;
use crate::vulkan_backend::descriptor::VDescriptorSetLayout;
use crate::vulkan_backend::descriptor::config::{
    VDescriptorBindingConfig, VDescriptorLayoutConfig,
};
use crate::vulkan_backend::frame::buffers::VFramebuffers;
use crate::vulkan_backend::memory::image::image_view::VImageView;
use crate::vulkan_backend::pipeline::{VPipelineInfoConfig, create_pipelines_from_infos};
use crate::vulkan_backend::push_constant::VPushConstant;
use crate::vulkan_backend::vertex_input::{BindableVertexInput, Vertex3D};
use crate::vulkan_backend::{
    device::VDevice, frame::context::VFrameRenderContext, pipeline::VPipelineInfo,
};
use ash::vk::{self, Rect2D};

pub struct GeometryLightingRenderStageConfig {
    pub color_format: vk::Format,
    pub depth_format: vk::Format,
}

pub struct GeometryLightingRenderStage {
    pub pipeline_infos: Vec<VPipelineInfo>,
    pub descriptor_set_layouts: Vec<VDescriptorSetLayout>,
    pub render_pass: vk::RenderPass,
    pub pipelines: Vec<vk::Pipeline>,
    pub v_framebuffers: VFramebuffers,
    pub render_area: Option<Rect2D>,
    pub viewport: Option<vk::Viewport>,
    clear_values: Vec<vk::ClearValue>,
}

impl GeometryLightingRenderStage {
    pub fn new(v_device: &VDevice, config: GeometryLightingRenderStageConfig) -> Self {
        let color_attachment = vk::AttachmentDescription::default()
            .samples(vk::SampleCountFlags::TYPE_1)
            .format(config.color_format)
            .load_op(vk::AttachmentLoadOp::CLEAR)
            .store_op(vk::AttachmentStoreOp::STORE)
            .initial_layout(vk::ImageLayout::UNDEFINED)
            .final_layout(vk::ImageLayout::PRESENT_SRC_KHR);

        let color_attachment_ref = vk::AttachmentReference::default()
            .attachment(0)
            .layout(vk::ImageLayout::COLOR_ATTACHMENT_OPTIMAL);

        let depth_attachment = vk::AttachmentDescription::default()
            .format(config.depth_format)
            .samples(vk::SampleCountFlags::TYPE_1)
            .load_op(vk::AttachmentLoadOp::CLEAR)
            .store_op(vk::AttachmentStoreOp::STORE)
            .stencil_load_op(vk::AttachmentLoadOp::DONT_CARE)
            .stencil_store_op(vk::AttachmentStoreOp::DONT_CARE)
            .initial_layout(vk::ImageLayout::UNDEFINED)
            .final_layout(vk::ImageLayout::DEPTH_STENCIL_ATTACHMENT_OPTIMAL);

        let depth_attachment_ref = vk::AttachmentReference::default()
            .attachment(1)
            .layout(vk::ImageLayout::DEPTH_STENCIL_ATTACHMENT_OPTIMAL);

        let subpass = vk::SubpassDescription::default()
            .color_attachments(std::slice::from_ref(&color_attachment_ref))
            .depth_stencil_attachment(&depth_attachment_ref);

        let subpasses = [subpass];

        let mut subpass_dependencies: Vec<vk::SubpassDependency> = Vec::new();
        subpass_dependencies.extend([
            vk::SubpassDependency::default()
                .src_subpass(vk::SUBPASS_EXTERNAL)
                .dst_subpass(0)
                .src_stage_mask(vk::PipelineStageFlags::COLOR_ATTACHMENT_OUTPUT)
                .dst_stage_mask(vk::PipelineStageFlags::COLOR_ATTACHMENT_OUTPUT)
                .dst_access_mask(
                    vk::AccessFlags::COLOR_ATTACHMENT_READ
                        | vk::AccessFlags::COLOR_ATTACHMENT_WRITE,
                ),
            vk::SubpassDependency::default()
                .src_subpass(vk::SUBPASS_EXTERNAL)
                .dst_subpass(0)
                .src_stage_mask(vk::PipelineStageFlags::LATE_FRAGMENT_TESTS)
                .dst_stage_mask(vk::PipelineStageFlags::EARLY_FRAGMENT_TESTS)
                .dst_access_mask(vk::AccessFlags::DEPTH_STENCIL_ATTACHMENT_WRITE),
        ]);

        let attachments = [color_attachment, depth_attachment];
        let render_pass_info = vk::RenderPassCreateInfo::default()
            .subpasses(&subpasses)
            .attachments(&attachments)
            .dependencies(&subpass_dependencies);

        let render_pass = unsafe {
            v_device
                .device
                .create_render_pass(&render_pass_info, None)
                .expect("failed to create render pass")
        };

        // Pipeline
        let vertex_binding_descriptions = Vertex3D::get_binding_descriptions();
        let vertex_attribute_descriptions = Vertex3D::get_attribute_descriptions();

        let model_push_constant = VPushConstant::new::<ModelPushConstant>(
            vk::ShaderStageFlags::VERTEX | vk::ShaderStageFlags::FRAGMENT,
        );

        // Creating Descriptor Layouts
        let global_uniform_layout = VDescriptorSetLayout::new(
            &v_device,
            VDescriptorLayoutConfig {
                bindings: vec![VDescriptorBindingConfig {
                    binding: 0,
                    count: 1,
                    descriptor_type: vk::DescriptorType::UNIFORM_BUFFER,
                    shader_stage: vk::ShaderStageFlags::VERTEX | vk::ShaderStageFlags::FRAGMENT,
                }],
            },
        );
        let lights_uniform_layout = VDescriptorSetLayout::new(
            &v_device,
            VDescriptorLayoutConfig {
                bindings: vec![
                    VDescriptorBindingConfig {
                        binding: 0,
                        count: 1,
                        descriptor_type: vk::DescriptorType::UNIFORM_BUFFER,
                        shader_stage: vk::ShaderStageFlags::FRAGMENT,
                    },
                    VDescriptorBindingConfig {
                        binding: 1,
                        count: 1,
                        descriptor_type: vk::DescriptorType::UNIFORM_BUFFER,
                        shader_stage: vk::ShaderStageFlags::FRAGMENT,
                    },
                    VDescriptorBindingConfig {
                        binding: 2,
                        count: 1,
                        descriptor_type: vk::DescriptorType::UNIFORM_BUFFER,
                        shader_stage: vk::ShaderStageFlags::FRAGMENT,
                    },
                ],
            },
        );
        let image_sampler_layout = VDescriptorSetLayout::new(
            &v_device,
            VDescriptorLayoutConfig {
                bindings: vec![VDescriptorBindingConfig {
                    binding: 0,
                    count: 1,
                    descriptor_type: vk::DescriptorType::COMBINED_IMAGE_SAMPLER,
                    shader_stage: vk::ShaderStageFlags::FRAGMENT,
                }],
            },
        );

        let pipeline_infos = vec![VPipelineInfo::new(
            &v_device,
            VPipelineInfoConfig {
                binding_descriptions: vertex_binding_descriptions.clone(),
                attribute_descriptions: vertex_attribute_descriptions.clone(),
                vertex_shader_file: Some("assets/shaders/shader.vert".into()),
                fragment_shader_file: Some("assets/shaders/shader.frag".into()),
            },
            Some(&model_push_constant),
            &[
                &global_uniform_layout,
                &lights_uniform_layout,
                &image_sampler_layout,
            ],
        )];
        let pipelines = create_pipelines_from_infos(v_device, render_pass, &pipeline_infos);

        let descriptor_set_layouts = vec![
            global_uniform_layout,
            lights_uniform_layout,
            image_sampler_layout,
        ];

        // Clear Values
        let mut cv = vk::ClearValue::default();
        cv.color = vk::ClearColorValue {
            float32: [0.18, 0.22, 0.28, 1.0],
        };
        let mut dv = vk::ClearValue::default();
        dv.depth_stencil = vk::ClearDepthStencilValue {
            depth: 1.0,
            stencil: 0,
        };

        // Framebuffers
        let v_framebuffers = VFramebuffers::new();

        Self {
            pipeline_infos,
            descriptor_set_layouts,
            render_pass,
            pipelines,
            v_framebuffers,
            render_area: None,
            viewport: None,
            clear_values: vec![cv, dv],
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

    pub fn init_framebuffers(
        &mut self,
        v_device: &VDevice,
        ids: &Vec<Id>,
        color_views: &Vec<VImageView>,
        depth_view: &VImageView,
        image_extent: vk::Extent2D,
    ) {
        self.v_framebuffers.remove_all_framebuffers(v_device);

        for (idx, id) in ids.iter().enumerate() {
            self.v_framebuffers.add_framebuffer(
                v_device,
                self.render_pass,
                *id,
                Some(&color_views[idx]),
                Some(depth_view),
                image_extent,
            );
        }

        self.set_render_area(0, 0, image_extent.width, image_extent.height);
        self.set_viewport(
            0f32,
            0f32,
            image_extent.width as f32,
            image_extent.height as f32,
            0f32,
            1f32,
        );
    }

    pub fn start(&self, v_device: &VDevice, cmd: vk::CommandBuffer, image_id: &Id) {
        log!("Starting geometry and lignting render pass");

        let render_area = self.render_area.expect("render_area not set");

        let framebuffer = *self.v_framebuffers.get_by_id(image_id);

        let begin_info = vk::RenderPassBeginInfo::default()
            .render_pass(self.render_pass)
            .clear_values(&self.clear_values)
            .framebuffer(framebuffer)
            .render_area(render_area);

        unsafe {
            v_device
                .device
                .cmd_begin_render_pass(cmd, &begin_info, vk::SubpassContents::INLINE);

            let viewport = self.viewport.expect("viewport not set");
            v_device.device.cmd_set_viewport(cmd, 0, &[viewport]);
            v_device.device.cmd_set_scissor(cmd, 0, &[render_area]);
        };
    }

    pub fn end(&self, v_device: &VDevice, ctx: &VFrameRenderContext) {
        unsafe {
            v_device.device.cmd_end_render_pass(ctx.cmd);
        }
    }

    pub fn destroy_pipeline_infos(v_device: &VDevice, infos: Vec<VPipelineInfo>) {
        for info in infos.iter() {
            info.destroy(v_device);
        }
    }

    pub fn destroy(&self, v_device: &VDevice) {
        unsafe {
            self.v_framebuffers.destroy(v_device);
            for each in self.pipeline_infos.iter() {
                each.destroy(v_device);
            }
            for each in self.descriptor_set_layouts.iter() {
                each.destroy(v_device);
            }
            for &pipeline in self.pipelines.iter() {
                v_device.device.destroy_pipeline(pipeline, None);
            }
            v_device.device.destroy_render_pass(self.render_pass, None);
        }
    }
}
