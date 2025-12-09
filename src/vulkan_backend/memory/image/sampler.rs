use ash::vk;

use crate::vulkan_backend::device::{VDevice, VPhysicalDevice};

pub struct VSampler {
    pub sampler: vk::Sampler,
}

impl VSampler {
    pub fn new(v_device: &VDevice, v_physical_device: &VPhysicalDevice) -> Self {
        let sampler_info = vk::SamplerCreateInfo::default()
            .mag_filter(vk::Filter::LINEAR)
            .min_filter(vk::Filter::LINEAR)
            .address_mode_u(vk::SamplerAddressMode::REPEAT)
            .address_mode_v(vk::SamplerAddressMode::REPEAT)
            .address_mode_w(vk::SamplerAddressMode::REPEAT)
            .anisotropy_enable(true)
            .max_anisotropy(v_physical_device.properties.limits.max_sampler_anisotropy)
            .border_color(vk::BorderColor::INT_OPAQUE_BLACK)
            .unnormalized_coordinates(false)
            .compare_enable(false)
            .compare_op(vk::CompareOp::ALWAYS)
            .mipmap_mode(vk::SamplerMipmapMode::LINEAR)
            .mip_lod_bias(0.0)
            .min_lod(0.0)
            .max_lod(0.0);

        let sampler = unsafe {
            v_device
                .device
                .create_sampler(&sampler_info, None)
                .expect("failed to create sampler")
        };

        Self { sampler }
    }

    pub fn destroy(&self, v_device: &VDevice) {
        unsafe { v_device.device.destroy_sampler(self.sampler, None) };
    }
}
