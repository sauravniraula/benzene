use glfw::Key;
use nalgebra::Vector3;
use std::collections::HashSet;

use crate::core::ecs::components::Transform3D;

pub struct Camera3D {
    pub transform: Transform3D,
    pub speed: f32,
    pub rotation_speed: f32,
    pub active_keys: HashSet<Key>,
}

impl Camera3D {
    pub fn new(transform: Transform3D) -> Self {
        Self {
            transform,
            speed: 5.0,
            rotation_speed: 1.5,
            active_keys: HashSet::new(),
        }
    }

    pub fn new_default() -> Self {
        Self::new(Transform3D::new(
            Vector3::new(2.0, 1.0, 0.0),
            Vector3::new(0.0, 0.0, 0.0),
            Vector3::new(1.0, 1.0, 1.0),
        ))
    }
}
