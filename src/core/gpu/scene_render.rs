use ash::vk;

use crate::{
    core::{
        gpu::recordable::{RecordContext, Recordable},
        model_push_constant::ModelPushConstant,
    },
    vulkan_backend::{
        backend::VBackend,
        backend_event::VBackendEvent,
        descriptor::{
            VDescriptorSetLayout,
            config::{
                VDescriptorBindingConfig, VDescriptorLayoutConfig,
            },
        },
        device::VDevice,
        pipeline::{VPipelineInfo, VPipelineInfoConfig},
        push_constant::VPushConstant,
        rendering::{VRenderingSystem, VRenderingSystemConfig, info::VRenderInfo},
        vertex_input::{BindableVertexInput, Vertex3D},
    },
};

pub struct SceneRender {
    v_rendering_system: VRenderingSystem,
    pipeline_infos: Vec<VPipelineInfo>,
    pub descriptor_sets_layouts: Vec<VDescriptorSetLayout>,
}

impl SceneRender {
    pub fn new(v_backend: &VBackend) -> Self {
        let vertex_binding_descriptions = Vertex3D::get_binding_descriptions();
        let vertex_attribute_descriptions = Vertex3D::get_attribute_descriptions();

        let model_push_constant = VPushConstant::new::<ModelPushConstant>(
            vk::ShaderStageFlags::VERTEX | vk::ShaderStageFlags::FRAGMENT,
        );

        // Creating Descriptor Layouts
        let global_uniform_layout = VDescriptorSetLayout::new(
            &v_backend.v_device,
            VDescriptorLayoutConfig {
                bindings: vec![VDescriptorBindingConfig {
                    binding: 0,
                    count: 1,
                    descriptor_type: vk::DescriptorType::UNIFORM_BUFFER,
                    shader_stage: vk::ShaderStageFlags::VERTEX | vk::ShaderStageFlags::FRAGMENT,
                }],
            },
        );
        let image_sampler_layout = VDescriptorSetLayout::new(
            &v_backend.v_device,
            VDescriptorLayoutConfig {
                bindings: vec![VDescriptorBindingConfig {
                    binding: 0,
                    count: 1,
                    descriptor_type: vk::DescriptorType::COMBINED_IMAGE_SAMPLER,
                    shader_stage: vk::ShaderStageFlags::FRAGMENT,
                }],
            },
        );
        let descriptor_sets_layouts = vec![global_uniform_layout, image_sampler_layout];

        let pipeline_infos = vec![VPipelineInfo::new(
            &v_backend.v_device,
            VPipelineInfoConfig {
                binding_descriptions: vertex_binding_descriptions,
                attribute_descriptions: vertex_attribute_descriptions,
                vertex_shader_file: "assets/shaders/shader.vert".into(),
                fragment_shader_file: "assets/shaders/shader.frag".into(),
            },
            Some(model_push_constant),
            &descriptor_sets_layouts,
        )];

        let v_rendering_system = VRenderingSystem::new(
            &v_backend.v_device,
            &v_backend.v_swapchain,
            VRenderingSystemConfig {
                pipeline_infos: &pipeline_infos,
            },
        );

        Self {
            v_rendering_system,
            pipeline_infos,
            descriptor_sets_layouts,
        }
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

        let ctx = RecordContext {
            v_device,
            cmd: info.command_buffer,
            frame_index: info.frame_index,
            pipeline_infos: &self.pipeline_infos,
            descriptor_sets_layouts: &self.descriptor_sets_layouts,
        };

        for recordable in recordables.iter() {
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
        for each in self.descriptor_sets_layouts.iter() {
            each.destroy(v_device);
        }
        self.v_rendering_system.destroy(v_device);
    }
}
