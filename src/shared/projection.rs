use nalgebra::Matrix4;

pub fn vulkan_orthographic(left: f32, right: f32, bottom: f32, top: f32, near: f32, far: f32) -> Matrix4<f32> {
    Matrix4::new(
        2.0 / (right - left), 0.0, 0.0, -(right + left) / (right - left),
        0.0, 2.0 / (top - bottom), 0.0, -(top + bottom) / (top - bottom),
        0.0, 0.0, 1.0 / (far - near), -near / (far - near),
        0.0, 0.0, 0.0, 1.0
    )
}