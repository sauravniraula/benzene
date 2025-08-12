use ash::vk;
use nalgebra::{Vector2, Vector3};

use crate::core::{
    backend::VBackend,
    memory::{VBuffer, VBufferConfig},
    vertex_input::Vertex2D,
};

pub struct VModel {
    pub vertices: Vec<Vertex2D>,
    pub v_buffer: VBuffer,
    pub indices: Vec<u32>,
    pub i_buffer: VBuffer,
}

impl VModel {
    pub fn new(v_backend: &VBackend) -> Self {
        let vertices: Vec<Vertex2D> = vec![
            Vertex2D {
                pos: Vector2::new(-0.5, -0.5),
                color: Vector3::new(1.0, 0.0, 0.0),
            },
            Vertex2D {
                pos: Vector2::new(0.5, -0.5),
                color: Vector3::new(0.0, 0.0, 1.0),
            },
            Vertex2D {
                pos: Vector2::new(0.5, 0.5),
                color: Vector3::new(0.0, 1.0, 0.0),
            },
            Vertex2D {
                pos: Vector2::new(-0.5, 0.5),
                color: Vector3::new(0.0, 0.0, 1.0),
            },
        ];
        let indices: Vec<u32> = vec![0, 2, 3, 0, 1, 2];

        // Vertex Buffer
        let v_buffer = VBuffer::new(
            v_backend,
            VBufferConfig {
                size: (size_of::<Vertex2D>() * vertices.len()) as u64,
                usage: vk::BufferUsageFlags::TRANSFER_DST | vk::BufferUsageFlags::VERTEX_BUFFER,
                sharing_mode: v_backend.v_device.buffer_sharing_mode,
                queue_families: Some(v_backend.v_device.buffer_queue_family_indices.clone()),
                memory_property: vk::MemoryPropertyFlags::DEVICE_LOCAL,
            },
        );
        let vertices_data_ptr = vertices.as_ptr() as *const u8;
        v_buffer.copy_to_buffer(
            v_backend,
            vertices_data_ptr,
            v_buffer.memory_requirements.size,
        );

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
        i_buffer.copy_to_buffer(
            v_backend,
            indices_data_ptr,
            i_buffer.memory_requirements.size,
        );

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
