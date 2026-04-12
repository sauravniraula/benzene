use std::sync::Arc;

use ash::vk;
use egui_ash_renderer::{Options as EguiOptions, Renderer as EguiRenderer};

use crate::{
    constants::MAX_FRAMES_IN_FLIGHT, error::Result, render::vulkan::VContext, ui::EguiFrame,
};

use super::is_srgb_format;

pub(super) struct EguiPass {
    renderer: EguiRenderer,
    context: Arc<VContext>,
    pending_texture_frees: Vec<Vec<egui::TextureId>>,
}

impl EguiPass {
    pub(super) fn new(
        context: Arc<VContext>,
        render_pass: vk::RenderPass,
        color_format: vk::Format,
    ) -> Result<Self> {
        let options = EguiOptions {
            in_flight_frames: MAX_FRAMES_IN_FLIGHT,
            enable_depth_test: false,
            enable_depth_write: false,
            srgb_framebuffer: is_srgb_format(color_format),
        };
        let renderer = EguiRenderer::with_default_allocator(
            &context.instance,
            context.physical_device,
            context.device.clone(),
            render_pass,
            options,
        )?;
        let pending_texture_frees = vec![Vec::new(); MAX_FRAMES_IN_FLIGHT];

        Ok(Self {
            renderer,
            context,
            pending_texture_frees,
        })
    }

    pub(super) fn draw(
        &mut self,
        frame_index: usize,
        command_buffer: vk::CommandBuffer,
        extent: vk::Extent2D,
        frame: &EguiFrame,
    ) -> Result<()> {
        if !self.pending_texture_frees[frame_index].is_empty() {
            self.renderer
                .free_textures(&self.pending_texture_frees[frame_index])?;
            self.pending_texture_frees[frame_index].clear();
        }

        self.renderer.set_textures(
            self.context.graphics_queue,
            self.context.upload_command_pool(),
            frame.textures_delta.set.as_slice(),
        )?;
        self.renderer.cmd_draw(
            command_buffer,
            extent,
            frame.pixels_per_point,
            frame.primitives.as_slice(),
        )?;
        self.pending_texture_frees[frame_index] = frame.textures_delta.free.clone();

        Ok(())
    }
}
