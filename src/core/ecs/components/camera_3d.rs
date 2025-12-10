use nalgebra::Vector3;

use crate::core::ecs::{
    components::Transform3D,
    types::{CursorMovedEvent, KeyboardInputEvent},
};

pub struct Camera3D {
    pub transform: Transform3D,
    pub speed: f32,
    pub rotation_speed: f32,
    pub ki_events: Vec<KeyboardInputEvent>,
    pub cm_events: Vec<CursorMovedEvent>,
}

impl Camera3D {
    pub fn new(transform: Transform3D) -> Self {
        Self {
            transform,
            speed: 5.0,
            rotation_speed: 0.1,
            ki_events: vec![],
            cm_events: vec![]
        }
    }

    pub fn new_default() -> Self {
        Self::new(Transform3D::new(
            Vector3::new(0.0, 1.0, 5.0),
            Vector3::new(0.0, 0.0, 0.0),
            Vector3::new(1.0, 1.0, 1.0),
        ))
    }
}
