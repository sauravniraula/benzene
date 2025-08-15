use crate::vulkan_backend::vertex_input::BindableVertexInput;
use ash::vk;
use memoffset::offset_of;
use nalgebra::Vector3;

pub struct Vertex3D {
    pub pos: Vector3<f32>,
    pub color: Vector3<f32>,
}

impl BindableVertexInput for Vertex3D {
    fn get_binding_descriptions() -> Vec<vk::VertexInputBindingDescription> {
        [vk::VertexInputBindingDescription::default()
            .binding(0)
            .input_rate(vk::VertexInputRate::VERTEX)
            .stride(size_of::<Vertex3D>() as u32)]
        .into()
    }

    fn get_attribute_descriptions() -> Vec<vk::VertexInputAttributeDescription> {
        [
            vk::VertexInputAttributeDescription::default()
                .binding(0)
                .location(0)
                .format(vk::Format::R32G32B32_SFLOAT)
                .offset(0),
            vk::VertexInputAttributeDescription::default()
                .binding(0)
                .location(1)
                .format(vk::Format::R32G32B32_SFLOAT)
                .offset(offset_of!(Vertex3D, color) as u32),
        ]
        .into()
    }
}
