use std::sync::Arc;

use crate::backend::{
    file::{compiled_spirv_path_for_source, load_file_as_vec_u32},
    vcontext::Vcontext,
    vertex_3d::Vertex3D,
};

pub struct RenderGeometry {
    vcontext: Arc<Vcontext>,
    pub pipeline: ash::vk::Pipeline,
    pub pipeline_layout: ash::vk::PipelineLayout,
}

impl RenderGeometry {
    pub fn new(vcontext: Arc<Vcontext>) -> Self {
        let (pipeline, pipeline_layout) =
            RenderGeometry::create_graphics_pipeline(&vcontext.device, vcontext.surface_format);

        Self {
            vcontext,
            pipeline,
            pipeline_layout,
        }
    }

    fn create_graphics_pipeline(
        device: &ash::Device,
        surface_format: ash::vk::SurfaceFormatKHR,
    ) -> (ash::vk::Pipeline, ash::vk::PipelineLayout) {
        let vs_path = compiled_spirv_path_for_source("assets/shaders/test.vert");
        let fs_path = compiled_spirv_path_for_source("assets/shaders/test.frag");

        let vs_code = load_file_as_vec_u32(&vs_path);
        let fs_code = load_file_as_vec_u32(&fs_path);

        let vs_module = unsafe {
            device
                .create_shader_module(
                    &ash::vk::ShaderModuleCreateInfo::default().code(&vs_code),
                    None,
                )
                .expect("unable to create shader module")
        };
        let fs_module = unsafe {
            device
                .create_shader_module(
                    &ash::vk::ShaderModuleCreateInfo::default().code(&fs_code),
                    None,
                )
                .expect("unable to create shader module")
        };

        let vs_stage = ash::vk::PipelineShaderStageCreateInfo::default()
            .stage(ash::vk::ShaderStageFlags::VERTEX)
            .module(vs_module)
            .name(c"main");

        let fs_stage = ash::vk::PipelineShaderStageCreateInfo::default()
            .stage(ash::vk::ShaderStageFlags::FRAGMENT)
            .module(fs_module)
            .name(c"main");

        let shader_stages = [vs_stage, fs_stage];

        let dynamic_state = ash::vk::PipelineDynamicStateCreateInfo::default().dynamic_states(&[
            ash::vk::DynamicState::VIEWPORT,
            ash::vk::DynamicState::SCISSOR,
        ]);

        let vbd = Vertex3D::get_binding_descriptions();
        let vad = Vertex3D::get_attribute_descriptions();
        let vertex_input_state = ash::vk::PipelineVertexInputStateCreateInfo::default()
            .vertex_binding_descriptions(&vbd)
            .vertex_attribute_descriptions(&vad);

        let input_assembly_state = ash::vk::PipelineInputAssemblyStateCreateInfo::default()
            .topology(ash::vk::PrimitiveTopology::TRIANGLE_LIST);

        let viewport_state = ash::vk::PipelineViewportStateCreateInfo::default()
            .viewport_count(1)
            .scissor_count(1);

        let rasterization_state = ash::vk::PipelineRasterizationStateCreateInfo::default()
            .depth_clamp_enable(false)
            .rasterizer_discard_enable(false)
            .polygon_mode(ash::vk::PolygonMode::FILL)
            .cull_mode(ash::vk::CullModeFlags::BACK)
            .front_face(ash::vk::FrontFace::CLOCKWISE)
            .depth_bias_enable(false)
            .line_width(1_f32);

        let multisampling_state = ash::vk::PipelineMultisampleStateCreateInfo::default()
            .rasterization_samples(ash::vk::SampleCountFlags::TYPE_1)
            .sample_shading_enable(false);

        let color_blend_attachments = [ash::vk::PipelineColorBlendAttachmentState::default()
            .blend_enable(false)
            .color_write_mask(ash::vk::ColorComponentFlags::RGBA)];

        let color_blend_state = ash::vk::PipelineColorBlendStateCreateInfo::default()
            .logic_op_enable(false)
            .logic_op(ash::vk::LogicOp::COPY)
            .attachments(&color_blend_attachments);

        let pipeline_layout = unsafe {
            device
                .create_pipeline_layout(&ash::vk::PipelineLayoutCreateInfo::default(), None)
                .expect("unable to create pipeline layout")
        };

        let color_attachment_formats = [surface_format.format];
        let mut rendering_info = ash::vk::PipelineRenderingCreateInfo::default()
            .color_attachment_formats(&color_attachment_formats);

        let create_info = ash::vk::GraphicsPipelineCreateInfo::default()
            .push_next(&mut rendering_info)
            .stages(&shader_stages)
            .vertex_input_state(&vertex_input_state)
            .input_assembly_state(&input_assembly_state)
            .rasterization_state(&rasterization_state)
            .multisample_state(&multisampling_state)
            .color_blend_state(&color_blend_state)
            .dynamic_state(&dynamic_state)
            .viewport_state(&viewport_state)
            .layout(pipeline_layout);

        let pipelines = unsafe {
            device
                .create_graphics_pipelines(ash::vk::PipelineCache::null(), &[create_info], None)
                .expect("unable to create pipelines")
        };

        unsafe {
            device.destroy_shader_module(vs_module, None);
            device.destroy_shader_module(fs_module, None);
        }

        (pipelines[0], pipeline_layout)
    }
}

impl Drop for RenderGeometry {
    fn drop(&mut self) {
        let device = &self.vcontext.device;
        unsafe {
            let _ = device.device_wait_idle();
            device.destroy_pipeline_layout(self.pipeline_layout, None);
            device.destroy_pipeline(self.pipeline, None);
        }
    }
}
