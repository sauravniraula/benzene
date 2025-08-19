use ash::vk;

use crate::vulkan_backend::{device::VDevice};
use crate::vulkan_backend::memory::image::config::VImageViewConfig;

pub struct VImageView {
    pub image_view: vk::ImageView,
}

impl VImageView {
    pub fn new(v_device: &VDevice, image: vk::Image, config: VImageViewConfig) -> Self {
        let subresource_range = vk::ImageSubresourceRange::default()
            .aspect_mask(config.aspect_mask)
            .base_mip_level(config.base_mip_level)
            .base_array_layer(config.base_array_layer)
            .level_count(config.level_count)
            .layer_count(config.layer_count);

        let image_info = vk::ImageViewCreateInfo::default()
            .image(image)
            .view_type(config.view_type)
            .format(config.format)
            .subresource_range(subresource_range);

        let image_view = unsafe {
            v_device
                .device
                .create_image_view(&image_info, None)
                .expect("failed to create Image View")
        };

        Self { image_view }
    }

    pub fn new_2d_color(v_device: &VDevice, image: vk::Image, format: vk::Format) -> Self {
        Self::new(
            v_device,
            image,
            VImageViewConfig {
                view_type: vk::ImageViewType::TYPE_2D,
                format,
                aspect_mask: vk::ImageAspectFlags::COLOR,
                base_mip_level: 0,
                level_count: 1,
                base_array_layer: 0,
                layer_count: 1,
            },
        )
    }

    

    pub fn destroy(&self, v_device: &VDevice) {
        unsafe { v_device.device.destroy_image_view(self.image_view, None) };
    }
}
