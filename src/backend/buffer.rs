use std::sync::Arc;

use crate::backend::{memory::find_memory_type_index, vcontext::Vcontext};

pub fn create_buffer(
    vcontext: &Arc<Vcontext>,
    size: u64,
    usage: ash::vk::BufferUsageFlags,
    properties: ash::vk::MemoryPropertyFlags,
) -> (ash::vk::Buffer, ash::vk::DeviceMemory) {
    let info = ash::vk::BufferCreateInfo::default()
        .size(size)
        .usage(usage)
        .sharing_mode(ash::vk::SharingMode::EXCLUSIVE);

    let buffer = unsafe {
        vcontext
            .device
            .create_buffer(&info, None)
            .expect("unable to create buffer")
    };
    let vmr = unsafe { vcontext.device.get_buffer_memory_requirements(buffer) };

    let mti = find_memory_type_index(
        &vcontext.memory_properties,
        vmr.memory_type_bits,
        properties,
    );

    let mai = ash::vk::MemoryAllocateInfo::default()
        .allocation_size(size)
        .memory_type_index(mti);

    let memory = unsafe {
        vcontext
            .device
            .allocate_memory(&mai, None)
            .expect("unable to allocate memory")
    };
    unsafe {
        vcontext
            .device
            .bind_buffer_memory(buffer, memory, 0)
            .expect("unable to bind buffer memory")
    };
    (buffer, memory)
}
