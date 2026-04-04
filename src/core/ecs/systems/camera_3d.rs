use nalgebra::{Rotation3, Vector3};
use winit::keyboard::KeyCode;

use crate::core::ecs::{
    components::{Camera3D, Transform3D},
    resources::InputState,
};

pub fn camera_3d_apply_input(
    camera: &Camera3D,
    transform: &mut Transform3D,
    input: &mut InputState,
    dt: f32,
) {
    let (pitch_delta, yaw_delta) = input.take_look_delta();

    let rotation = Rotation3::from_euler_angles(
        transform.rotation.x,
        transform.rotation.y,
        transform.rotation.z,
    );
    let forward = (rotation * Vector3::<f32>::new(0.0, 0.0, -1.0)).normalize();
    let up = Vector3::y();
    let right = forward.cross(&up).normalize();

    let mut position_delta = Vector3::new(0.0, 0.0, 0.0);
    let key_dir = [
        (KeyCode::KeyW, forward),
        (KeyCode::KeyS, -forward),
        (KeyCode::KeyA, -right),
        (KeyCode::KeyD, right),
        (KeyCode::Space, up),
        (KeyCode::AltLeft, -up),
    ];

    for (key, dir) in key_dir {
        if input.is_key_pressed(key) {
            position_delta += dir;
        }
    }

    let rotation_delta = Vector3::new(
        pitch_delta * dt * camera.rotation_speed,
        yaw_delta * dt * camera.rotation_speed,
        0.0,
    );

    if position_delta != Vector3::zeros() || rotation_delta != Vector3::zeros() {
        transform.position += position_delta * dt * camera.speed;
        transform.rotation += rotation_delta;
        transform.dirty = true;
    }
}
