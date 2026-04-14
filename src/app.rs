use std::ffi::CStr;

use crate::{log, log_info, vcontext::Vcontext};
use ash_window;
use winit::{
    self,
    raw_window_handle::{HasDisplayHandle, HasWindowHandle},
};

pub struct App {
    window: Option<winit::window::Window>,
    vcontext: Option<Vcontext>,
}

impl App {
    pub fn new() -> Self {
        let event_loop = winit::event_loop::EventLoop::new().expect("failed to create event loop");
        let mut app = Self {
            window: None,
            vcontext: None,
        };
        event_loop
            .run_app(&mut app)
            .expect("failed to run event loop");

        app
    }
}

impl winit::application::ApplicationHandler for App {
    fn resumed(&mut self, event_loop: &winit::event_loop::ActiveEventLoop) {
        let window = event_loop
            .create_window(winit::window::Window::default_attributes())
            .expect("failed to create window");

        let display_handle = event_loop.display_handle().unwrap();
        let raw_display_handle = display_handle.as_raw();
        let window_handle = window.window_handle().unwrap();

        let required_extensions: Vec<&CStr> =
            ash_window::enumerate_required_extensions(raw_display_handle)
                .expect("failed to get required window extensions")
                .iter()
                .map(|&each| unsafe { CStr::from_ptr(each) })
                .collect();
        log_info!("Required window extensions: {:?}", required_extensions);

        self.vcontext = Some(Vcontext::new(
            required_extensions,
            vec![],
            display_handle,
            window_handle,
        ));
        self.window = Some(window);
    }

    fn window_event(
        &mut self,
        event_loop: &winit::event_loop::ActiveEventLoop,
        window_id: winit::window::WindowId,
        event: winit::event::WindowEvent,
    ) {
        match event {
            winit::event::WindowEvent::CloseRequested => {
                event_loop.exit();
            }
            _ => (),
        }
    }
}
