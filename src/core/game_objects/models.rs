use ash::vk;

use crate::core::{
    backend::VBackend,
    memory::{VBuffer, VBufferConfig},
    vertex_input::Vertex,
};

pub struct VModel {
    pub vertices: Vec<Vertex>,
    pub v_buffer: VBuffer,
}

impl VModel {
    pub fn new(v_backend: &VBackend) -> Self {
        let vertices: Vec<Vertex> = vec![
            Vertex {
                pos: [-0.5, -0.5],
                color: [1.0, 0.0, 0.0],
            },
            Vertex {
                pos: [0.5, 0.5],
                color: [0.0, 1.0, 0.0],
            },
            Vertex {
                pos: [-0.5, 0.5],
                color: [0.0, 0.0, 1.0],
            },
        ];

        let mut queue_families = vec![v_backend.v_device.graphics_queue_family_index];
        if !v_backend.v_device.is_graphics_and_transfer_queue_same {
            queue_families.push(v_backend.v_device.transfer_queue_family_index);
        }
        let v_buffer = VBuffer::new(
            v_backend,
            VBufferConfig {
                size: (size_of::<Vertex>() * vertices.len()) as u64,
                usage: vk::BufferUsageFlags::TRANSFER_DST | vk::BufferUsageFlags::VERTEX_BUFFER,
                sharing_mode: if v_backend.v_device.is_graphics_and_transfer_queue_same {
                    vk::SharingMode::EXCLUSIVE
                } else {
                    vk::SharingMode::CONCURRENT
                },
                queue_families: Some(queue_families),
                memory_property: vk::MemoryPropertyFlags::DEVICE_LOCAL,
            },
        );

        let vertices_data_ptr = vertices.as_ptr() as *const u8;
        v_buffer.copy_to_buffer(
            v_backend,
            vertices_data_ptr,
            size_of::<Vertex>() * vertices.len(),
        );

        Self { vertices, v_buffer }
    }

    pub fn destroy(&self, v_backend: &VBackend) {
        self.v_buffer.destroy(v_backend);
    }
}
