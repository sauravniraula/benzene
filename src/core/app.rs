use winit::{
    application::ApplicationHandler,
    event::WindowEvent,
    event_loop::{self, EventLoop},
    window::Window,
};

use crate::{
    core::{
        GameEngine,
        ecs::types::{CursorMovedEvent, KeyboardInputEvent},
    },
    log,
};

type BenzeneAppCallback = dyn FnMut(&mut GameEngine);

pub struct BenzeneApp {
    window: Option<Window>,
    engine: Option<GameEngine>,
    on_init: Box<BenzeneAppCallback>,
    on_new_frame: Box<BenzeneAppCallback>,

    // State
    state: Box<dyn std::any::Any>,
}

impl BenzeneApp {
    pub fn new<S: 'static>(
        initial_state: S,
        on_init: Box<BenzeneAppCallback>,
        on_new_frame: Box<BenzeneAppCallback>,
    ) -> Self {
        let mut app = Self {
            window: None,
            engine: None,
            on_init,
            on_new_frame,
            state: Box::new(initial_state),
        };
        let event_loop = EventLoop::new().expect("failed to create event loop");
        let _ = event_loop.run_app(&mut app);

        app
    }

    pub fn run() {}
}

impl ApplicationHandler for BenzeneApp {
    fn resumed(&mut self, event_loop: &event_loop::ActiveEventLoop) {
        let window = event_loop
            .create_window(Window::default_attributes())
            .expect("failed to create window");

        let mut engine = GameEngine::new(&window);

        (self.on_init)(&mut engine);

        self.engine = Some(engine);
        self.window = Some(window);
    }

    fn window_event(
        &mut self,
        event_loop: &event_loop::ActiveEventLoop,
        _: winit::window::WindowId,
        event: winit::event::WindowEvent,
    ) {
        let engine = self.engine.as_mut().expect("game engine not initialized");
        let window = self.window.as_ref().expect("window not initialized");

        match event {
            WindowEvent::Resized(_) => {
                log!("WindowEvent: Resized");
            }
            WindowEvent::KeyboardInput { event, .. } => {
                let ki_event =
                    KeyboardInputEvent::new(event.physical_key, event.state, event.repeat);
                log!(format!("WindowEvent: KeyboardInput - {:?}", ki_event));

                engine.handle_keyboard_input(&ki_event);
            }
            WindowEvent::CursorMoved { position, .. } => {
                let cm_event = CursorMovedEvent::new(position.x, position.y);
                log!(format!("WindowEvent: CursorMoved - {:?}", cm_event));
            }
            WindowEvent::RedrawRequested => {
                log!("---------------------------------");
                (self.on_new_frame)(engine);
                engine.pre_render();
                engine.render(window);

                self.window.as_ref().unwrap().request_redraw();
            }
            WindowEvent::CloseRequested => {
                log!("WindowEvent: CloseRequested");
                event_loop.exit();
            }
            _ => (),
        }
    }
}
