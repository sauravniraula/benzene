use glfw::Key;

use crate::core::ecs::components::Transform3D;

pub struct Camera3D {
    pub transform: Transform3D,
    pub speed: f32,
    pub rotation_speed: f32,
    pub recent_keys: Vec<Key>,
    pub dirty: bool,
}

impl Camera3D {
    pub fn new(transform: Transform3D) -> Self {
        Self {
            transform,
            speed: 5.0,
            rotation_speed: 1.5,
            recent_keys: Vec::new(),
            dirty: true,
        }
    }

    pub fn new_default() -> Self {
        Self::new(Transform3D::new_default())
    }
}
