use super::VPipelineInfoConfig;
use crate::{
    shared::load_file_as_vec_u32,
    utils::compiled_spirv_path_for_source,
    vulkan_backend::{
        descriptor::VDescriptorSetLayout, device::VDevice, push_constant::VPushConstant,
    },
};
use ash::vk;

pub struct VPipelineInfo {
    pub config: VPipelineInfoConfig,
    pub vert_shader_module: Option<vk::ShaderModule>,
    pub frag_shader_module: Option<vk::ShaderModule>,
    pub layout: vk::PipelineLayout,
    pub color_blend_attachments: Vec<vk::PipelineColorBlendAttachmentState>,
}

impl VPipelineInfo {
    pub fn new(
        v_device: &VDevice,
        config: VPipelineInfoConfig,
        v_push_constant: Option<&VPushConstant>,
        v_descriptor_set_layouts: &[&VDescriptorSetLayout],
    ) -> Self {
        let vert_shader_module: Option<vk::ShaderModule> = match &config.vertex_shader_file {
            Some(file) => {
                let vert_shader_byte_code_path = compiled_spirv_path_for_source(file);
                let vert_shader_code = load_file_as_vec_u32(&vert_shader_byte_code_path);
                let vert_shader_module_create_info =
                    vk::ShaderModuleCreateInfo::default().code(&vert_shader_code);
                Some(unsafe {
                    v_device
                        .device
                        .create_shader_module(&vert_shader_module_create_info, None)
                        .expect("failed to create vertex shader module")
                })
            }
            None => None,
        };

        let frag_shader_module: Option<vk::ShaderModule> = match &config.fragment_shader_file {
            Some(file) => {
                let frag_shader_byte_code_path = compiled_spirv_path_for_source(file);
                let frag_shader_code = load_file_as_vec_u32(&frag_shader_byte_code_path);
                let frag_shader_module_create_info =
                    vk::ShaderModuleCreateInfo::default().code(&frag_shader_code);
                Some(unsafe {
                    v_device
                        .device
                        .create_shader_module(&frag_shader_module_create_info, None)
                        .expect("failed to create fragment shader module")
                })
            }
            None => None,
        };

        let descriptor_set_layouts: Vec<vk::DescriptorSetLayout> = v_descriptor_set_layouts
            .iter()
            .map(|each| each.layout)
            .collect();

        let mut layout_info =
            vk::PipelineLayoutCreateInfo::default().set_layouts(&descriptor_set_layouts);

        let layout = unsafe {
            match v_push_constant {
                Some(pc) => {
                    layout_info =
                        layout_info.push_constant_ranges(std::slice::from_ref(&pc.push_constant));
                    v_device
                        .device
                        .create_pipeline_layout(&layout_info, None)
                        .expect("failed to create pipeline layout")
                }
                None => v_device
                    .device
                    .create_pipeline_layout(&layout_info, None)
                    .expect("failed to create pipeline layout"),
            }
        };

        let color_blend_attachments = vec![
            vk::PipelineColorBlendAttachmentState::default()
                .blend_enable(false)
                .color_write_mask(vk::ColorComponentFlags::RGBA),
        ];

        Self {
            config,
            vert_shader_module,
            frag_shader_module,
            layout,
            color_blend_attachments,
        }
    }

    pub fn get_vertex_input_state(&self) -> vk::PipelineVertexInputStateCreateInfo<'_> {
        vk::PipelineVertexInputStateCreateInfo::default()
            .vertex_binding_descriptions(&self.config.binding_descriptions)
            .vertex_attribute_descriptions(&self.config.attribute_descriptions)
    }

    pub fn get_input_assembly_state(&self) -> vk::PipelineInputAssemblyStateCreateInfo<'_> {
        vk::PipelineInputAssemblyStateCreateInfo::default()
            .topology(vk::PrimitiveTopology::TRIANGLE_LIST)
    }

    pub fn get_shader_states(&self) -> Vec<vk::PipelineShaderStageCreateInfo<'_>> {
        let mut stages: Vec<vk::PipelineShaderStageCreateInfo> = Vec::new();
        if let Some(module) = self.vert_shader_module {
            stages.push(
                vk::PipelineShaderStageCreateInfo::default()
                    .name(c"main")
                    .module(module)
                    .stage(vk::ShaderStageFlags::VERTEX),
            );
        }
        if let Some(module) = self.frag_shader_module {
            stages.push(
                vk::PipelineShaderStageCreateInfo::default()
                    .name(c"main")
                    .module(module)
                    .stage(vk::ShaderStageFlags::FRAGMENT),
            );
        }
        stages
    }

    pub fn get_rasterization_state(&self) -> vk::PipelineRasterizationStateCreateInfo<'_> {
        vk::PipelineRasterizationStateCreateInfo::default()
            .line_width(1.0)
            .cull_mode(vk::CullModeFlags::BACK)
            .front_face(vk::FrontFace::COUNTER_CLOCKWISE)
    }

    pub fn get_depth_stencil_state(&self) -> vk::PipelineDepthStencilStateCreateInfo<'_> {
        vk::PipelineDepthStencilStateCreateInfo::default()
            .depth_test_enable(true)
            .depth_write_enable(true)
            .depth_compare_op(vk::CompareOp::LESS)
            .depth_bounds_test_enable(false)
            .stencil_test_enable(false)
    }

    pub fn get_multisampling_state(&self) -> vk::PipelineMultisampleStateCreateInfo<'_> {
        vk::PipelineMultisampleStateCreateInfo::default()
            .sample_shading_enable(false)
            .rasterization_samples(vk::SampleCountFlags::TYPE_1)
    }

    pub fn get_color_blend_state(&self) -> vk::PipelineColorBlendStateCreateInfo<'_> {
        vk::PipelineColorBlendStateCreateInfo::default()
            .attachments(&self.color_blend_attachments)
            .logic_op_enable(false)
    }

    pub fn get_dynamic_state(&self) -> vk::PipelineDynamicStateCreateInfo<'_> {
        vk::PipelineDynamicStateCreateInfo::default()
            .dynamic_states(&[vk::DynamicState::VIEWPORT, vk::DynamicState::SCISSOR])
    }

    pub fn get_viewport_state(&self) -> vk::PipelineViewportStateCreateInfo<'_> {
        vk::PipelineViewportStateCreateInfo::default()
            .viewport_count(1)
            .scissor_count(1)
    }

    pub fn destroy(&self, v_device: &VDevice) {
        unsafe {
            if let Some(module) = self.vert_shader_module {
                v_device.device.destroy_shader_module(module, None);
            }
            if let Some(module) = self.frag_shader_module {
                v_device.device.destroy_shader_module(module, None);
            }
            v_device.device.destroy_pipeline_layout(self.layout, None);
        }
    }
}
