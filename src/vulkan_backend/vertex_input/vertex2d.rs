use crate::vulkan_backend::vertex_input::BindableVertexInput;
use ash::vk;
use memoffset::offset_of;

pub struct Vertex2D {
    pub pos: [f32; 2],
    pub color: [f32; 3],
}

impl BindableVertexInput for Vertex2D {
    fn get_binding_descriptions() -> Vec<vk::VertexInputBindingDescription> {
        [vk::VertexInputBindingDescription::default()
            .binding(0)
            .input_rate(vk::VertexInputRate::VERTEX)
            .stride(size_of::<Vertex2D>() as u32)]
        .into()
    }

    fn get_attribute_descriptions() -> Vec<vk::VertexInputAttributeDescription> {
        [
            vk::VertexInputAttributeDescription::default()
                .binding(0)
                .location(0)
                .format(vk::Format::R32G32_SFLOAT)
                .offset(0),
            vk::VertexInputAttributeDescription::default()
                .binding(0)
                .location(0)
                .format(vk::Format::R32G32_SFLOAT)
                .offset(offset_of!(Vertex2D, color) as u32),
        ]
        .into()
    }
}
