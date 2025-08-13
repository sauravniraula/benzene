use ash::vk;
use nalgebra::Vector3;

use crate::core::{
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
			// House geometry: base (walls), gable roof, door and windows
			let wall_color = Vector3::new(0.95, 0.90, 0.80); // light beige walls
			let roof_color = Vector3::new(0.60, 0.10, 0.10); // dark red/brown roof
			let door_color = Vector3::new(0.40, 0.20, 0.05); // wooden brown door
			let window_color = Vector3::new(0.60, 0.80, 1.00); // light blue windows

			// Dimensions
			let x_min = -0.6f32; let x_max = 0.6f32;
			let y_min = -0.5f32; let y_wall_top = 0.2f32; let y_ridge = 0.6f32;
			let z_min = -0.6f32; let z_max = 0.6f32;

			let vertices: Vec<Vertex3D> = vec![
				// Walls (rectangular prism)
				// Front face (z = z_max)
				Vertex3D { pos: Vector3::new(x_min, y_min, z_max), color: wall_color }, // 0
				Vertex3D { pos: Vector3::new(x_max, y_min, z_max), color: wall_color }, // 1
				Vertex3D { pos: Vector3::new(x_max, y_wall_top, z_max), color: wall_color }, // 2
				Vertex3D { pos: Vector3::new(x_min, y_wall_top, z_max), color: wall_color }, // 3
				// Back face (z = z_min)
				Vertex3D { pos: Vector3::new(x_min, y_min, z_min), color: wall_color }, // 4
				Vertex3D { pos: Vector3::new(x_max, y_min, z_min), color: wall_color }, // 5
				Vertex3D { pos: Vector3::new(x_max, y_wall_top, z_min), color: wall_color }, // 6
				Vertex3D { pos: Vector3::new(x_min, y_wall_top, z_min), color: wall_color }, // 7

				// Roof edges (duplicate positions with roof color)
				Vertex3D { pos: Vector3::new(x_min, y_wall_top, z_max), color: roof_color }, // 8  (front-left eave)
				Vertex3D { pos: Vector3::new(x_max, y_wall_top, z_max), color: roof_color }, // 9  (front-right eave)
				Vertex3D { pos: Vector3::new(x_max, y_wall_top, z_min), color: roof_color }, // 10 (back-right eave)
				Vertex3D { pos: Vector3::new(x_min, y_wall_top, z_min), color: roof_color }, // 11 (back-left eave)
				Vertex3D { pos: Vector3::new(0.0, y_ridge, z_max), color: roof_color },     // 12 (front ridge)
				Vertex3D { pos: Vector3::new(0.0, y_ridge, z_min), color: roof_color },     // 13 (back ridge)

				// Gable wall ridge duplicates (same positions but wall color)
				Vertex3D { pos: Vector3::new(0.0, y_ridge, z_max), color: wall_color },     // 14 (front ridge - wall)
				Vertex3D { pos: Vector3::new(0.0, y_ridge, z_min), color: wall_color },     // 15 (back ridge - wall)

				// Door (slightly in front of the wall to avoid z-fighting)
				Vertex3D { pos: Vector3::new(-0.125, -0.5,  z_max + 0.001), color: door_color }, // 16 bl
				Vertex3D { pos: Vector3::new( 0.125, -0.5,  z_max + 0.001), color: door_color }, // 17 br
				Vertex3D { pos: Vector3::new( 0.125, -0.15, z_max + 0.001), color: door_color }, // 18 tr
				Vertex3D { pos: Vector3::new(-0.125, -0.15, z_max + 0.001), color: door_color }, // 19 tl

				// Left window
				Vertex3D { pos: Vector3::new(-0.425, -0.125, z_max + 0.002), color: window_color }, // 20 bl
				Vertex3D { pos: Vector3::new(-0.275, -0.125, z_max + 0.002), color: window_color }, // 21 br
				Vertex3D { pos: Vector3::new(-0.275,  0.025, z_max + 0.002), color: window_color }, // 22 tr
				Vertex3D { pos: Vector3::new(-0.425,  0.025, z_max + 0.002), color: window_color }, // 23 tl

				// Right window
				Vertex3D { pos: Vector3::new( 0.275, -0.125, z_max + 0.002), color: window_color }, // 24 bl
				Vertex3D { pos: Vector3::new( 0.425, -0.125, z_max + 0.002), color: window_color }, // 25 br
				Vertex3D { pos: Vector3::new( 0.425,  0.025, z_max + 0.002), color: window_color }, // 26 tr
				Vertex3D { pos: Vector3::new( 0.275,  0.025, z_max + 0.002), color: window_color }, // 27 tl
			];

			// Indices, clockwise winding where applicable
			let indices: Vec<u32> = vec![
				// Walls - Front
				0, 1, 2, 0, 2, 3,
				// Walls - Back
				4, 6, 5, 4, 7, 6,
				// Walls - Left
				0, 3, 7, 0, 7, 4,
				// Walls - Right
				1, 5, 6, 1, 6, 2,
				// Walls - Bottom
				0, 4, 5, 0, 5, 1,

				// Roof - Left slope (quad split)
				8, 12, 13, 8, 13, 11,
				// Roof - Right slope (quad split)
				9, 10, 13, 9, 13, 12,

				// Gable walls (triangles)
				3, 2, 14, // front gable
				7, 15, 6, // back gable

				// Door (front quads)
				16, 17, 18, 16, 18, 19,
				// Windows
				20, 21, 22, 20, 22, 23,
				24, 25, 26, 24, 26, 27,
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
