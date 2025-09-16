use ash::vk;

pub struct VPushConstant {
    pub push_constant: vk::PushConstantRange,
}

impl VPushConstant {
    pub fn new<T>(stage_flags: vk::ShaderStageFlags) -> Self {
        let push_constant = vk::PushConstantRange::default()
            .offset(0)
            .size(size_of::<T>() as u32)
            .stage_flags(stage_flags);

        Self { push_constant }
    }

    pub fn with_size(size: u32, stage_flags: vk::ShaderStageFlags) -> Self {
        let push_constant = vk::PushConstantRange::default()
            .offset(0)
            .size(size)
            .stage_flags(stage_flags);

        Self { push_constant }
    }
}
