use std::{f32::consts::PI, sync::Arc};

use crate::{
    backend::{buffer::create_buffer, vcontext::Vcontext},
    core::transform::Transform,
};

pub struct CameraBufferObject {
    view: glam::Mat4,
    projection: glam::Mat4,
}

pub struct Camera {
    pub buffer: ash::vk::Buffer,
    memory: ash::vk::DeviceMemory,
    transform: Transform,
    projection: glam::Mat4,
}

impl Camera {
    pub fn new(vcontext: &Arc<Vcontext>) -> Self {
        let mut transform = Transform::new();

        let view = transform.get_transformation_matrix();
        let mut projection = glam::Mat4::perspective_rh(PI / 180.0 * 90.0, 1.0, 0.1, 10.0);
        projection.col_mut(1)[1] *= -1.0;

        let size = size_of::<CameraBufferObject>() as u64;
        let (buffer, memory) = create_buffer(
            vcontext,
            size,
            ash::vk::BufferUsageFlags::UNIFORM_BUFFER,
            ash::vk::MemoryPropertyFlags::HOST_VISIBLE
                | ash::vk::MemoryPropertyFlags::HOST_COHERENT,
        );
        let cbo = CameraBufferObject { view, projection };

        unsafe {
            let mapped = vcontext
                .device
                .map_memory(memory, 0, size, ash::vk::MemoryMapFlags::empty())
                .expect("unable to map memory") as *mut u8;

            std::ptr::copy_nonoverlapping(
                &cbo as *const CameraBufferObject as *const u8,
                mapped,
                size as usize,
            );
            vcontext.device.unmap_memory(memory);
        }

        Self {
            buffer,
            memory,
            transform,
            projection,
        }
    }
}
