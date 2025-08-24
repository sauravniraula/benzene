use nalgebra::Matrix4;

pub struct ModelPushConstant {
    pub transform: Matrix4<f32>,
}
