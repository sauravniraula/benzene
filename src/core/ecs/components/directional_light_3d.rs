use nalgebra::Vector4;

pub struct DirectionalLight3D {
    pub color: Vector4<f32>,
}

impl DirectionalLight3D {
    pub fn new(color: Vector4<f32>) -> Self {
        Self { color }
    }
}
