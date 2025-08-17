use ash::vk;

use crate::vulkan_backend::{
    backend::VBackend,
    backend_event::VBackendEvent,
    descriptor::{VDescriptorLayout, VDescriptorPool, VDescriptorSets},
    device::VDevice,
    pipeline::{VPipelineInfo, VPipelineInfoConfig},
    rendering::{VRenderingSystem, VRenderingSystemConfig, info::VRenderInfo},
    vertex_input::{BindableVertexInput, Vertex3D},
};

use crate::{constants::MAX_FRAMES_IN_FLIGHT, vulkan_backend::rendering::Recordable};

pub struct SceneRender {
    v_rendering_system: VRenderingSystem,
    pub pipeline_infos: Vec<VPipelineInfo>,
    pub descriptor_layouts: Vec<VDescriptorLayout>,
    descriptor_pool: VDescriptorPool,
}

impl SceneRender {
    pub fn new(v_backend: &VBackend) -> Self {
        let vertex_binding_descriptions = Vertex3D::get_binding_descriptions();
        let vertex_attribute_descriptions = Vertex3D::get_attribute_descriptions();
        let descriptor_layouts = vec![VDescriptorLayout::new(&v_backend.v_device)];

        let pipeline_infos = vec![VPipelineInfo::new(
            &v_backend.v_device,
            VPipelineInfoConfig {
                binding_descriptions: vertex_binding_descriptions,
                attribute_descriptions: vertex_attribute_descriptions,
                vertex_shader_file: "assets/shaders/shader.vert".into(),
                fragment_shader_file: "assets/shaders/shader.frag".into(),
            },
            &descriptor_layouts,
        )];

        let v_rendering_system = VRenderingSystem::new(
            &v_backend.v_device,
            &v_backend.v_swapchain,
            VRenderingSystemConfig {
                pipeline_infos: &pipeline_infos,
            },
        );

        let descriptor_pool = VDescriptorPool::new(&v_backend.v_device, MAX_FRAMES_IN_FLIGHT);

        Self { v_rendering_system, pipeline_infos, descriptor_layouts, descriptor_pool }
    }

    pub fn get_descriptor_set_layout_at_binding(&self, binding: usize) -> &VDescriptorLayout {
        &self.descriptor_layouts[binding]
    }

    pub fn allocate_descriptor_sets(&self, v_device: &VDevice, layout: &VDescriptorLayout, count: usize) -> VDescriptorSets {
        VDescriptorSets::new(v_device, &self.descriptor_pool, layout, count)
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
        self.descriptor_pool.destroy(v_device);
        self.v_rendering_system.destroy(v_device);
    }
}
