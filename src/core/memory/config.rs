use ash::vk;

pub struct VBufferConfig {
    pub size: u64,
}

pub struct VAllocateMemoryConfig {
    pub size: u64,
    pub memory_type: u32,
    pub properties: vk::MemoryPropertyFlags,
}
