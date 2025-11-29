use ash::vk::{self};

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
pub trait SceneRenderRecordable {
    fn record_geometry(
        &self,
        v_device: &VDevice,
        materials_manager: &MaterialsManager,
        cmd: vk::CommandBuffer,
        frame_index: usize,
        pipeline_infos: &Vec<VPipelineInfo>,
        descriptor_sets_layouts: &Vec<VDescriptorSetLayout>,
    );
}

pub trait SceneRenderDrawable {
    fn draw(&self, v_device: &VDevice, command_buffer: vk::CommandBuffer);
}

pub struct SceneRender {
    pub v_rendering_system: VRenderingSystem,
    pipeline_infos: Vec<VPipelineInfo>,

    // shadow
    pub v_shadow_rendering_system: VRenderingSystem,
    shadow_pipeline_infos: Vec<VPipelineInfo>,

    pub descriptor_set_layouts: Vec<VDescriptorSetLayout>,
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

        let pipeline_infos = vec![VPipelineInfo::new(
            &v_backend.v_device,
            VPipelineInfoConfig {
                binding_descriptions: vertex_binding_descriptions.clone(),
                attribute_descriptions: vertex_attribute_descriptions.clone(),
                vertex_shader_file: Some("assets/shaders/shader.vert".into()),
                fragment_shader_file: Some("assets/shaders/shader.frag".into()),
            },
            Some(&model_push_constant),
            &[
                &global_uniform_layout,
                &lights_uniform_layout,
                &image_sampler_layout,
            ],
        )];

        let color_views = &v_backend.v_swapchain.v_image_views;
        let depth_view = &v_backend.v_swapchain.depth_v_image_view;

        let mut v_rendering_system = VRenderingSystem::new(
            &v_backend.v_device,
            VRenderingSystemConfig {
                pipeline_infos: &pipeline_infos,
                color_format: Some(v_backend.v_swapchain.v_images[0].config.format),
                depth_format: Some(v_backend.v_swapchain.depth_format),
                color_final_layout: Some(vk::ImageLayout::PRESENT_SRC_KHR),
                depth_final_layout: Some(vk::ImageLayout::DEPTH_STENCIL_ATTACHMENT_OPTIMAL),
                dynamic_viewport: true,
            },
        );
        for (i, id) in v_backend.v_swapchain.image_ids.iter().copied().enumerate() {
            v_rendering_system.add_framebuffer(
                &v_backend.v_device,
                id,
                Some(&color_views[i]),
                Some(depth_view),
                v_backend.v_swapchain.image_extent,
                i == 0,
                i == 0,
            );
        }

        let shadow_pipeline_infos = vec![VPipelineInfo::new(
            &v_backend.v_device,
            VPipelineInfoConfig {
                binding_descriptions: vertex_binding_descriptions,
                attribute_descriptions: vertex_attribute_descriptions,
                vertex_shader_file: Some("assets/shaders/shadow.vert".into()),
                fragment_shader_file: None,
            },
            Some(&model_push_constant),
            &[&global_uniform_layout],
        )];

        let v_shadow_rendering_system = VRenderingSystem::new(
            &v_backend.v_device,
            VRenderingSystemConfig {
                pipeline_infos: &shadow_pipeline_infos,
                color_format: None,
                depth_format: Some(
                    v_backend
                        .v_physical_device
                        .get_format_for_depth_stencil(&v_backend.v_instance),
                ),
                color_final_layout: None,
                depth_final_layout: Some(vk::ImageLayout::DEPTH_STENCIL_ATTACHMENT_OPTIMAL),
                dynamic_viewport: true,
            },
        );

        Self {
            v_rendering_system,
            pipeline_infos,
            v_shadow_rendering_system,
            shadow_pipeline_infos,
            descriptor_set_layouts: vec![
                global_uniform_layout,
                lights_uniform_layout,
                image_sampler_layout,
            ],
        }
    }

    pub fn handle_backend_event(&mut self, event: &VBackendEvent) {
        match event {
            VBackendEvent::UpdateFramebuffers(v_device, v_swapchain) => {
                self.v_rendering_system.remove_all_framebuffers(v_device);

                let color_views = &v_swapchain.v_image_views;
                let depth_view = &v_swapchain.depth_v_image_view;

                for (i, id) in v_swapchain.image_ids.iter().copied().enumerate() {
                    self.v_rendering_system.add_framebuffer(
                        v_device,
                        id,
                        Some(&color_views[i]),
                        Some(depth_view),
                        v_swapchain.image_extent,
                        i == 0,
                        i == 0,
                    );
                }
            }
            _ => {}
        }
    }

    pub fn render(
        &self,
        v_device: &VDevice,
        materials_manager: &MaterialsManager,
        info: &VRenderInfo,
        recordables: &[&dyn SceneRenderRecordable],
    ) {
        // Shadow Pass
        if !self.v_shadow_rendering_system.framebuffers.is_empty() {
            self.v_shadow_rendering_system
                .start(v_device, info.command_buffer, &info.image_id);

            self.v_shadow_rendering_system.end(v_device, info);
        }

        // Geometry Pass
        self.v_rendering_system
            .start(v_device, info.command_buffer, &info.image_id);

        unsafe {
            v_device.device.cmd_bind_pipeline(
                info.command_buffer,
                vk::PipelineBindPoint::GRAPHICS,
                self.v_rendering_system.pipelines[0],
            )
        };

        for recordable in recordables.iter() {
            recordable.record_geometry(
                v_device,
                materials_manager,
                info.command_buffer,
                info.frame_index,
                &self.pipeline_infos,
                &self.descriptor_set_layouts,
            );
        }

        self.v_rendering_system.end(v_device, info);
    }

    pub fn destroy(&self, v_device: &VDevice) {
        for each in self.shadow_pipeline_infos.iter() {
            each.destroy(v_device);
        }
        for each in self.pipeline_infos.iter() {
            each.destroy(v_device);
        }
        for each in self.descriptor_set_layouts.iter() {
            each.destroy(v_device);
        }
        self.v_shadow_rendering_system.destroy(v_device);
        self.v_rendering_system.destroy(v_device);
    }
}
