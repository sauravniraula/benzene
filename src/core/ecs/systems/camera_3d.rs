use ash::vk::Extent2D;
use glfw::WindowEvent;
use nalgebra::Matrix4;

use crate::core::ecs::components::Camera3D;

pub fn camera_3d_handle_window_event(camera: &mut Camera3D, event: &WindowEvent) {
    // TODO: handle key presses
}

pub fn camera_3d_update(
    camera: &mut Camera3D,
    dt: f32,
    image_extent: Extent2D,
) -> (Matrix4<f32>, Matrix4<f32>) {
    // TODO: handle camera update
    (Matrix4::identity(), Matrix4::identity())
}
