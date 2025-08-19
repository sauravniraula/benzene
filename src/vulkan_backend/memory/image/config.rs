use ash::vk;

pub struct VImageConfig {
    pub extent: vk::Extent3D,
    pub size: u64,
    pub usage: vk::ImageUsageFlags,
    pub sharing_mode: vk::SharingMode,
    pub queue_families: Option<Vec<u32>>,
    pub memory_property: vk::MemoryPropertyFlags,
    pub format: vk::Format,
    pub tiling: vk::ImageTiling,
    pub initial_layout: vk::ImageLayout,
    pub mip_levels: u32,
    pub array_layers: u32,
    pub samples: vk::SampleCountFlags,
}

pub struct VImageViewConfig {
    pub view_type: vk::ImageViewType,
    pub format: vk::Format,
    pub aspect_mask: vk::ImageAspectFlags,
    pub base_mip_level: u32,
    pub level_count: u32,
    pub base_array_layer: u32,
    pub layer_count: u32,
}

impl Default for VImageViewConfig {
    fn default() -> Self {
        Self {
            view_type: vk::ImageViewType::TYPE_2D,
            format: vk::Format::R8G8B8A8_SRGB,
            aspect_mask: vk::ImageAspectFlags::COLOR,
            base_mip_level: 0,
            level_count: 1,
            base_array_layer: 0,
            layer_count: 1,
        }
    }
}

impl VImageConfig {
    pub fn color_2d(
        extent: vk::Extent3D,
        size: u64,
        usage: vk::ImageUsageFlags,
        sharing_mode: vk::SharingMode,
        queue_families: Option<Vec<u32>>,
        memory_property: vk::MemoryPropertyFlags,
        format: vk::Format,
    ) -> Self {
        Self {
            extent,
            size,
            usage,
            sharing_mode,
            queue_families,
            memory_property,
            format,
            tiling: vk::ImageTiling::OPTIMAL,
            initial_layout: vk::ImageLayout::UNDEFINED,
            mip_levels: 1,
            array_layers: 1,
            samples: vk::SampleCountFlags::TYPE_1,
        }
    }

    pub fn external_color_2d(
        extent: vk::Extent3D,
        usage: vk::ImageUsageFlags,
        sharing_mode: vk::SharingMode,
        queue_families: Option<Vec<u32>>,
        format: vk::Format,
    ) -> Self {
        Self {
            extent,
            size: 0,
            usage,
            sharing_mode,
            queue_families,
            memory_property: vk::MemoryPropertyFlags::empty(),
            format,
            tiling: vk::ImageTiling::OPTIMAL,
            initial_layout: vk::ImageLayout::UNDEFINED,
            mip_levels: 1,
            array_layers: 1,
            samples: vk::SampleCountFlags::TYPE_1,
        }
    }
}


