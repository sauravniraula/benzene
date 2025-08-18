use ash::vk;

use crate::vulkan_backend::{
    backend::VBackend,
    backend_event::VBackendEvent,
    descriptor::{VDescriptorLayout, VDescriptorPool, VDescriptorSets},
    device::VDevice,
    pipeline::{VPipelineInfo, VPipelineInfoConfig},
    rendering::{RecordContext, VRenderingSystem, VRenderingSystemConfig, info::VRenderInfo},
    vertex_input::{BindableVertexInput, Vertex3D},
};

use crate::{constants::MAX_FRAMES_IN_FLIGHT, vulkan_backend::rendering::Recordable};

pub struct SceneRender {
    v_rendering_system: VRenderingSystem,
    pipeline_infos: Vec<VPipelineInfo>,
    descriptor_layouts: Vec<VDescriptorLayout>,
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

        Self {
            v_rendering_system,
            pipeline_infos,
            descriptor_layouts,
            descriptor_pool,
        }
    }

    pub fn get_global_uniform_descriptor_set(&self, v_device: &VDevice) -> VDescriptorSets {
        VDescriptorSets::new(
            v_device,
            &self.descriptor_pool,
            &self.descriptor_layouts[0],
            MAX_FRAMES_IN_FLIGHT,
        )
    }

    pub fn render(&self, v_device: &VDevice, info: &VRenderInfo, recordables: &[&dyn Recordable]) {
        self.v_rendering_system.start(v_device, info);
        unsafe {
            v_device.device.cmd_bind_pipeline(
                info.command_buffer,
                vk::PipelineBindPoint::GRAPHICS,
                self.v_rendering_system.pipelines[0],
            )
        };

        for recordable in recordables.iter() {
            let ctx = RecordContext {
                v_device,
                cmd: info.command_buffer,
                frame_index: info.frame_index,
                pipeline_layout: self.pipeline_infos[0].layout,
            };
            recordable.record(&ctx);
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
