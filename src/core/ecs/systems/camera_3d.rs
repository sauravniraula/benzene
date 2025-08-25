use ash::vk::Extent2D;
use glfw::{Action, Key, WindowEvent};
use nalgebra::{Matrix4, Perspective3, Translation3, UnitQuaternion, Vector3};

use crate::core::ecs::components::Camera3D;

pub fn camera_3d_handle_window_event(camera: &mut Camera3D, event: &WindowEvent) {
    match event {
        WindowEvent::Key(key, _scancode, action, _mods) => {
            let is_relevant = matches!(
                key,
                Key::A
                    | Key::S
                    | Key::D
                    | Key::W
                    | Key::Left
                    | Key::Right
                    | Key::Up
                    | Key::Down
                    | Key::Space
                    | Key::LeftAlt
            );
            if !is_relevant {
                return;
            }

            match action {
                Action::Press | Action::Repeat => {
                    camera.active_keys.insert(*key);
                }
                Action::Release => {
                    camera.active_keys.remove(key);
                }
            }
        }
        _ => (),
    }
}

pub fn camera_3d_compute_transform(camera: &mut Camera3D, dt: f32) {
    let mut position_delta = Vector3::new(0.0, 0.0, 0.0);
    let mut rotation_delta = Vector3::new(0.0, 0.0, 0.0);

    // Our euler storage: x=pitch, y=roll, z=yaw (existing usage)
    // For FPS-style controls use yaw around Y and pitch around X in world/local terms.
    // Here we continue to store in (x, y, z) but interpret forward/right from yaw about Y.
    let yaw = camera.transform.rotation.y;
    let (sin_yaw, cos_yaw) = yaw.sin_cos();
    let forward = Vector3::new(sin_yaw, 0.0, cos_yaw);
    let right = Vector3::new(cos_yaw, 0.0, -sin_yaw);
    let step = camera.speed * dt;
    let rot_step = camera.rotation_speed * dt;

    for key in camera.active_keys.iter() {
        match key {
            Key::W => position_delta -= forward * step,
            Key::S => position_delta += forward * step,
            Key::A => position_delta -= right * step,
            Key::D => position_delta += right * step,
            // Vertical movement: Space goes up (+Y), Alt goes down (-Y)
            Key::Space => position_delta.y += step,
            Key::LeftAlt => position_delta.y -= step,
            // Rotation with arrows: pitch (X) and yaw (Y)
            Key::Up => rotation_delta.x += rot_step,
            Key::Down => rotation_delta.x -= rot_step,
            Key::Left => rotation_delta.y += rot_step,
            Key::Right => rotation_delta.y -= rot_step,
            _ => (),
        }
    }

    if position_delta != Vector3::new(0.0, 0.0, 0.0) {
        camera.transform.position += position_delta;
        camera.transform.dirty = true;
    }

    if rotation_delta != Vector3::new(0.0, 0.0, 0.0) {
        camera.transform.rotation += rotation_delta;
        // Clamp pitch to avoid flipping (roughly +/- 89 degrees)
        let pitch_limit = 1.55334306_f32; // ~89 degrees in radians
        if camera.transform.rotation.x > pitch_limit {
            camera.transform.rotation.x = pitch_limit;
        }
        if camera.transform.rotation.x < -pitch_limit {
            camera.transform.rotation.x = -pitch_limit;
        }
        camera.transform.dirty = true;
    }

    // Inputs persist via active_keys; no per-frame clearing required
}

pub fn get_camera_3d_view_projection(
    camera: &mut Camera3D,
    image_extent: Extent2D,
) -> (Matrix4<f32>, Matrix4<f32>) {
    // Build view matrix from position and euler
    let pos = camera.transform.position;
    let e = camera.transform.rotation;
    // Our current convention stores yaw in Y, pitch in X, roll in Z
    let r = UnitQuaternion::from_euler_angles(e.x, e.y, e.z);
    let r_inv = r.inverse();
    let t_inv = Translation3::new(-pos.x, -pos.y, -pos.z);
    let view = r_inv.to_homogeneous() * t_inv.to_homogeneous();

    // Projection (Vulkan NDC requires Y flip)
    let aspect = (image_extent.width as f32).max(1.0) / (image_extent.height as f32).max(1.0);
    let fovy = std::f32::consts::FRAC_PI_3; // 60 degrees
    let znear = 0.1_f32;
    let zfar = 100.0_f32;
    let mut projection = Perspective3::new(aspect, fovy, znear, zfar).to_homogeneous();
    projection[(1, 1)] *= -1.0;

    camera.transform.dirty = false;

    (view, projection)
}
