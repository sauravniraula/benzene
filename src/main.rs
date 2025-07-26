use glfw::{Key, WindowEvent, WindowMode};
use vulkan_engine::entities::{VulkanApp, Window, WindowInstruction};

fn main() {
    let mut window = Window::new(1000, 600, "Hello Vulkan", WindowMode::Windowed);
    let app = VulkanApp::new(&window, 1, true);
    window.start(
        || {
            // println!("Looping");
            app.draw_frame();
        },
        |event| match event {
            WindowEvent::Key(key, _, __, ___) => {
                if key == Key::Escape {
                    return WindowInstruction::Close;
                }
                return WindowInstruction::None;
            }
            _ => WindowInstruction::None,
        },
    );
}
