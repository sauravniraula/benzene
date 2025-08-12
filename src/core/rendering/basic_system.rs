use ash::vk;

use crate::core::{
    backend_event::VBackendEvent,
    descriptor::VDescriptorLayout,
    device::VDevice,
    pipeline::{VPipelineInfo, VPipelineInfoConfig},
    rendering::{VRenderingSystem, VRenderingSystemConfig},
    swapchain::VSwapchain,
    vertex_input::{BindableVertexInput, Vertex2D},
};

pub struct BasicRenderingSystem {
    v_rendering_system: VRenderingSystem,
    pub descriptor_layouts: Vec<VDescriptorLayout>,
}

impl BasicRenderingSystem {
    pub fn new(v_device: &VDevice, v_swapchain: &VSwapchain) -> Self {
        let vertex_binding_descriptions = Vertex2D::get_binding_descriptions();
        let vertex_attribute_descriptions = Vertex2D::get_attribute_descriptions();
        let descriptor_layouts = vec![VDescriptorLayout::new(v_device)];

        let v_pipeline_info = VPipelineInfo::new(
            &v_device,
            VPipelineInfoConfig {
                binding_descriptions: vertex_binding_descriptions,
                attribute_descriptions: vertex_attribute_descriptions,
                vertex_shader_file: "src/shaders/shader.vert".into(),
                fragment_shader_file: "src/shaders/shader.frag".into(),
                descriptor_layouts: &descriptor_layouts,
            },
        );

        let v_rendering_system = VRenderingSystem::new(
            v_device,
            v_swapchain,
            VRenderingSystemConfig {
                pipeline_infos: vec![v_pipeline_info],
            },
        );

        Self {
            v_rendering_system,
            descriptor_layouts,
        }
    }

    pub fn render(
        &self,
        v_device: &VDevice,
        command_buffer: vk::CommandBuffer,
        image_index: usize,
        vertex_buffers: &[vk::Buffer],
        index_buffer: vk::Buffer,
        indices_count: u32,
    ) {
        self.v_rendering_system
            .start(v_device, command_buffer, image_index);

        unsafe {
            v_device
                .device
                .cmd_bind_vertex_buffers(command_buffer, 0, &vertex_buffers, &[0]);

            v_device.device.cmd_bind_index_buffer(
                command_buffer,
                index_buffer,
                0,
                vk::IndexType::UINT32,
            );

            v_device
                .device
                .cmd_draw_indexed(command_buffer, indices_count, 1, 0, 0, 0);
        }

        self.v_rendering_system.end(v_device, command_buffer);
    }

    pub fn handle_backend_event(&mut self, event: &VBackendEvent) {
        self.v_rendering_system.handle_backend_event(event);
    }

    pub fn destroy(&self, v_device: &VDevice) {
        self.v_rendering_system.destroy(v_device);
    }
}
