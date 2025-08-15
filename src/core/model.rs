use ash::vk;
use nalgebra::Vector3;

use crate::vulkan_backend::{
    backend::VBackend,
    memory::{VBuffer, VBufferConfig},
    vertex_input::Vertex3D,
};

pub struct VModel {
    pub vertices: Vec<Vertex3D>,
    pub v_buffer: VBuffer,
    pub indices: Vec<u32>,
    pub i_buffer: VBuffer,
}

impl VModel {
    pub fn new(v_backend: &VBackend) -> Self {
			// Cube geometry: 1x1x1 centered at origin, per-face colors
			let half = 0.5f32;
			let front_color = Vector3::new(1.0, 0.0, 0.0);   // red
			let back_color = Vector3::new(0.0, 1.0, 0.0);    // green
			let left_color = Vector3::new(0.0, 0.0, 1.0);    // blue
			let right_color = Vector3::new(1.0, 1.0, 0.0);   // yellow
			let bottom_color = Vector3::new(1.0, 0.0, 1.0);  // magenta
			let top_color = Vector3::new(0.0, 1.0, 1.0);     // cyan

			let vertices: Vec<Vertex3D> = vec![
				// Front (+Z)
				Vertex3D { pos: Vector3::new(-half, -half,  half), color: front_color }, // 0
				Vertex3D { pos: Vector3::new( half, -half,  half), color: front_color }, // 1
				Vertex3D { pos: Vector3::new( half,  half,  half), color: front_color }, // 2
				Vertex3D { pos: Vector3::new(-half,  half,  half), color: front_color }, // 3
				// Back (-Z)
				Vertex3D { pos: Vector3::new(-half, -half, -half), color: back_color },   // 4
				Vertex3D { pos: Vector3::new( half, -half, -half), color: back_color },   // 5
				Vertex3D { pos: Vector3::new( half,  half, -half), color: back_color },   // 6
				Vertex3D { pos: Vector3::new(-half,  half, -half), color: back_color },   // 7
				// Left (-X)
				Vertex3D { pos: Vector3::new(-half, -half,  half), color: left_color },   // 8
				Vertex3D { pos: Vector3::new(-half,  half,  half), color: left_color },   // 9
				Vertex3D { pos: Vector3::new(-half,  half, -half), color: left_color },   // 10
				Vertex3D { pos: Vector3::new(-half, -half, -half), color: left_color },   // 11
				// Right (+X)
				Vertex3D { pos: Vector3::new( half, -half,  half), color: right_color },  // 12
				Vertex3D { pos: Vector3::new( half, -half, -half), color: right_color },  // 13
				Vertex3D { pos: Vector3::new( half,  half, -half), color: right_color },  // 14
				Vertex3D { pos: Vector3::new( half,  half,  half), color: right_color },  // 15
				// Bottom (-Y)
				Vertex3D { pos: Vector3::new(-half, -half,  half), color: bottom_color }, // 16
				Vertex3D { pos: Vector3::new(-half, -half, -half), color: bottom_color }, // 17
				Vertex3D { pos: Vector3::new( half, -half, -half), color: bottom_color }, // 18
				Vertex3D { pos: Vector3::new( half, -half,  half), color: bottom_color }, // 19
				// Top (+Y)
				Vertex3D { pos: Vector3::new(-half,  half,  half), color: top_color },    // 20
				Vertex3D { pos: Vector3::new( half,  half,  half), color: top_color },    // 21
				Vertex3D { pos: Vector3::new( half,  half, -half), color: top_color },    // 22
				Vertex3D { pos: Vector3::new(-half,  half, -half), color: top_color },    // 23
			];

			// Indices (clockwise winding)
			let indices: Vec<u32> = vec![
				// Front
				0, 1, 2, 0, 2, 3,
				// Back
				4, 6, 5, 4, 7, 6,
				// Left
				8, 9, 10, 8, 10, 11,
				// Right
				12, 13, 14, 12, 14, 15,
				// Bottom
				16, 17, 18, 16, 18, 19,
				// Top
				20, 21, 22, 20, 22, 23,
			];

        // Vertex Buffer
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

        // Index Buffer
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

        Self {
            vertices,
            v_buffer,
            indices,
            i_buffer,
        }
    }

    pub fn destroy(&self, v_backend: &VBackend) {
        self.i_buffer.destroy(v_backend);
        self.v_buffer.destroy(v_backend);
    }
}
