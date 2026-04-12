use std::{mem::size_of, sync::Arc};

use ash::vk;
use memoffset::offset_of;

use crate::{
    assets::MeshVertex,
    constants::MAX_FRAMES_IN_FLIGHT,
    error::{EngineError, Result},
    render::vulkan::VContext,
    ui::SceneViewport,
};

use super::{ModelPushConstants, create_descriptor_set_layout};

pub(super) struct GeometryPass {
    context: Arc<VContext>,
    pub(super) render_pass: vk::RenderPass,
    pub(super) pipeline_layout: vk::PipelineLayout,
    pub(super) pipeline: vk::Pipeline,
    pub(super) global_layout: vk::DescriptorSetLayout,
    pub(super) lights_layout: vk::DescriptorSetLayout,
    material_layout: vk::DescriptorSetLayout,
    pub(super) scene_descriptor_pool: vk::DescriptorPool,
    pub(super) framebuffers: Vec<vk::Framebuffer>,
}

impl GeometryPass {
    pub(super) fn new(
        context: Arc<VContext>,
        color_format: vk::Format,
        depth_format: vk::Format,
    ) -> Result<Self> {
        let global_layout = create_descriptor_set_layout(
            context.device(),
            &[vk::DescriptorSetLayoutBinding::default()
                .binding(0)
                .descriptor_count(1)
                .descriptor_type(vk::DescriptorType::UNIFORM_BUFFER)
                .stage_flags(vk::ShaderStageFlags::VERTEX | vk::ShaderStageFlags::FRAGMENT)],
        )?;
        let lights_layout = create_descriptor_set_layout(
            context.device(),
            &[
                vk::DescriptorSetLayoutBinding::default()
                    .binding(0)
                    .descriptor_count(1)
                    .descriptor_type(vk::DescriptorType::UNIFORM_BUFFER)
                    .stage_flags(vk::ShaderStageFlags::FRAGMENT),
                vk::DescriptorSetLayoutBinding::default()
                    .binding(1)
                    .descriptor_count(1)
                    .descriptor_type(vk::DescriptorType::UNIFORM_BUFFER)
                    .stage_flags(vk::ShaderStageFlags::FRAGMENT),
                vk::DescriptorSetLayoutBinding::default()
                    .binding(2)
                    .descriptor_count(1)
                    .descriptor_type(vk::DescriptorType::UNIFORM_BUFFER)
                    .stage_flags(vk::ShaderStageFlags::FRAGMENT),
            ],
        )?;
        let material_layout = create_descriptor_set_layout(
            context.device(),
            &[vk::DescriptorSetLayoutBinding::default()
                .binding(0)
                .descriptor_count(1)
                .descriptor_type(vk::DescriptorType::COMBINED_IMAGE_SAMPLER)
                .stage_flags(vk::ShaderStageFlags::FRAGMENT)],
        )?;

        let attachments = [
            vk::AttachmentDescription::default()
                .format(color_format)
                .samples(vk::SampleCountFlags::TYPE_1)
                .load_op(vk::AttachmentLoadOp::CLEAR)
                .store_op(vk::AttachmentStoreOp::STORE)
                .initial_layout(vk::ImageLayout::UNDEFINED)
                .final_layout(vk::ImageLayout::PRESENT_SRC_KHR),
            vk::AttachmentDescription::default()
                .format(depth_format)
                .samples(vk::SampleCountFlags::TYPE_1)
                .load_op(vk::AttachmentLoadOp::CLEAR)
                .store_op(vk::AttachmentStoreOp::STORE)
                .stencil_load_op(vk::AttachmentLoadOp::DONT_CARE)
                .stencil_store_op(vk::AttachmentStoreOp::DONT_CARE)
                .initial_layout(vk::ImageLayout::UNDEFINED)
                .final_layout(vk::ImageLayout::DEPTH_STENCIL_ATTACHMENT_OPTIMAL),
        ];
        let color_attachment_ref = vk::AttachmentReference::default()
            .attachment(0)
            .layout(vk::ImageLayout::COLOR_ATTACHMENT_OPTIMAL);
        let depth_attachment_ref = vk::AttachmentReference::default()
            .attachment(1)
            .layout(vk::ImageLayout::DEPTH_STENCIL_ATTACHMENT_OPTIMAL);
        let subpass = vk::SubpassDescription::default()
            .pipeline_bind_point(vk::PipelineBindPoint::GRAPHICS)
            .color_attachments(std::slice::from_ref(&color_attachment_ref))
            .depth_stencil_attachment(&depth_attachment_ref);
        let dependencies = [
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
        ];
        let render_pass_info = vk::RenderPassCreateInfo::default()
            .attachments(&attachments)
            .subpasses(std::slice::from_ref(&subpass))
            .dependencies(&dependencies);
        let render_pass = unsafe {
            context
                .device()
                .create_render_pass(&render_pass_info, None)
                .map_err(|result| EngineError::vk("creating geometry render pass", result))?
        };

        let shader_entry = c"main";
        let vertex_module = context.load_shader_module("assets/shaders/shader.vert")?;
        let fragment_module = context.load_shader_module("assets/shaders/shader.frag")?;
        let shader_stages = [
            vk::PipelineShaderStageCreateInfo::default()
                .module(vertex_module)
                .name(shader_entry)
                .stage(vk::ShaderStageFlags::VERTEX),
            vk::PipelineShaderStageCreateInfo::default()
                .module(fragment_module)
                .name(shader_entry)
                .stage(vk::ShaderStageFlags::FRAGMENT),
        ];

        let set_layouts = [global_layout, lights_layout, material_layout];
        let push_constant = vk::PushConstantRange::default()
            .stage_flags(vk::ShaderStageFlags::VERTEX | vk::ShaderStageFlags::FRAGMENT)
            .offset(0)
            .size(size_of::<ModelPushConstants>() as u32);
        let pipeline_layout_info = vk::PipelineLayoutCreateInfo::default()
            .set_layouts(&set_layouts)
            .push_constant_ranges(std::slice::from_ref(&push_constant));
        let pipeline_layout = unsafe {
            context
                .device()
                .create_pipeline_layout(&pipeline_layout_info, None)
                .map_err(|result| EngineError::vk("creating pipeline layout", result))?
        };

        let vertex_binding = [vk::VertexInputBindingDescription::default()
            .binding(0)
            .stride(size_of::<MeshVertex>() as u32)
            .input_rate(vk::VertexInputRate::VERTEX)];
        let vertex_attributes = [
            vk::VertexInputAttributeDescription::default()
                .binding(0)
                .location(0)
                .format(vk::Format::R32G32B32_SFLOAT)
                .offset(offset_of!(MeshVertex, position) as u32),
            vk::VertexInputAttributeDescription::default()
                .binding(0)
                .location(1)
                .format(vk::Format::R32G32B32_SFLOAT)
                .offset(offset_of!(MeshVertex, color) as u32),
            vk::VertexInputAttributeDescription::default()
                .binding(0)
                .location(2)
                .format(vk::Format::R32G32B32_SFLOAT)
                .offset(offset_of!(MeshVertex, normal) as u32),
            vk::VertexInputAttributeDescription::default()
                .binding(0)
                .location(3)
                .format(vk::Format::R32G32_SFLOAT)
                .offset(offset_of!(MeshVertex, uv) as u32),
        ];
        let vertex_input = vk::PipelineVertexInputStateCreateInfo::default()
            .vertex_binding_descriptions(&vertex_binding)
            .vertex_attribute_descriptions(&vertex_attributes);
        let input_assembly = vk::PipelineInputAssemblyStateCreateInfo::default()
            .topology(vk::PrimitiveTopology::TRIANGLE_LIST);
        let viewport_state = vk::PipelineViewportStateCreateInfo::default()
            .viewport_count(1)
            .scissor_count(1);
        let rasterizer = vk::PipelineRasterizationStateCreateInfo::default()
            .line_width(1.0)
            .cull_mode(vk::CullModeFlags::BACK)
            .front_face(vk::FrontFace::COUNTER_CLOCKWISE);
        let multisampling = vk::PipelineMultisampleStateCreateInfo::default()
            .rasterization_samples(vk::SampleCountFlags::TYPE_1);
        let depth_stencil = vk::PipelineDepthStencilStateCreateInfo::default()
            .depth_test_enable(true)
            .depth_write_enable(true)
            .depth_compare_op(vk::CompareOp::LESS);
        let color_blend_attachment = [vk::PipelineColorBlendAttachmentState::default()
            .color_write_mask(vk::ColorComponentFlags::RGBA)
            .blend_enable(false)];
        let color_blend =
            vk::PipelineColorBlendStateCreateInfo::default().attachments(&color_blend_attachment);
        let dynamic_states = [vk::DynamicState::VIEWPORT, vk::DynamicState::SCISSOR];
        let dynamic_state =
            vk::PipelineDynamicStateCreateInfo::default().dynamic_states(&dynamic_states);

        let pipeline_info = vk::GraphicsPipelineCreateInfo::default()
            .stages(&shader_stages)
            .vertex_input_state(&vertex_input)
            .input_assembly_state(&input_assembly)
            .viewport_state(&viewport_state)
            .rasterization_state(&rasterizer)
            .multisample_state(&multisampling)
            .depth_stencil_state(&depth_stencil)
            .color_blend_state(&color_blend)
            .dynamic_state(&dynamic_state)
            .layout(pipeline_layout)
            .render_pass(render_pass)
            .subpass(0);
        let pipeline = unsafe {
            context
                .device()
                .create_graphics_pipelines(
                    vk::PipelineCache::null(),
                    std::slice::from_ref(&pipeline_info),
                    None,
                )
                .map_err(|(_, result)| EngineError::vk("creating graphics pipeline", result))?[0]
        };

        unsafe {
            context.device().destroy_shader_module(vertex_module, None);
            context
                .device()
                .destroy_shader_module(fragment_module, None);
        }

        let pool_sizes = [vk::DescriptorPoolSize::default()
            .ty(vk::DescriptorType::UNIFORM_BUFFER)
            .descriptor_count((MAX_FRAMES_IN_FLIGHT * 4) as u32)];
        let scene_descriptor_pool_info = vk::DescriptorPoolCreateInfo::default()
            .pool_sizes(&pool_sizes)
            .max_sets((MAX_FRAMES_IN_FLIGHT * 2) as u32);
        let scene_descriptor_pool = unsafe {
            context
                .device()
                .create_descriptor_pool(&scene_descriptor_pool_info, None)
                .map_err(|result| EngineError::vk("creating scene descriptor pool", result))?
        };

        Ok(Self {
            context,
            render_pass,
            pipeline_layout,
            pipeline,
            global_layout,
            lights_layout,
            material_layout,
            scene_descriptor_pool,
            framebuffers: Vec::new(),
        })
    }

    pub(super) fn material_layout(&self) -> vk::DescriptorSetLayout {
        self.material_layout
    }

    pub(super) fn rebuild_framebuffers(&mut self, context: &VContext) -> Result<()> {
        self.destroy_framebuffers();

        let (swapchain_image_views, depth_view, extent) = context.framebuffer_views();
        for image_view in &swapchain_image_views {
            let attachments = [*image_view, depth_view];
            let framebuffer_info = vk::FramebufferCreateInfo::default()
                .render_pass(self.render_pass)
                .attachments(&attachments)
                .width(extent.width)
                .height(extent.height)
                .layers(1);
            let framebuffer = unsafe {
                self.context
                    .device()
                    .create_framebuffer(&framebuffer_info, None)
                    .map_err(|result| EngineError::vk("creating framebuffer", result))?
            };
            self.framebuffers.push(framebuffer);
        }

        Ok(())
    }

    pub(super) fn begin(
        &self,
        command_buffer: vk::CommandBuffer,
        framebuffer: vk::Framebuffer,
        extent: vk::Extent2D,
        scene_viewport: SceneViewport,
    ) {
        let scene_viewport = scene_viewport.clamped(extent.width, extent.height);
        let clear_values = [
            vk::ClearValue {
                color: vk::ClearColorValue {
                    float32: [0.18, 0.22, 0.28, 1.0],
                },
            },
            vk::ClearValue {
                depth_stencil: vk::ClearDepthStencilValue {
                    depth: 1.0,
                    stencil: 0,
                },
            },
        ];
        let render_area = vk::Rect2D {
            offset: vk::Offset2D { x: 0, y: 0 },
            extent,
        };
        let render_pass_info = vk::RenderPassBeginInfo::default()
            .render_pass(self.render_pass)
            .framebuffer(framebuffer)
            .render_area(render_area)
            .clear_values(&clear_values);

        unsafe {
            self.context.device().cmd_begin_render_pass(
                command_buffer,
                &render_pass_info,
                vk::SubpassContents::INLINE,
            );
            let viewport = vk::Viewport::default()
                .x(scene_viewport.offset_x as f32)
                .y(scene_viewport.offset_y as f32)
                .width(scene_viewport.width as f32)
                .height(scene_viewport.height as f32)
                .min_depth(0.0)
                .max_depth(1.0);
            self.context.device().cmd_set_viewport(
                command_buffer,
                0,
                std::slice::from_ref(&viewport),
            );
            let scene_scissor = vk::Rect2D {
                offset: vk::Offset2D {
                    x: scene_viewport.offset_x as i32,
                    y: scene_viewport.offset_y as i32,
                },
                extent: vk::Extent2D {
                    width: scene_viewport.width,
                    height: scene_viewport.height,
                },
            };
            self.context.device().cmd_set_scissor(
                command_buffer,
                0,
                std::slice::from_ref(&scene_scissor),
            );
            self.context.device().cmd_bind_pipeline(
                command_buffer,
                vk::PipelineBindPoint::GRAPHICS,
                self.pipeline,
            );
        }
    }

    pub(super) fn end(&self, command_buffer: vk::CommandBuffer) {
        unsafe {
            self.context.device().cmd_end_render_pass(command_buffer);
        }
    }

    fn destroy_framebuffers(&mut self) {
        unsafe {
            for framebuffer in self.framebuffers.drain(..) {
                self.context.device().destroy_framebuffer(framebuffer, None);
            }
        }
    }
}

impl Drop for GeometryPass {
    fn drop(&mut self) {
        unsafe {
            self.destroy_framebuffers();
            self.context
                .device()
                .destroy_descriptor_pool(self.scene_descriptor_pool, None);
            self.context.device().destroy_pipeline(self.pipeline, None);
            self.context
                .device()
                .destroy_pipeline_layout(self.pipeline_layout, None);
            self.context
                .device()
                .destroy_descriptor_set_layout(self.material_layout, None);
            self.context
                .device()
                .destroy_descriptor_set_layout(self.lights_layout, None);
            self.context
                .device()
                .destroy_descriptor_set_layout(self.global_layout, None);
            self.context
                .device()
                .destroy_render_pass(self.render_pass, None);
        }
    }
}
