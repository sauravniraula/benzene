use ash::vk;

use crate::vulkan_backend::{
	backend::VBackend,
	device::VDevice,
	memory::{VBuffer, VBufferConfig},
	rendering::Drawable,
	vertex_input::Vertex3D,
};

pub struct Model {
	pub v_buffer: VBuffer,
	pub i_buffer: VBuffer,
	pub index_count: u32,
}

impl Model {
	pub fn new(v_backend: &VBackend, vertices: &[Vertex3D], indices: &[u32]) -> Self {
		let v_buffer = VBuffer::new(
			v_backend,
			VBufferConfig {
				size: (size_of::<Vertex3D>() * vertices.len()) as u64,
				usage: vk::BufferUsageFlags::TRANSFER_DST | vk::BufferUsageFlags::VERTEX_BUFFER,
				sharing_mode: v_backend.v_device.buffer_sharing_mode,
				queue_families: Some(v_backend.v_device.buffer_queue_family_indices.clone()),
				memory_property: vk::MemoryPropertyFlags::DEVICE_LOCAL,
			},
		);
		let vertices_data_ptr = vertices.as_ptr() as *const u8;
		v_buffer.copy_to_buffer(v_backend, vertices_data_ptr, v_buffer.config.size);

		let i_buffer = VBuffer::new(
			v_backend,
			VBufferConfig {
				size: (size_of::<u32>() * indices.len()) as u64,
				usage: vk::BufferUsageFlags::TRANSFER_DST | vk::BufferUsageFlags::INDEX_BUFFER,
				sharing_mode: v_backend.v_device.buffer_sharing_mode,
				queue_families: Some(v_backend.v_device.buffer_queue_family_indices.clone()),
				memory_property: vk::MemoryPropertyFlags::DEVICE_LOCAL,
			},
		);
		let indices_data_ptr = indices.as_ptr() as *const u8;
		i_buffer.copy_to_buffer(v_backend, indices_data_ptr, i_buffer.config.size);

		Self { v_buffer, i_buffer, index_count: indices.len() as u32 }
	}

	pub fn destroy(&self, v_backend: &VBackend) {
		self.i_buffer.destroy(v_backend);
		self.v_buffer.destroy(v_backend);
	}
}

impl Drawable for Model {
	fn draw(&self, v_device: &VDevice, command_buffer: vk::CommandBuffer) {
		unsafe {
			let vertex_buffers = [self.v_buffer.buffer];
			let offsets = [0u64];
			v_device
				.device
				.cmd_bind_vertex_buffers(command_buffer, 0, &vertex_buffers, &offsets);
			v_device
				.device
				.cmd_bind_index_buffer(command_buffer, self.i_buffer.buffer, 0, vk::IndexType::UINT32);
			v_device
				.device
				.cmd_draw_indexed(command_buffer, self.index_count, 1, 0, 0, 0);
		}
	}
}


