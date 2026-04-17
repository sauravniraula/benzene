use std::mem::offset_of;

use ash;
use glam;

pub struct Vertex3D {
    pub pos: glam::Vec3,
    pub color: glam::Vec3,
}

impl Vertex3D {
    pub fn size() -> u64 {
        size_of::<Vertex3D>() as u64
    }

    pub fn get_binding_descriptions() -> Vec<ash::vk::VertexInputBindingDescription> {
        vec![
            ash::vk::VertexInputBindingDescription::default()
                .binding(0)
                .stride(Vertex3D::size() as u32)
                .input_rate(ash::vk::VertexInputRate::VERTEX),
        ]
    }

    pub fn get_attribute_descriptions() -> Vec<ash::vk::VertexInputAttributeDescription> {
        vec![
            ash::vk::VertexInputAttributeDescription::default()
                .location(0)
                .binding(0)
                .format(ash::vk::Format::R32G32B32_SFLOAT)
                .offset(offset_of!(Vertex3D, pos) as u32),
            ash::vk::VertexInputAttributeDescription::default()
                .location(1)
                .binding(0)
                .format(ash::vk::Format::R32G32B32_SFLOAT)
                .offset(offset_of!(Vertex3D, color) as u32),
        ]
    }
}
