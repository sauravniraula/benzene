use winit::{
    application::ApplicationHandler,
    dpi::PhysicalPosition,
    event::{ElementState, MouseButton, WindowEvent},
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

type BenzeneAppCallback<S> = dyn FnMut(&mut GameEngine, &mut S);

pub struct BenzeneApp<S> {
    window: Option<Window>,
    engine: Option<GameEngine>,
    on_init: Box<BenzeneAppCallback<S>>,
    on_new_frame: Box<BenzeneAppCallback<S>>,

    // State
    state: S,
    cursor_locked: bool,
}

impl<S> BenzeneApp<S> {
    pub fn new(
        state: S,
        on_init: Box<BenzeneAppCallback<S>>,
        on_new_frame: Box<BenzeneAppCallback<S>>,
    ) -> Self {
        let mut app = Self {
            window: None,
            engine: None,
            on_init,
            on_new_frame,
            state,
            cursor_locked: false,
        };
        let event_loop = EventLoop::new().expect("failed to create event loop");
        let _ = event_loop.run_app(&mut app);

        app
    }

    pub fn run() {}
}

impl<S> ApplicationHandler for BenzeneApp<S> {
    fn resumed(&mut self, event_loop: &event_loop::ActiveEventLoop) {
        let window = event_loop
            .create_window(Window::default_attributes())
            .expect("failed to create window");

        let mut engine = GameEngine::new(&window);

        (self.on_init)(&mut engine, &mut self.state);

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
            WindowEvent::MouseInput { state, button, .. } => {
                if let (ElementState::Pressed, MouseButton::Left) = (state, button) {
                    self.cursor_locked = true;
                }
                if let (ElementState::Released, MouseButton::Left) = (state, button) {
                    self.cursor_locked = false;
                }
            }
            WindowEvent::CursorMoved { position, .. } => {
                let cm_event = CursorMovedEvent::new(position.x, position.y);
                log!(format!("WindowEvent: CursorMoved - {:?}", cm_event));

                if self.cursor_locked {
                    engine.handle_cursor_moved(&cm_event);
                }
            }
            WindowEvent::RedrawRequested => {
                log!("---------------------------------");
                (self.on_new_frame)(engine, &mut self.state);
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
