use ash::vk;

pub struct VertexData {
    pub position: [f32; 2],
    pub color: [f32; 3],
}

impl VertexData {
    pub fn get_binding_description() -> vk::VertexInputBindingDescription {
        vk::VertexInputBindingDescription::default()
            .binding(0)
            .stride(std::mem::size_of::<VertexData>() as u32)
            .input_rate(vk::VertexInputRate::VERTEX)
    }

    pub fn get_attribute_descriptions() -> [vk::VertexInputAttributeDescription; 2] {
        [
            vk::VertexInputAttributeDescription::default()
                .binding(0)
                .format(vk::Format::R32G32_SFLOAT)
                .location(0)
                .offset(0),
            vk::VertexInputAttributeDescription::default()
                .binding(0)
                .format(vk::Format::R32G32B32_SFLOAT)
                .location(1)
                .offset(std::mem::size_of::<[f32; 2]>() as u32),
        ]
    }
}
