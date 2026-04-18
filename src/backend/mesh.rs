use std::sync::Arc;

use crate::backend::{buffer::create_buffer, vcontext::Vcontext, vertex_3d::Vertex3D};

pub struct Mesh {
    pub vertices: Vec<Vertex3D>,
    pub vertex_buffer: ash::vk::Buffer,
    pub vertex_memory: ash::vk::DeviceMemory,
}

impl Mesh {
    pub fn new(vcontext: &Arc<Vcontext>, vertices: Vec<Vertex3D>) -> Self {
        let vertices_len = vertices.len() as u64;
        let vertex_size = vertices_len * Vertex3D::size();

        let (vertex_buffer, vertex_memory) = create_buffer(
            &vcontext,
            vertex_size,
            ash::vk::BufferUsageFlags::VERTEX_BUFFER,
            ash::vk::MemoryPropertyFlags::DEVICE_LOCAL,
        );

        unsafe {
            let mapped = vcontext
                .device
                .map_memory(
                    vertex_memory,
                    0,
                    vertex_size,
                    ash::vk::MemoryMapFlags::empty(),
                )
                .expect("unable to map memory") as *mut u8;

            std::ptr::copy_nonoverlapping(
                vertices.as_ptr() as *const u8,
                mapped,
                vertex_size as usize,
            );
            vcontext.device.unmap_memory(vertex_memory);
        }

        Self {
            vertices,
            vertex_buffer,
            vertex_memory,
        }
    }

    pub fn drop(&self, vcontext: &Arc<Vcontext>) {
        let device = &vcontext.device;
        unsafe {
            let _ = device.device_wait_idle();
            device.free_memory(self.vertex_memory, None);
            device.destroy_buffer(self.vertex_buffer, None);
        }
    }
}
