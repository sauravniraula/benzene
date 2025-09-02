use ash::vk;

use crate::{
    core::{gpu::materials_manager::MaterialsManager, model_push_constant::ModelPushConstant},
    vulkan_backend::{
        backend::VBackend,
        backend_event::VBackendEvent,
        descriptor::{
            VDescriptorSetLayout,
            config::{VDescriptorBindingConfig, VDescriptorLayoutConfig},
        },
        device::VDevice,
        pipeline::{VPipelineInfo, VPipelineInfoConfig},
        push_constant::VPushConstant,
        rendering::{VRenderingSystem, VRenderingSystemConfig, info::VRenderInfo},
        vertex_input::{BindableVertexInput, Vertex3D},
    },
};
pub struct SceneRenderRecordContext<'a> {
    pub v_device: &'a VDevice,
    pub materials_manager: &'a MaterialsManager,
    pub cmd: vk::CommandBuffer,
    pub frame_index: usize,
    pub pipeline_infos: &'a Vec<VPipelineInfo>,
    pub descriptor_sets_layouts: &'a Vec<VDescriptorSetLayout>,
}

pub trait SceneRenderRecordable {
    fn record(&self, ctx: &SceneRenderRecordContext);
}

pub trait SceneRenderDrawable {
    fn draw(&self, v_device: &VDevice, command_buffer: vk::CommandBuffer);
}

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
        let lights_uniform_layout = VDescriptorSetLayout::new(
            &v_backend.v_device,
            VDescriptorLayoutConfig {
                bindings: vec![
                    VDescriptorBindingConfig {
                        binding: 0,
                        count: 1,
                        descriptor_type: vk::DescriptorType::UNIFORM_BUFFER,
                        shader_stage: vk::ShaderStageFlags::FRAGMENT,
                    },
                    VDescriptorBindingConfig {
                        binding: 1,
                        count: 1,
                        descriptor_type: vk::DescriptorType::UNIFORM_BUFFER,
                        shader_stage: vk::ShaderStageFlags::FRAGMENT,
                    },
                    VDescriptorBindingConfig {
                        binding: 2,
                        count: 1,
                        descriptor_type: vk::DescriptorType::UNIFORM_BUFFER,
                        shader_stage: vk::ShaderStageFlags::FRAGMENT,
                    },
                ],
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
        let descriptor_sets_layouts = vec![
            global_uniform_layout,
            lights_uniform_layout,
            image_sampler_layout,
        ];

        let pipeline_infos = vec![VPipelineInfo::new(
            &v_backend.v_device,
            VPipelineInfoConfig {
                binding_descriptions: vertex_binding_descriptions,
                attribute_descriptions: vertex_attribute_descriptions,
                vertex_shader_file: Some("assets/shaders/shader.vert".into()),
                fragment_shader_file: Some("assets/shaders/shader.frag".into()),
            },
            Some(model_push_constant),
            &descriptor_sets_layouts,
        )];

        let color_views: Vec<vk::ImageView> = v_backend
            .v_swapchain
            .v_image_views
            .iter()
            .map(|v| v.image_view)
            .collect();
        let depth_views: Vec<vk::ImageView> =
            vec![v_backend.v_swapchain.depth_v_image_view.image_view];

        let v_rendering_system = VRenderingSystem::new(
            &v_backend.v_device,
            VRenderingSystemConfig {
                pipeline_infos: &pipeline_infos,
                extent: v_backend.v_swapchain.image_extent,
                color_image_views: Some(&color_views),
                depth_image_views: Some(&depth_views),
                color_format: Some(v_backend.v_swapchain.v_images[0].config.format),
                depth_format: Some(v_backend.v_swapchain.depth_format),
                color_final_layout: Some(vk::ImageLayout::PRESENT_SRC_KHR),
                depth_final_layout: Some(vk::ImageLayout::DEPTH_STENCIL_ATTACHMENT_OPTIMAL),
            },
        );

        Self {
            v_rendering_system,
            pipeline_infos,
            descriptor_sets_layouts,
        }
    }

    pub fn render(
        &self,
        v_device: &VDevice,
        materials_manager: &MaterialsManager,
        info: &VRenderInfo,
        recordables: &[&dyn SceneRenderRecordable],
    ) {
        self.v_rendering_system.start(v_device, info);
        unsafe {
            v_device.device.cmd_bind_pipeline(
                info.command_buffer,
                vk::PipelineBindPoint::GRAPHICS,
                self.v_rendering_system.pipelines[0],
            )
        };

        let ctx = SceneRenderRecordContext {
            v_device,
            materials_manager: materials_manager,
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
