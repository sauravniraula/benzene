use ash::vk;

use crate::vulkan_backend::{
    backend_event::VBackendEvent,
    descriptor::VDescriptorLayout,
    device::VDevice,
    pipeline::{VPipelineInfo, VPipelineInfoConfig},
    rendering::{VRenderingSystem, VRenderingSystemConfig, info::VRenderInfo},
    swapchain::VSwapchain,
    vertex_input::{BindableVertexInput, Vertex3D},
};

pub struct BasicRenderingSystem {
    v_rendering_system: VRenderingSystem,
    pub pipeline_infos: Vec<VPipelineInfo>,
    pub descriptor_layouts: Vec<VDescriptorLayout>,
}

use super::recordable::Recordable;

impl BasicRenderingSystem {
    pub fn new(v_device: &VDevice, v_swapchain: &VSwapchain) -> Self {
        let vertex_binding_descriptions = Vertex3D::get_binding_descriptions();
        let vertex_attribute_descriptions = Vertex3D::get_attribute_descriptions();
        let descriptor_layouts = vec![VDescriptorLayout::new(v_device)];

        let pipeline_infos = vec![VPipelineInfo::new(
            &v_device,
            VPipelineInfoConfig {
                binding_descriptions: vertex_binding_descriptions,
                attribute_descriptions: vertex_attribute_descriptions,
                vertex_shader_file: "src/shaders/shader.vert".into(),
                fragment_shader_file: "src/shaders/shader.frag".into(),
            },
            &descriptor_layouts,
        )];

        let v_rendering_system = VRenderingSystem::new(
            v_device,
            v_swapchain,
            VRenderingSystemConfig {
                pipeline_infos: &pipeline_infos,
            },
        );

        Self {
            v_rendering_system,
            pipeline_infos,
            descriptor_layouts,
        }
    }

    pub fn get_descriptor_set_layout_at_binding(&self, binding: usize) -> &VDescriptorLayout {
        &self.descriptor_layouts[binding]
    }

    pub fn render(&self, v_device: &VDevice, info: &VRenderInfo, recordables: &[&dyn Recordable]) {
        self.v_rendering_system.start(v_device, info);

        let pipeline_layouts: Vec<vk::PipelineLayout> =
            self.pipeline_infos.iter().map(|p| p.layout).collect();
        for recordable in recordables.iter() {
            recordable.record(
                v_device,
                info.command_buffer,
                info.frame_index,
                &pipeline_layouts,
            );
        }

        self.v_rendering_system.end(v_device, info);
    }

    pub fn handle_backend_event(&mut self, event: &VBackendEvent) {
        self.v_rendering_system.handle_backend_event(event);
    }

    pub fn destroy(&self, v_device: &VDevice) {
        for each in self.pipeline_infos.iter() {
            each.destroy(v_device);
        }
        for each in self.descriptor_layouts.iter() {
            each.destroy(v_device);
        }
        self.v_rendering_system.destroy(v_device);
    }
}
