use nalgebra::Vector4;

pub struct PointLight3D {
    pub color: Vector4<f32>,
}

impl PointLight3D {
    pub fn new(color: Vector4<f32>) -> Self {
        Self { color }
    }
}
