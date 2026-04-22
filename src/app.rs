use std::{ffi::CStr, sync::Arc};

use crate::{
    backend::{render_loop::RenderLoop, vcontext::Vcontext},
    log_info,
    render::{
        egui_integration::EguiIntegration,
        owner::RenderOwner,
    },
};
use ash_window;
use winit::{
    self,
    raw_window_handle::{HasDisplayHandle, HasWindowHandle},
};

pub struct App {
    window: Option<winit::window::Window>,
    vcontext: Option<Arc<Vcontext>>,
    render_loop: Option<RenderLoop>,
    render_owner: Option<RenderOwner>,
    egui_integration: Option<EguiIntegration>,
}

impl Drop for App {
    fn drop(&mut self) {
        self.egui_integration.take();
        self.render_owner.take();
        self.render_loop.take();
        self.vcontext.take();
        self.window.take();
    }
}

impl App {
    pub fn new() -> Self {
        let event_loop = winit::event_loop::EventLoop::new().expect("failed to create event loop");
        let mut app = Self {
            window: None,
            vcontext: None,
            render_loop: None,
            render_owner: None,
            egui_integration: None,
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

        let vcontext = Arc::new(Vcontext::new(
            required_extensions,
            vec![],
            display_handle,
            window_handle,
        ));

        self.render_owner = Some(RenderOwner::new(vcontext.clone()));
        self.render_loop = Some(RenderLoop::new(vcontext.clone()));
        self.egui_integration = Some(EguiIntegration::new(vcontext.clone(), &window));
        self.window = Some(window);
        self.vcontext = Some(vcontext);
    }

    fn window_event(
        &mut self,
        event_loop: &winit::event_loop::ActiveEventLoop,
        _window_id: winit::window::WindowId,
        event: winit::event::WindowEvent,
    ) {
        let window = self.window.as_ref().unwrap();
        let vcontext = self.vcontext.as_ref().unwrap();
        let render_loop = self.render_loop.as_mut().unwrap();
        let render_owner = self.render_owner.as_ref().unwrap();
        let egui_integration = self.egui_integration.as_mut().unwrap();

        match event {
            winit::event::WindowEvent::CloseRequested => {
                event_loop.exit();
            }
            winit::event::WindowEvent::RedrawRequested => {
                let success = render_loop.draw(|render_context| {
                    egui_integration.render(window);

                    render_owner.render(render_context);
                });
                if !success {
                    vcontext.recreate_swapchain();
                }
                self.window.as_ref().unwrap().request_redraw();
            }
            _ => (),
        }
    }
}
