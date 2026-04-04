use std::collections::HashSet;

use nalgebra::Vector4;
use winit::{
    event::ElementState,
    keyboard::{KeyCode, PhysicalKey},
};

use crate::core::ecs::{
    entities::Entity,
    types::{CursorMovedEvent, KeyboardInputEvent},
};

pub struct InputState {
    pressed_keys: HashSet<KeyCode>,
    look_delta_x: f32,
    look_delta_y: f32,
    last_cursor_position: Option<(f64, f64)>,
}

impl InputState {
    pub fn new() -> Self {
        Self {
            pressed_keys: HashSet::new(),
            look_delta_x: 0.0,
            look_delta_y: 0.0,
            last_cursor_position: None,
        }
    }

    pub fn handle_keyboard_input(&mut self, event: &KeyboardInputEvent) {
        if let PhysicalKey::Code(key) = event.key {
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

    pub fn handle_cursor_moved(&mut self, event: &CursorMovedEvent) {
        if let Some((last_x, last_y)) = self.last_cursor_position {
            self.look_delta_y += (event.x - last_x) as f32;
            self.look_delta_x += (event.y - last_y) as f32;
        }

        self.last_cursor_position = Some((event.x, event.y));
    }

    pub fn is_key_pressed(&self, key: KeyCode) -> bool {
        self.pressed_keys.contains(&key)
    }

    pub fn has_pending_camera_input(&self) -> bool {
        !self.pressed_keys.is_empty() || self.look_delta_x != 0.0 || self.look_delta_y != 0.0
    }

    pub fn take_look_delta(&mut self) -> (f32, f32) {
        let delta = (self.look_delta_x, self.look_delta_y);
        self.look_delta_x = 0.0;
        self.look_delta_y = 0.0;
        delta
    }

    pub fn reset_cursor_tracking(&mut self) {
        self.last_cursor_position = None;
        self.look_delta_x = 0.0;
        self.look_delta_y = 0.0;
    }
}

pub struct SceneResources {
    pub active_camera: Option<Entity>,
    pub ambient_color: Vector4<f32>,
    pub input: InputState,
}

impl SceneResources {
    pub fn new(ambient_color: Vector4<f32>) -> Self {
        Self {
            active_camera: None,
            ambient_color,
            input: InputState::new(),
        }
    }
}
