use vulkan_engine::core::{
    device::{VDevice, VPhysicalDevice, config::VPhysicalDeviceConfig},
    instance::{VInstance, VInstanceConfig},
    memory::{VBuffer, VBufferConfig},
    rendering::{VRenderer, basic_system::BasicRenderingSystem},
    surface::VSurface,
    swapchain::VSwapchain,
    vertex_input::Vertex,
    window::{VWindow, VWindowConfig},
};

fn main() {
    let v_window = VWindow::new(VWindowConfig::default());

    let v_instance = VInstance::new(&v_window, VInstanceConfig::default());

    let v_surface = VSurface::new(&v_window, &v_instance);

    let mut v_physical_devices = VPhysicalDevice::get_compatible_devices(
        &v_instance,
        &v_surface,
        VPhysicalDeviceConfig::default(),
    );
    let v_physical_device = v_physical_devices.remove(0);

    let v_device = VDevice::new(&v_instance, &v_surface, &v_physical_device);

    let v_swapchain = VSwapchain::new(
        &v_window,
        &v_instance,
        &v_surface,
        &v_physical_device,
        &v_device,
    );

    let v_renderer = VRenderer::new(&v_device, &v_swapchain);

    let vertices: [Vertex; 3] = [
        Vertex {
            pos: [-0.5, -0.5],
            color: [1.0, 0.0, 0.0],
        },
        Vertex {
            pos: [0.5, 0.5],
            color: [0.0, 1.0, 0.0],
        },
        Vertex {
            pos: [-0.5, 0.5],
            color: [0.0, 0.0, 1.0],
        },
    ];

    let vertices_buffer = VBuffer::new(
        &v_physical_device,
        &v_device,
        VBufferConfig {
            size: (size_of::<Vertex>() * vertices.len()) as u64,
        },
    );
    vertices_buffer.copy_to_buffer(&v_device, &vertices);

    let buffers = vec![vertices_buffer.buffer];

    let basic_rendering_system = BasicRenderingSystem::<Vertex>::new(&v_device, &v_swapchain);

    while !v_window.window.should_close() {
        v_renderer.render(|command_buffer, image_index| {
            basic_rendering_system.render(
                &v_device,
                command_buffer,
                image_index,
                vertices.len() as u32,
                &buffers,
            );
        });
    }
}
