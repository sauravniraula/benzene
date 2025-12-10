use std::collections::HashSet;

use nalgebra::{Rotation3, Vector3};
use winit::{
    event::ElementState,
    keyboard::{KeyCode, PhysicalKey},
};

use crate::{
    core::ecs::{
        components::Camera3D,
        types::{CursorMovedEvent, KeyboardInputEvent},
    },
    log,
};

pub fn camera_3d_handle_ki_event(camera: &mut Camera3D, event: &KeyboardInputEvent) {
    match event.key {
        PhysicalKey::Code(key_code) => {
            let is_relevant = matches!(
                key_code,
                KeyCode::KeyW
                    | KeyCode::KeyA
                    | KeyCode::KeyS
                    | KeyCode::KeyD
                    | KeyCode::Space
                    | KeyCode::AltLeft,
            );
            if !is_relevant {
                return;
            }
            camera.ki_events.push(event.clone());
        }
        _ => (),
    }
}

pub fn camera_3d_handle_cm_event(camera: &mut Camera3D, event: &CursorMovedEvent) {
    camera.cm_events.push(event.clone());
}

pub fn camera_3d_compute_transform(camera: &mut Camera3D, dt: f32) {
    log!("Computing camera 3d transform");

    let mut d_rot = Vector3::<f32>::new(0.0, 0.0, 0.0);

    let mut last_x = 0.0;
    let mut last_y = 0.0;
    let mut is_first = true;
    for event in camera.cm_events.iter() {
        if !is_first {
            d_rot.y += (event.x - last_x) as f32;
            d_rot.x += (event.y - last_y) as f32;
        }
        last_x = event.x;
        last_y = event.y;
        is_first = false;
    }

    // Directions
    let ea = camera.transform.rotation;
    let rotation = Rotation3::from_euler_angles(ea.x, ea.y, ea.z);
    let fv = Vector3::<f32>::new(0.0, 0.0, -1.0);
    let fv = (rotation * fv).normalize();
    let uv = Vector3::y();
    let rv = fv.cross(&uv).normalize();

    let mut d_pos = Vector3::new(0.0, 0.0, 0.0);

    let key_dir = [
        (KeyCode::KeyW, fv),
        (KeyCode::KeyS, -fv),
        (KeyCode::KeyA, -rv),
        (KeyCode::KeyD, rv),
        (KeyCode::Space, uv),
        (KeyCode::AltLeft, -uv),
    ];

    let mut pressed: HashSet<KeyCode> = HashSet::new();

    for event in camera.ki_events.iter() {
        if let PhysicalKey::Code(key) = event.key {
            match event.state {
                ElementState::Pressed => {
                    pressed.insert(key);
                }
                ElementState::Released => {
                    pressed.remove(&key);
                }
            }
        }
    }

    for (key, dir) in key_dir {
        if pressed.contains(&key) {
            d_pos += dir;
        }
    }

    // Clear events and add those still not released
    camera.cm_events.clear();
    camera.ki_events.clear();
    for key in pressed {
        camera.ki_events.push(KeyboardInputEvent {
            key: PhysicalKey::Code(key),
            state: ElementState::Pressed,
            repeat: true,
        });
    }

    camera.transform.position += d_pos * dt * camera.speed;
    camera.transform.rotation += d_rot * dt * camera.rotation_speed;
    camera.transform.dirty = true;
}
