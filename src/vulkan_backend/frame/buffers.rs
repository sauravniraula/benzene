use std::collections::HashMap;

use ash::vk::{self, Extent2D};

use crate::{
    shared::types::Id,
    vulkan_backend::{device::VDevice, memory::image::image_view::VImageView},
};

pub struct VFramebuffers {
    buffers: HashMap<Id, vk::Framebuffer>,
}

impl VFramebuffers {
    pub fn new() -> Self {
        Self {
            buffers: HashMap::new(),
        }
    }

    pub fn get_by_id(&self, id: &Id) -> &vk::Framebuffer {
        self.buffers.get(id).unwrap()
    }

    pub fn add_framebuffer(
        &mut self,
        v_device: &VDevice,
        render_pass: vk::RenderPass,
        id: Id,
        color_view: Option<&VImageView>,
        depth_view: Option<&VImageView>,
        image_extent: Extent2D,
    ) {
        let mut attachments: Vec<vk::ImageView> = Vec::new();
        if let Some(cv) = color_view {
            attachments.push(cv.image_view);
        }
        if let Some(dv) = depth_view {
            attachments.push(dv.image_view);
        }

        let info = vk::FramebufferCreateInfo::default()
            .attachments(&attachments)
            .render_pass(render_pass)
            .width(image_extent.width)
            .height(image_extent.height)
            .layers(1);
        let framebuffer = unsafe {
            v_device
                .device
                .create_framebuffer(&info, None)
                .expect("failed to create framebuffer")
        };
        self.buffers.insert(id, framebuffer);
    }

    pub fn remove_framebuffer(&mut self, v_device: &VDevice, id: &Id) {
        if let Some(fb) = self.buffers.remove(id) {
            unsafe {
                v_device.device.destroy_framebuffer(fb, None);
            }
        }
    }

    pub fn remove_all_framebuffers(&mut self, v_device: &VDevice) {
        unsafe {
            for (_, &framebuffer) in self.buffers.iter() {
                v_device.device.destroy_framebuffer(framebuffer, None);
            }
        }
        self.buffers.clear();
    }

    pub fn destroy(&self, v_device: &VDevice) {
        unsafe {
            for (_, &framebuffer) in self.buffers.iter() {
                v_device.device.destroy_framebuffer(framebuffer, None);
            }
        }
    }
}
