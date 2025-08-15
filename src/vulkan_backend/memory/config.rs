use ash::vk;

pub struct VBufferConfig {
    pub size: u64,
    pub usage: vk::BufferUsageFlags,
    pub sharing_mode: vk::SharingMode,
    pub queue_families: Option<Vec<u32>>,
    pub memory_property: vk::MemoryPropertyFlags,
}

pub struct VAllocateMemoryConfig {
    pub size: u64,
    pub memory_type: u32,
    pub properties: vk::MemoryPropertyFlags,
}
