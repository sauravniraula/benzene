use ash::vk;

use crate::{
    constants::MAX_FRAMES_IN_FLIGHT,
    core::{
        gpu::recordable::{RecordContext, Recordable},
        model_push_constant::ModelPushConstant,
    },
    vulkan_backend::{
        backend::VBackend,
        backend_event::VBackendEvent,
        descriptor::{
            VDescriptorPool, VDescriptorSetLayout, VDescriptorSets,
            config::{
                VDescriptorBindingConfig, VDescriptorLayoutConfig, VDescriptorPoolConfig,
                VDescriptorPoolSetConfig,
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
    descriptor_set_layouts: Vec<VDescriptorSetLayout>,
    descriptor_pool: VDescriptorPool,
}

impl SceneRender {
    pub fn new(v_backend: &VBackend) -> Self {
        let vertex_binding_descriptions = Vertex3D::get_binding_descriptions();
        let vertex_attribute_descriptions = Vertex3D::get_attribute_descriptions();

        let model_push_constant = VPushConstant::new::<ModelPushConstant>(
            vk::ShaderStageFlags::VERTEX | vk::ShaderStageFlags::FRAGMENT,
        );

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

        let descriptor_set_layouts = [global_uniform_layout, image_sampler_layout].into();

        let pipeline_infos = vec![VPipelineInfo::new(
            &v_backend.v_device,
            VPipelineInfoConfig {
                binding_descriptions: vertex_binding_descriptions,
                attribute_descriptions: vertex_attribute_descriptions,
                vertex_shader_file: "assets/shaders/shader.vert".into(),
                fragment_shader_file: "assets/shaders/shader.frag".into(),
            },
            Some(model_push_constant),
            &descriptor_set_layouts,
        )];

        let v_rendering_system = VRenderingSystem::new(
            &v_backend.v_device,
            &v_backend.v_swapchain,
            VRenderingSystemConfig {
                pipeline_infos: &pipeline_infos,
            },
        );

        let descriptor_pool_config = VDescriptorPoolConfig {
            sets: vec![
                VDescriptorPoolSetConfig {
                    layout: descriptor_set_layouts[0].config.clone(),
                    count: MAX_FRAMES_IN_FLIGHT,
                },
                VDescriptorPoolSetConfig {
                    layout: descriptor_set_layouts[1].config.clone(),
                    count: 1,
                },
            ],
        };
        let descriptor_pool = VDescriptorPool::new(&v_backend.v_device, &descriptor_pool_config);

        Self {
            v_rendering_system,
            pipeline_infos,
            descriptor_set_layouts,
            descriptor_pool,
        }
    }

    pub fn get_global_uniform_descriptor_set(&self, v_device: &VDevice) -> VDescriptorSets {
        VDescriptorSets::new(
            v_device,
            &self.descriptor_pool,
            &self.descriptor_set_layouts[0],
            MAX_FRAMES_IN_FLIGHT,
        )
    }

    pub fn get_image_sampler_descriptor_set(&self, v_device: &VDevice) -> VDescriptorSets {
        VDescriptorSets::new(
            v_device,
            &self.descriptor_pool,
            &self.descriptor_set_layouts[1],
            1,
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

        let ctx = RecordContext {
            v_device,
            cmd: info.command_buffer,
            frame_index: info.frame_index,
            pipeline_layout: self.pipeline_infos[0].layout,
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
        for each in self.descriptor_set_layouts.iter() {
            each.destroy(v_device);
        }
        self.descriptor_pool.destroy(v_device);
        self.v_rendering_system.destroy(v_device);
    }
}
