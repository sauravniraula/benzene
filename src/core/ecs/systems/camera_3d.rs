use std::collections::HashSet;

use nalgebra::Vector3;
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

pub fn camera_3d_handle_cm_event(camera: &mut Camera3D, event: &CursorMovedEvent) {}

pub fn camera_3d_compute_transform(camera: &mut Camera3D, dt: f32) {
    log!("Computing camera 3d transform");

    let mut d_pos = Vector3::new(0.0, 0.0, 0.0);

    let key_dir = [
        (KeyCode::KeyW, Vector3::new(0.0, 0.0, -1.0)),
        (KeyCode::KeyS, Vector3::new(0.0, 0.0, 1.0)),
        (KeyCode::KeyA, Vector3::new(-1.0, 0.0, 0.0)),
        (KeyCode::KeyD, Vector3::new(1.0, 0.0, 0.0)),
        (KeyCode::Space, Vector3::new(0.0, 1.0, 0.0)),
        (KeyCode::AltLeft, Vector3::new(0.0, -1.0, 0.0)),
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

    camera.transform.position += d_pos * dt * camera.speed;
    camera.transform.dirty = true;

    // Clear events and add those still not released
    camera.ki_events.clear();
    for key in pressed {
        camera.ki_events.push(KeyboardInputEvent {
            key: PhysicalKey::Code(key),
            state: ElementState::Pressed,
            repeat: true,
        });
    }
}
