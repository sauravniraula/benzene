use std::collections::HashMap;

use ash::vk;

use crate::{
    core::ecs::types::Id,
    vulkan_backend::{
        backend::VBackend,
        memory::image::{VImage, VImageConfig, image_view::VImageView, sampler::VSampler},
    },
};

pub struct ShadowMapping {
    pub spot_light_maps: HashMap<Id, VImage>,
    pub spot_light_views: HashMap<Id, VImageView>,
    pub spot_light_samplers: HashMap<Id, VSampler>,
}

impl ShadowMapping {
    pub fn new() -> Self {
        Self {
            spot_light_maps: HashMap::new(),
            spot_light_views: HashMap::new(),
            spot_light_samplers: HashMap::new(),
        }
    }

    pub fn add_spot_light(&mut self, v_backend: &VBackend, entity_id: Id) {
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

        self.spot_light_maps.insert(entity_id, spot_light_map);
        self.spot_light_views.insert(entity_id, spot_light_map_view);
        self.spot_light_samplers
            .insert(entity_id, spot_light_sampler);
    }

    pub fn remove_spot_light(&mut self, v_backend: &VBackend, entity_id: &Id) {
        if let Some(image) = self.spot_light_maps.remove(entity_id) {
            image.destroy(&v_backend.v_device, &v_backend.v_memory_manager);
        }
        if let Some(view) = self.spot_light_views.remove(entity_id) {
            view.destroy(&v_backend.v_device);
        }
        if let Some(sampler) = self.spot_light_samplers.remove(entity_id) {
            sampler.destroy(&v_backend.v_device);
        }
    }

    pub fn destroy(&mut self, v_backend: &VBackend) {
        for (_, image) in self.spot_light_maps.drain() {
            image.destroy(&v_backend.v_device, &v_backend.v_memory_manager);
        }
        for (_, view) in self.spot_light_views.drain() {
            view.destroy(&v_backend.v_device);
        }
        for (_, sampler) in self.spot_light_samplers.drain() {
            sampler.destroy(&v_backend.v_device);
        }
    }
}
