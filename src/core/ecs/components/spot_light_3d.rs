use nalgebra::Vector4;

pub struct SpotLight3D {
    pub color: Vector4<f32>,
}

impl SpotLight3D {
    pub fn new(color: Vector4<f32>) -> Self {
        Self { color }
    }
}
