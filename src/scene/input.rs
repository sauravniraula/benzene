use std::collections::HashSet;

use nalgebra::Vector3;
use winit::{
    event::{ElementState, MouseButton, WindowEvent},
    keyboard::{KeyCode, PhysicalKey},
};

use super::{Camera, Transform};

#[derive(Clone, Debug, Default)]
pub struct FrameInput {
    pressed_keys: HashSet<KeyCode>,
    look_delta: (f32, f32),
}

impl FrameInput {
    pub fn is_key_pressed(&self, key: KeyCode) -> bool {
        self.pressed_keys.contains(&key)
    }

    pub fn look_delta(&self) -> (f32, f32) {
        self.look_delta
    }
}

#[derive(Debug, Default)]
pub struct InputState {
    pub(crate) pressed_keys: HashSet<KeyCode>,
    last_cursor_position: Option<(f64, f64)>,
    look_delta: (f32, f32),
    cursor_captured: bool,
}

impl InputState {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn handle_window_event(&mut self, event: &WindowEvent) {
        match event {
            WindowEvent::KeyboardInput { event, .. } => {
                if let PhysicalKey::Code(key) = event.physical_key {
                    match event.state {
                        ElementState::Pressed => {
                            self.pressed_keys.insert(key);
                        }
                        ElementState::Released => {
                            self.pressed_keys.remove(&key);
                        }
                    }
                }
            }
            WindowEvent::MouseInput { state, button, .. } => {
                if *button == MouseButton::Left {
                    self.cursor_captured = *state == ElementState::Pressed;
                    self.last_cursor_position = None;
                    self.look_delta = (0.0, 0.0);
                }
            }
            WindowEvent::CursorMoved { position, .. } => {
                if !self.cursor_captured {
                    return;
                }

                if let Some((last_x, last_y)) = self.last_cursor_position {
                    self.look_delta.1 += (position.x - last_x) as f32;
                    self.look_delta.0 += (position.y - last_y) as f32;
                }

                self.last_cursor_position = Some((position.x, position.y));
            }
            WindowEvent::Focused(false) => {
                self.cursor_captured = false;
                self.last_cursor_position = None;
                self.look_delta = (0.0, 0.0);
            }
            _ => {}
        }
    }

    pub fn frame_input(&mut self) -> FrameInput {
        let frame = FrameInput {
            pressed_keys: self.pressed_keys.clone(),
            look_delta: self.look_delta,
        };
        self.look_delta = (0.0, 0.0);
        frame
    }

    pub fn cursor_captured(&self) -> bool {
        self.cursor_captured
    }
}

pub(crate) fn apply_camera_input(
    camera: &Camera,
    transform: &mut Transform,
    input: &FrameInput,
    dt: f32,
) {
    let (pitch_delta, yaw_delta) = input.look_delta();
    let rotation = transform.rotation_matrix();
    let forward = (rotation * Vector3::<f32>::new(0.0, 0.0, -1.0)).normalize();
    let up = Vector3::y();
    let right = forward.cross(&up).normalize();

    let mut position_delta = Vector3::new(0.0, 0.0, 0.0);
    let directions = [
        (KeyCode::KeyW, forward),
        (KeyCode::KeyS, -forward),
        (KeyCode::KeyA, -right),
        (KeyCode::KeyD, right),
        (KeyCode::Space, up),
        (KeyCode::AltLeft, -up),
    ];

    for (key, direction) in directions {
        if input.is_key_pressed(key) {
            position_delta += direction;
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
