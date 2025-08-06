use super::VPipelineInfoConfig;
use crate::{core::device::VDevice, shared::load_file_as_vec_u32};
use ash::vk;

pub struct VPipelineInfo {
    pub config: VPipelineInfoConfig,
    pub vert_shader_module: vk::ShaderModule,
    pub frag_shader_module: vk::ShaderModule,
    pub layout: vk::PipelineLayout,
}

impl VPipelineInfo {
    pub fn new(v_device: &VDevice, config: VPipelineInfoConfig) -> Self {
        let mut vert_shader_byte_code_path = config.vertex_shader_file.clone();
        vert_shader_byte_code_path.push_str(".spv");
        let mut frag_shader_byte_code_path = config.fragment_shader_file.clone();
        frag_shader_byte_code_path.push_str(".spv");

        let vert_shader_code = load_file_as_vec_u32(&vert_shader_byte_code_path);
        let frag_shader_code = load_file_as_vec_u32(&frag_shader_byte_code_path);

        let vert_shader_module_create_info =
            vk::ShaderModuleCreateInfo::default().code(&vert_shader_code);
        let frag_shader_module_create_info =
            vk::ShaderModuleCreateInfo::default().code(&frag_shader_code);

        let vert_shader_module = unsafe {
            v_device
                .device
                .create_shader_module(&vert_shader_module_create_info, None)
                .expect("failed to create vertex shader module")
        };
        let frag_shader_module = unsafe {
            v_device
                .device
                .create_shader_module(&frag_shader_module_create_info, None)
                .expect("failed to create fragment shader module")
        };

        let layout_info = vk::PipelineLayoutCreateInfo::default();
        let layout = unsafe {
            v_device
                .device
                .create_pipeline_layout(&layout_info, None)
                .expect("failed to create pipeline layout")
        };

        Self {
            config,
            vert_shader_module,
            frag_shader_module,
            layout,
        }
    }

    pub fn get_vertex_input_stage(&self) -> vk::PipelineVertexInputStateCreateInfo {
        vk::PipelineVertexInputStateCreateInfo::default()
    }

    pub fn get_input_assembly_stage(&self) -> vk::PipelineInputAssemblyStateCreateInfo {
        vk::PipelineInputAssemblyStateCreateInfo::default()
            .topology(vk::PrimitiveTopology::TRIANGLE_LIST)
    }

    pub fn get_shader_stages(&self) -> Vec<vk::PipelineShaderStageCreateInfo> {
        vec![
            vk::PipelineShaderStageCreateInfo::default()
                .name(c"main")
                .module(self.vert_shader_module)
                .stage(vk::ShaderStageFlags::VERTEX),
            vk::PipelineShaderStageCreateInfo::default()
                .name(c"main")
                .module(self.frag_shader_module)
                .stage(vk::ShaderStageFlags::FRAGMENT),
        ]
    }

    pub fn get_rasterization_stage(&self) -> vk::PipelineRasterizationStateCreateInfo {
        vk::PipelineRasterizationStateCreateInfo::default().line_width(1.0)
    }

    pub fn get_dynamic_state(&self) -> vk::PipelineDynamicStateCreateInfo {
        vk::PipelineDynamicStateCreateInfo::default()
            .dynamic_states(&[vk::DynamicState::VIEWPORT, vk::DynamicState::SCISSOR])
    }

    pub fn get_viewport_state(&self) -> vk::PipelineViewportStateCreateInfo {
        vk::PipelineViewportStateCreateInfo::default()
            .viewport_count(1)
            .scissor_count(1)
    }
}
