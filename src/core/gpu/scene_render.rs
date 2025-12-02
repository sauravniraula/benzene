use ash::vk::{self};

use crate::{
    core::gpu::{
        materials_manager::MaterialsManager,
        render_stage::geometry_and_lighting::{
            GeometryLightingRenderStage, GeometryLightingRenderStageConfig,
        },
    },
    vulkan_backend::{
        backend::VBackend, backend_event::VBackendEvent, descriptor::VDescriptorSetLayout,
        device::VDevice, frame::context::VFrameRenderContext, swapchain::VSwapchain,
    },
};
pub trait RecordableScene {
    fn record_scene(
        &self,
        v_device: &VDevice,
        cmd: vk::CommandBuffer,
        materials_m: &MaterialsManager,
        scene_r: &SceneRenderer,
    );
}

pub trait DrawableSceneElement {
    fn draw(&self, v_device: &VDevice, cmd: vk::CommandBuffer);
}

pub struct SceneRenderer {
    pub geometry_lighting_render_stage: GeometryLightingRenderStage,
    // shadow
    // pub v_shadow_rendering_system: VRenderingSystem,
    // shadow_pipeline_infos: Vec<VPipelineInfo>,
}

impl SceneRenderer {
    pub fn new(v_backend: &VBackend) -> Self {
        let geometry_lighting_render_stage = GeometryLightingRenderStage::new(
            &v_backend.v_device,
            GeometryLightingRenderStageConfig {
                color_format: v_backend.v_swapchain.v_images[0].config.format,
                depth_format: v_backend.v_swapchain.depth_format,
            },
        );

        let mut scene_renderer = Self {
            geometry_lighting_render_stage,
        };

        scene_renderer
            .init_geometry_lighting_render_stage(&v_backend.v_device, &v_backend.v_swapchain);

        scene_renderer
    }

    pub fn init_geometry_lighting_render_stage(
        &mut self,
        v_device: &VDevice,
        v_swapchain: &VSwapchain,
    ) {
        self.geometry_lighting_render_stage.init_framebuffers(
            v_device,
            &v_swapchain.image_ids,
            &v_swapchain.v_image_views,
            &v_swapchain.depth_v_image_view,
            v_swapchain.image_extent,
        );
    }

    pub fn get_pipeline_layout(&self) -> &vk::PipelineLayout {
        &self.geometry_lighting_render_stage.pipeline_infos[0].layout
    }

    pub fn get_global_uniform_layout(&self) -> &VDescriptorSetLayout {
        &self.geometry_lighting_render_stage.descriptor_set_layouts[0]
    }

    pub fn get_lights_uniform_layout(&self) -> &VDescriptorSetLayout {
        &self.geometry_lighting_render_stage.descriptor_set_layouts[1]
    }

    pub fn get_image_sampler_layout(&self) -> &VDescriptorSetLayout {
        &self.geometry_lighting_render_stage.descriptor_set_layouts[2]
    }

    pub fn handle_backend_event(&mut self, event: &VBackendEvent) {
        match event {
            VBackendEvent::UpdateFramebuffers(v_device, v_swapchain) => {
                self.init_geometry_lighting_render_stage(v_device, v_swapchain);
            }
            _ => {}
        }
    }

    pub fn render(
        &self,
        v_device: &VDevice,
        materials_manager: &MaterialsManager,
        ctx: &VFrameRenderContext,
        recordables: &[&dyn RecordableScene],
    ) {
        // Geometry Pass
        self.geometry_lighting_render_stage
            .start(v_device, ctx.cmd, &ctx.image_id);

        unsafe {
            v_device.device.cmd_bind_pipeline(
                ctx.cmd,
                vk::PipelineBindPoint::GRAPHICS,
                self.geometry_lighting_render_stage.pipelines[0],
            )
        };

        for recordable in recordables.iter() {
            recordable.record_scene(v_device, ctx.cmd, materials_manager, self);
        }

        self.geometry_lighting_render_stage.end(v_device, ctx);
    }

    pub fn destroy(&self, v_device: &VDevice) {
        // self.v_shadow_rendering_system.destroy(v_device);
        self.geometry_lighting_render_stage.destroy(v_device);
    }
}
