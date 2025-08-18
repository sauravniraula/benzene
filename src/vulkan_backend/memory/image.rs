use crate::vulkan_backend::backend::VBackend;
use ash::vk::{self, Extent3D};

pub struct VImage {
    pub image: vk::Image,
}

impl VImage {
    pub fn new(v_backend: &VBackend) -> Self {
        let image_info = vk::ImageCreateInfo::default()
            .image_type(vk::ImageType::TYPE_2D)
            .extent(Extent3D {
                width: 512,
                height: 512,
                depth: 1,
            })
            .mip_levels(1)
            .array_layers(1)
            .format(vk::Format::R8G8B8_SRGB)
            .tiling(vk::ImageTiling::OPTIMAL)
            .initial_layout(vk::ImageLayout::UNDEFINED)
            .usage(vk::ImageUsageFlags::TRANSFER_DST | vk::ImageUsageFlags::SAMPLED)
            .sharing_mode(vk::SharingMode::EXCLUSIVE)
            .samples(vk::SampleCountFlags::TYPE_1);

        let image = unsafe {
            v_backend
                .v_device
                .device
                .create_image(&image_info, None)
                .expect("failed to create image")
        };

        Self { image }
    }
}
