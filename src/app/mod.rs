mod scene_setup;
mod ui;

use std::time::Instant;

use benzene::{EguiLayer, Engine, EngineError, InputState, Scene};
use winit::{
    application::ApplicationHandler,
    event::WindowEvent,
    event_loop::{ActiveEventLoop, EventLoop},
    window::{CursorGrabMode, Window, WindowId},
};

use self::{scene_setup::build_scene, ui::draw_debug_ui};

pub fn run() {
    let event_loop = EventLoop::new().expect("failed to create event loop");
    let mut app = SampleApp::default();
    event_loop.run_app(&mut app).expect("failed to run app");
}

#[derive(Default)]
struct SampleApp {
    window: Option<Window>,
    engine: Option<Engine>,
    scene: Option<Scene>,
    egui: Option<EguiLayer>,
    input: InputState,
    last_frame: Option<Instant>,
}

impl ApplicationHandler for SampleApp {
    fn resumed(&mut self, event_loop: &ActiveEventLoop) {
        let window = event_loop
            .create_window(Window::default_attributes().with_title("benzene"))
            .expect("failed to create window");

        let mut engine = Engine::new(&window).expect("failed to initialize engine");
        let scene = build_scene(&mut engine).expect("failed to build sample scene");

        self.window = Some(window);
        self.engine = Some(engine);
        self.scene = Some(scene);
        self.egui = self.window.as_ref().map(EguiLayer::new);
        self.last_frame = Some(Instant::now());
    }

    fn window_event(&mut self, event_loop: &ActiveEventLoop, _: WindowId, event: WindowEvent) {
        let Some(window) = self.window.as_ref() else {
            return;
        };

        let egui_consumed = self
            .egui
            .as_mut()
            .map(|egui| {
                let response = egui.handle_window_event(window, &event);
                if response.repaint {
                    window.request_redraw();
                }
                response.consumed
            })
            .unwrap_or(false);
        if !egui_consumed {
            self.input.handle_window_event(&event);
        }

        match event {
            WindowEvent::CloseRequested => event_loop.exit(),
            WindowEvent::Resized(size) => {
                if let Some(engine) = self.engine.as_mut() {
                    if size.width > 0 && size.height > 0 {
                        if let Err(error) = engine.resize(window) {
                            eprintln!("resize failed: {error}");
                            event_loop.exit();
                            return;
                        }
                    }
                }
            }
            WindowEvent::RedrawRequested => {
                let Some(engine) = self.engine.as_mut() else {
                    return;
                };
                let Some(scene) = self.scene.as_mut() else {
                    return;
                };

                let now = Instant::now();
                let dt = self
                    .last_frame
                    .replace(now)
                    .map(|last| now.duration_since(last).as_secs_f32())
                    .unwrap_or(0.0);

                let frame_input = self.input.frame_input();
                scene.update(dt, &frame_input);

                let egui_frame = self
                    .egui
                    .as_mut()
                    .map(|egui| egui.run(window, |context| draw_debug_ui(context, dt, scene)));
                let render_result = match egui_frame.as_ref() {
                    Some(egui_frame) => engine.render_with_egui(scene, egui_frame),
                    None => engine.render(scene),
                };

                match render_result {
                    Ok(()) | Err(EngineError::ZeroSizedWindow) => {}
                    Err(EngineError::SwapchainOutOfDate) => {
                        if let Err(error) = engine.resize(window) {
                            eprintln!("swapchain resize failed: {error}");
                            event_loop.exit();
                            return;
                        }
                    }
                    Err(error) => {
                        eprintln!("render failed: {error}");
                        event_loop.exit();
                        return;
                    }
                }

                window.request_redraw();
            }
            WindowEvent::MouseInput { .. } | WindowEvent::Focused(_) => {
                sync_cursor_capture(window, self.input.cursor_captured());
            }
            _ => {}
        }
    }
}

fn sync_cursor_capture(window: &Window, captured: bool) {
    window.set_cursor_visible(!captured);
    if captured {
        let _ = window
            .set_cursor_grab(CursorGrabMode::Locked)
            .or_else(|_| window.set_cursor_grab(CursorGrabMode::Confined));
    } else {
        let _ = window.set_cursor_grab(CursorGrabMode::None);
    }
}
