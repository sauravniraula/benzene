use glfw::{Action, Key, WindowEvent};
use vulkan_engine::{
    app::VApp,
    core::{
        backend::VBackend,
        window::{VWindow, VWindowConfig},
    },
};

fn main() {
    let mut v_window = VWindow::new(VWindowConfig::default());
    let mut v_backend = VBackend::new(&v_window);

    // Apps
    let mut an_app = VApp::new(&v_backend);

    v_window.window.set_key_polling(true);
    while !v_window.window.should_close() {
        v_window.glfwi.poll_events();

        let window_messages: Vec<(f64, WindowEvent)> =
            glfw::flush_messages(&v_window.receiver).collect();

        for (_, event) in window_messages {
            match event {
                glfw::WindowEvent::Key(Key::Escape, _, Action::Press, _) => {
                    v_window.window.set_should_close(true);
                }
                _ => {}
            }
        }

        v_backend.render(&v_window, vec![&mut an_app]);
    }

    v_backend.v_device.wait_till_idle();
    an_app.destroy(&v_backend);
    v_backend.destroy();
}
