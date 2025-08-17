use ash::vk;

use crate::vulkan_backend::{device::VDevice, rendering::RecordContext};

pub trait Recordable {
	fn record(&self, ctx: &RecordContext);
}

pub trait Drawable {
	fn draw(&self, v_device: &VDevice, command_buffer: vk::CommandBuffer);
}



