use ash::vk::{self};

use crate::{
    core::gpu::{
        materials_manager::MaterialsManager,
        render_stage::geometry_and_lighting::{
            GeometryLightingRenderStage, GeometryLightingRenderStageConfig,
        },
    }, log, vulkan_backend::{
        backend::VBackend, backend_event::VBackendEvent, descriptor::VDescriptorSetLayout,
        device::VDevice, frame::context::VFrameRenderContext, swapchain::VSwapchain,
    }
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
    pub gl_rs: GeometryLightingRenderStage,
    // shadow
    // pub v_shadow_rendering_system: VRenderingSystem,
    // shadow_pipeline_infos: Vec<VPipelineInfo>,
}

impl SceneRenderer {
    pub fn new(v_backend: &VBackend) -> Self {
        let gl_rs = GeometryLightingRenderStage::new(
            &v_backend.v_device,
            GeometryLightingRenderStageConfig {
                color_format: v_backend.v_swapchain.v_images[0].config.format,
                depth_format: v_backend.v_swapchain.depth_format,
            },
        );

        let mut scene_renderer = Self {
            gl_rs,
        };

        scene_renderer
            .init_gl_rs(&v_backend.v_device, &v_backend.v_swapchain);

        scene_renderer
    }

    pub fn init_gl_rs(
        &mut self,
        v_device: &VDevice,
        v_swapchain: &VSwapchain,
    ) {
        self.gl_rs.init_framebuffers(
            v_device,
            &v_swapchain.image_ids,
            &v_swapchain.v_image_views,
            &v_swapchain.depth_v_image_view,
            v_swapchain.image_extent,
        );
    }

    pub fn get_pipeline_layout(&self) -> &vk::PipelineLayout {
        &self.gl_rs.pipeline_infos[0].layout
    }

    pub fn get_global_uniform_layout(&self) -> &VDescriptorSetLayout {
        &self.gl_rs.descriptor_set_layouts[0]
    }

    pub fn get_lights_uniform_layout(&self) -> &VDescriptorSetLayout {
        &self.gl_rs.descriptor_set_layouts[1]
    }

    pub fn get_image_sampler_layout(&self) -> &VDescriptorSetLayout {
        &self.gl_rs.descriptor_set_layouts[2]
    }

    pub fn handle_backend_event(&mut self, event: &VBackendEvent) {
        match event {
            VBackendEvent::UpdateFramebuffers(v_device, v_swapchain) => {
                self.init_gl_rs(v_device, v_swapchain);
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
        self.gl_rs
            .start(v_device, ctx.cmd, &ctx.image_id);

        unsafe {
            v_device.device.cmd_bind_pipeline(
                ctx.cmd,
                vk::PipelineBindPoint::GRAPHICS,
                self.gl_rs.pipelines[0],
            )
        };

        for recordable in recordables.iter() {
            recordable.record_scene(v_device, ctx.cmd, materials_manager, self);
        }

        self.gl_rs.end(v_device, ctx);
    }

    pub fn destroy(&self, v_device: &VDevice) {
        // self.v_shadow_rendering_system.destroy(v_device);
        self.gl_rs.destroy(v_device);
    }
}
