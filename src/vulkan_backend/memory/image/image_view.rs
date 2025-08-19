use ash::vk;

use crate::vulkan_backend::device::VDevice;

pub struct VImageView {
    pub image_view: vk::ImageView,
}

impl VImageView {
    pub fn new(v_device: &VDevice, image: vk::Image) -> Self {
        let subresource_range = vk::ImageSubresourceRange::default()
            .aspect_mask(vk::ImageAspectFlags::COLOR)
            .base_mip_level(0)
            .base_array_layer(0)
            .level_count(1)
            .layer_count(1);

        let image_info = vk::ImageViewCreateInfo::default()
            .image(image)
            .view_type(vk::ImageViewType::TYPE_2D)
            .format(vk::Format::R8G8B8A8_SRGB)
            .subresource_range(subresource_range);

        let image_view = unsafe {
            v_device
                .device
                .create_image_view(&image_info, None)
                .expect("failed to create Image View")
        };

        Self { image_view }
    }

    pub fn destroy(&self, v_device: &VDevice) {
        unsafe { v_device.device.destroy_image_view(self.image_view, None) };
    }
}
