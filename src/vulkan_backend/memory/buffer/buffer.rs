use ash::vk;

use crate::vulkan_backend::{
    device::{VDevice, VPhysicalDevice},
    memory::{VBufferConfig, VMemory, VMemoryManager, VMemoryState},
};

pub struct VBuffer {
    pub buffer: vk::Buffer,
    pub memory_requirements: vk::MemoryRequirements,
    pub v_memory: VMemory,
    pub config: VBufferConfig,
}

impl VBuffer {
    pub fn new(
        v_device: &VDevice,
        v_physical_device: &VPhysicalDevice,
        v_memory_manager: &VMemoryManager,
        config: VBufferConfig,
    ) -> Self {
        assert!(
            config.sharing_mode != vk::SharingMode::CONCURRENT || config.queue_families.is_some(),
            "Queue families must be provided on CONCURRENT Sharing Mode"
        );
        let mut buffer_info = vk::BufferCreateInfo::default()
            .size(config.size)
            .usage(config.usage)
            .sharing_mode(config.sharing_mode);

        let queue_families: Vec<u32>;
        if config.sharing_mode == vk::SharingMode::CONCURRENT {
            queue_families = config.queue_families.clone().unwrap();
            buffer_info = buffer_info.queue_family_indices(&queue_families);
        }

        let buffer = unsafe {
            v_device
                .device
                .create_buffer(&buffer_info, None)
                .expect("failed to create buffer")
        };

        let memory_requirements = unsafe { v_device.device.get_buffer_memory_requirements(buffer) };

        let v_memory = VMemory::new(
            v_memory_manager,
            v_physical_device,
            v_device,
            &memory_requirements,
            config.memory_property,
        );

        unsafe {
            v_device
                .device
                .bind_buffer_memory(buffer, v_memory.memory, 0)
                .expect("failed to bind buffer memory")
        };

        Self {
            buffer,
            memory_requirements,
            v_memory,
            config,
        }
    }

    pub fn is_host_visible(&self) -> bool {
        self.config
            .memory_property
            .contains(vk::MemoryPropertyFlags::HOST_VISIBLE)
    }

    pub fn copy_to_buffer(
        &mut self,
        v_device: &VDevice,
        v_physical_device: &VPhysicalDevice,
        v_memory_manager: &VMemoryManager,
        offset: u64,
        size: u64,
        data: *const u8,
    ) {
        if self.is_host_visible() {
            if let VMemoryState::MAPPED(_offset, _size, _dst) = self.v_memory.state {
                if _offset <= offset && _offset + _size >= offset + size {
                    unsafe {
                        std::ptr::copy_nonoverlapping(
                            data,
                            _dst.add((offset - _offset) as usize),
                            size as usize,
                        )
                    };
                    return;
                }
            }
            let mapped = self.v_memory.map(v_device, v_memory_manager, offset, size);
            if let VMemoryState::MAPPED(_, __, dst) = mapped {
                unsafe { std::ptr::copy_nonoverlapping(data, dst, size as usize) };
            }
            return;
        }
        let mut staging_buffer = VBuffer::new(
            v_device,
            v_physical_device,
            v_memory_manager,
            VBufferConfig {
                size: size,
                usage: vk::BufferUsageFlags::TRANSFER_SRC,
                sharing_mode: vk::SharingMode::EXCLUSIVE,
                queue_families: None,
                memory_property: vk::MemoryPropertyFlags::HOST_VISIBLE
                    | vk::MemoryPropertyFlags::HOST_COHERENT,
            },
        );
        staging_buffer.copy_to_buffer(
            v_device,
            v_physical_device,
            v_memory_manager,
            offset,
            size,
            data,
        );
        v_memory_manager.run_single_cmd_submit(v_device, false, |cmd| {
            let copy_regions = [vk::BufferCopy::default().dst_offset(offset).size(size)];
            unsafe {
                v_device.device.cmd_copy_buffer(
                    cmd,
                    staging_buffer.buffer,
                    self.buffer,
                    &copy_regions,
                );
            }
        });
        staging_buffer.destroy(v_device, v_memory_manager);
    }

    pub fn destroy(&self, v_device: &VDevice, v_memory_manager: &VMemoryManager) {
        self.v_memory.free(v_device, v_memory_manager);
        unsafe {
            v_device.device.destroy_buffer(self.buffer, None);
        }
    }
}
