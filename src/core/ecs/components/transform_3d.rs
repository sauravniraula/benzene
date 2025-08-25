use nalgebra::{Matrix4, Vector3};

#[derive(Clone, Debug)]
pub struct Transform3D {
    pub position: Vector3<f32>,
    // rotation_euler contains (roll, pitch, yaw)
    pub rotation_euler: Vector3<f32>,
    pub scale: Vector3<f32>,
    pub cached_transform: Matrix4<f32>,
    pub dirty: bool,
}

impl Transform3D {
    pub fn new(position: Vector3<f32>, rotation_euler: Vector3<f32>, scale: Vector3<f32>) -> Self {
        Self {
            position,
            rotation_euler,
            scale,
            cached_transform: Matrix4::identity(),
            dirty: true,
        }
    }
    pub fn new_default() -> Self {
        Self::new(
            Vector3::new(0.0, 2.0, 6.0),
            Vector3::new(0.0, 0.0, 0.0),
            Vector3::new(1.0, 1.0, 1.0),
        )
    }
}
