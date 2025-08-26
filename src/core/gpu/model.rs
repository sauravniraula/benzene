use ash::vk;

use crate::{
    core::gpu::recordable::Drawable,
    vulkan_backend::{
        backend::VBackend,
        device::VDevice,
        memory::{VBuffer, VBufferConfig},
        vertex_input::Vertex3D,
    },
};

pub struct Model {
    pub v_buffer: VBuffer,
    pub i_buffer: VBuffer,
    pub index_count: u32,
}

impl Model {
    pub fn new(v_backend: &VBackend, vertices: &[Vertex3D], indices: &[u32]) -> Self {
        let mut v_buffer = VBuffer::new(
            &v_backend.v_device,
            &v_backend.v_physical_device,
            &v_backend.v_memory_manager,
            VBufferConfig {
                size: (size_of::<Vertex3D>() * vertices.len()) as u64,
                usage: vk::BufferUsageFlags::TRANSFER_DST | vk::BufferUsageFlags::VERTEX_BUFFER,
                sharing_mode: v_backend.v_device.buffer_sharing_mode,
                queue_families: Some(v_backend.v_device.buffer_queue_family_indices.clone()),
                memory_property: vk::MemoryPropertyFlags::DEVICE_LOCAL,
            },
        );
        let vertices_data_ptr = vertices.as_ptr() as *const u8;
        v_buffer.copy_to_buffer(
            &v_backend.v_device,
            &v_backend.v_physical_device,
            &v_backend.v_memory_manager,
            0,
            v_buffer.config.size,
            vertices_data_ptr,
        );

        let mut i_buffer = VBuffer::new(
            &v_backend.v_device,
            &v_backend.v_physical_device,
            &v_backend.v_memory_manager,
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
            &v_backend.v_device,
            &v_backend.v_physical_device,
            &v_backend.v_memory_manager,
            0,
            i_buffer.config.size,
            indices_data_ptr,
        );

        Self {
            v_buffer,
            i_buffer,
            index_count: indices.len() as u32,
        }
    }

    pub fn from_obj(v_backend: &VBackend, obj_path: &str) -> Self {
        let (models, _) =
            tobj::load_obj(obj_path, &tobj::GPU_LOAD_OPTIONS).expect("failed to load obj model");
        assert!(models.len() > 0, "No models found in obj file");

        let mesh = &models[0].mesh;
        let total_vertices = mesh.positions.len() / 3;

        let mut vertices: Vec<Vertex3D> = vec![];
        for i in 0..total_vertices {
            vertices.push(Vertex3D {
                pos: [
                    mesh.positions[3 * i],
                    mesh.positions[3 * i + 1],
                    mesh.positions[3 * i + 2],
                ],
                color: [1.0, 1.0, 1.0],
                normal: [
                    mesh.normals[3 * i],
                    mesh.normals[3 * i + 1],
                    mesh.normals[3 * i + 2],
                ],
                uv: [mesh.texcoords[2 * i], mesh.texcoords[2 * 1 + 1]],
            });
        }

        Self::new(v_backend, &vertices, &mesh.indices)
    }

    pub fn destroy(&self, v_backend: &VBackend) {
        self.i_buffer
            .destroy(&v_backend.v_device, &v_backend.v_memory_manager);
        self.v_buffer
            .destroy(&v_backend.v_device, &v_backend.v_memory_manager);
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
            v_device.device.cmd_bind_index_buffer(
                command_buffer,
                self.i_buffer.buffer,
                0,
                vk::IndexType::UINT32,
            );
            v_device
                .device
                .cmd_draw_indexed(command_buffer, self.index_count, 1, 0, 0, 0);
        }
    }
}
