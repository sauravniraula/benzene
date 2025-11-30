pub mod config;
pub mod pipeline_info;

use ash::vk;
pub use config::VPipelineInfoConfig;
pub use pipeline_info::VPipelineInfo;

use crate::vulkan_backend::device::VDevice;

pub fn create_pipelines_from_infos(
    v_device: &VDevice,
    render_pass: vk::RenderPass,
    infos: &Vec<VPipelineInfo>,
) -> Vec<vk::Pipeline> {
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

    for info in infos {
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

    for i in 0..infos.len() {
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
            .layout(infos[i].layout);
        pipeline_create_infos.push(pipeline_create_info);
    }

    unsafe {
        v_device
            .device
            .create_graphics_pipelines(vk::PipelineCache::null(), &pipeline_create_infos, None)
            .expect("failed to create pipelines")
    }
}
