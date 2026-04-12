use egui::{ClippedPrimitive, Color32, Context, Rect, TexturesDelta, ViewportId};
use egui_winit::EventResponse;
use winit::{event::WindowEvent, window::Window};

#[derive(Clone, Copy, Debug)]
pub struct SceneViewport {
    pub offset_x: u32,
    pub offset_y: u32,
    pub width: u32,
    pub height: u32,
}

impl SceneViewport {
    pub fn full(width: u32, height: u32) -> Self {
        Self {
            offset_x: 0,
            offset_y: 0,
            width,
            height,
        }
    }

    fn from_points(rect: Rect, pixels_per_point: f32) -> Self {
        let pixels_per_point = pixels_per_point.max(f32::EPSILON);
        let min_x = (rect.min.x * pixels_per_point).round().max(0.0) as u32;
        let min_y = (rect.min.y * pixels_per_point).round().max(0.0) as u32;
        let max_x = (rect.max.x * pixels_per_point).round().max(0.0) as u32;
        let max_y = (rect.max.y * pixels_per_point).round().max(0.0) as u32;

        Self {
            offset_x: min_x,
            offset_y: min_y,
            width: max_x.saturating_sub(min_x).max(1),
            height: max_y.saturating_sub(min_y).max(1),
        }
    }

    pub(crate) fn clamped(self, width: u32, height: u32) -> Self {
        if width == 0 || height == 0 {
            return Self::full(0, 0);
        }

        let offset_x = self.offset_x.min(width.saturating_sub(1));
        let offset_y = self.offset_y.min(height.saturating_sub(1));
        let max_width = width.saturating_sub(offset_x).max(1);
        let max_height = height.saturating_sub(offset_y).max(1);

        Self {
            offset_x,
            offset_y,
            width: self.width.clamp(1, max_width),
            height: self.height.clamp(1, max_height),
        }
    }
}

pub struct EguiFrame {
    pub(crate) primitives: Vec<ClippedPrimitive>,
    pub(crate) textures_delta: TexturesDelta,
    pub(crate) pixels_per_point: f32,
    pub(crate) scene_viewport: SceneViewport,
}

pub struct EguiLayer {
    context: Context,
    state: egui_winit::State,
}

impl EguiLayer {
    pub fn new(window: &Window) -> Self {
        let context = Context::default();
        let mut visuals = egui::Visuals::dark();
        visuals.window_fill = Color32::from_rgba_unmultiplied(18, 22, 24, 232);
        visuals.panel_fill = Color32::from_rgba_unmultiplied(12, 15, 16, 220);
        visuals.widgets.noninteractive.bg_fill = Color32::from_rgba_unmultiplied(32, 39, 42, 220);
        context.set_visuals(visuals);

        let state = egui_winit::State::new(
            context.clone(),
            ViewportId::ROOT,
            window,
            Some(window.scale_factor() as f32),
            window.theme(),
            None,
        );

        Self { context, state }
    }

    pub fn context(&self) -> &Context {
        &self.context
    }

    pub fn handle_window_event(&mut self, window: &Window, event: &WindowEvent) -> EventResponse {
        self.state.on_window_event(window, event)
    }

    pub fn run(
        &mut self,
        window: &Window,
        mut build_ui: impl FnMut(&Context) -> Rect,
    ) -> EguiFrame {
        let input = self.state.take_egui_input(window);
        let mut scene_rect = Rect::ZERO;
        let output = self.context.run(input, |context| {
            scene_rect = build_ui(context);
        });
        self.state
            .handle_platform_output(window, output.platform_output);

        let pixels_per_point = output.pixels_per_point;
        let primitives = self.context.tessellate(output.shapes, pixels_per_point);
        let scene_viewport = SceneViewport::from_points(scene_rect, pixels_per_point);

        EguiFrame {
            primitives,
            textures_delta: output.textures_delta,
            pixels_per_point,
            scene_viewport,
        }
    }
}
