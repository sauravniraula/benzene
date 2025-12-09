use crate::core::ecs::components::PointLight3D;

// Placeholder system functions for point lights; extend as needed
pub fn set_point_light_3d_color(light: &mut PointLight3D, rgba: nalgebra::Vector4<f32>) {
    light.color = rgba;
}
