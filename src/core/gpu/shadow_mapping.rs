use ash::vk;

use crate::vulkan_backend::{
    backend::VBackend,
    memory::image::{VImage, VImageConfig, image_view::VImageView, sampler::VSampler},
};

pub struct ShadowMapping {
    pub spot_light_maps: Vec<VImage>,
    pub spot_light_views: Vec<VImageView>,
    pub spot_light_samplers: Vec<VSampler>,
}

impl ShadowMapping {
    pub fn new() -> Self {
        Self {
            spot_light_maps: Vec::new(),
            spot_light_views: Vec::new(),
            spot_light_samplers: Vec::new(),
        }
    }

    pub fn add_spot_light(&mut self, v_backend: &VBackend) {
        let extent = v_backend.v_swapchain.image_extent;
        let spot_light_map = VImage::new(
            &v_backend.v_device,
            &v_backend.v_physical_device,
            &v_backend.v_memory_manager,
            VImageConfig::image_2d(
                vk::Extent3D {
                    width: extent.width,
                    height: extent.height,
                    depth: 1,
                },
                extent.width as u64 * extent.height as u64 * 4,
                vk::ImageUsageFlags::DEPTH_STENCIL_ATTACHMENT | vk::ImageUsageFlags::SAMPLED,
                v_backend.v_device.buffer_sharing_mode,
                Some(v_backend.v_device.buffer_queue_family_indices.clone()),
                vk::MemoryPropertyFlags::DEVICE_LOCAL,
                v_backend
                    .v_physical_device
                    .get_format_for_depth_stencil(&v_backend.v_instance),
            ),
        );
        let spot_light_map_view = VImageView::new_2d(
            &v_backend.v_device,
            &spot_light_map,
            vk::ImageAspectFlags::DEPTH,
            spot_light_map.config.format,
        );
        let spot_light_sampler = VSampler::new(&v_backend.v_device, &v_backend.v_physical_device);

        self.spot_light_maps.push(spot_light_map);
        self.spot_light_views.push(spot_light_map_view);
        self.spot_light_samplers.push(spot_light_sampler);
    }
}
