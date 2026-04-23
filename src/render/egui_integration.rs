use std::sync::Arc;

use egui;
use egui_ash_renderer::{DynamicRendering, Options, Renderer};
use egui_winit;
use winit;

use crate::{
    backend::{
        command_buffer::create_command_pool, render_loop::RenderContext, vcontext::Vcontext,
    },
    render::scene::Scene,
};

pub struct EguiIntegration {
    pub vcontext: Arc<Vcontext>,
    pub ctx: egui::Context,
    pub state: egui_winit::State,
    pub renderer: Renderer,
    pub textures_to_free: Vec<egui::TextureId>,
    pub command_pool: ash::vk::CommandPool,
    pub scene: Arc<Scene>,
}

impl Drop for EguiIntegration {
    fn drop(&mut self) {
        let device = &self.vcontext.device;
        unsafe {
            let _ = device.device_wait_idle();
            device.destroy_command_pool(self.command_pool, None);
        }
    }
}

impl EguiIntegration {
    pub fn new(vcontext: Arc<Vcontext>, window: &winit::window::Window, scene: Arc<Scene>) -> Self {
        let ctx = egui::Context::default();
        let state = egui_winit::State::new(
            ctx.clone(),
            egui::ViewportId::ROOT,
            window,
            None,
            None,
            None,
        );

        let mut options = Options::default();
        options.in_flight_frames = vcontext.state.borrow().image_count as usize;

        let renderer = Renderer::with_default_allocator(
            &vcontext.instance,
            vcontext.physical_device,
            vcontext.device.clone(),
            DynamicRendering {
                color_attachment_format: vcontext.surface_format.format,
                depth_attachment_format: None,
            },
            options,
        )
        .expect("unable to create egui renderer");

        let command_pool = create_command_pool(&vcontext.device, vcontext.graphics_queue_index);

        Self {
            vcontext,
            ctx,
            state,
            renderer,
            textures_to_free: vec![],
            command_pool,
            scene,
        }
    }

    pub fn render(&mut self, window: &winit::window::Window, render_context: &RenderContext) {
        let raw_input = self.state.take_egui_input(window);

        if !self.textures_to_free.is_empty() {
            self.renderer
                .free_textures(&self.textures_to_free)
                .expect("failed to free textures");
        }

        let output = self.build_ui(raw_input);

        self.state
            .handle_platform_output(window, output.platform_output);

        if !output.textures_delta.free.is_empty() {
            self.textures_to_free = output.textures_delta.free;
        }

        if !output.textures_delta.set.is_empty() {
            self.renderer
                .set_textures(
                    self.vcontext.graphics_queue,
                    self.command_pool,
                    &output.textures_delta.set,
                )
                .expect("unable to set textures")
        }

        let clipped_primitives = self.ctx.tessellate(output.shapes, output.pixels_per_point);
        let extent = self
            .vcontext
            .state
            .borrow()
            .surface_capabilities
            .current_extent;

        self.renderer
            .cmd_draw(
                render_context.cmd,
                extent,
                output.pixels_per_point,
                &clipped_primitives,
            )
            .expect("failed to record egui render command");
    }

    fn build_ui(&mut self, raw_input: egui::RawInput) -> egui::FullOutput {
        let frame_state = self.scene.frame_state.borrow_mut();

        self.ctx.run(raw_input, |_| {
            egui::SidePanel::right("right-side-panel")
                .default_width(200.0)
                .resizable(true)
                .show(&self.ctx, |ui| {
                    ui.take_available_width();
                    egui::ScrollArea::vertical().show(ui, |ui| {
                        ui.take_available_width();
                        ui.label(format!("FPS: {}", frame_state.fps));
                    });
                });
        })
    }
}
