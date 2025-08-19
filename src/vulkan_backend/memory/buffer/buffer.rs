use ash::vk;

use crate::vulkan_backend::{
    backend::VBackend,
    memory::{VBufferConfig, VMemory, VMemoryState},
};

pub struct VBuffer {
    pub buffer: vk::Buffer,
    pub memory_requirements: vk::MemoryRequirements,
    pub v_memory: VMemory,
    pub config: VBufferConfig,
}

impl VBuffer {
    pub fn new(v_backend: &VBackend, config: VBufferConfig) -> Self {
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
            v_backend
                .v_device
                .device
                .create_buffer(&buffer_info, None)
                .expect("failed to create buffer")
        };

        let memory_requirements = unsafe {
            v_backend
                .v_device
                .device
                .get_buffer_memory_requirements(buffer)
        };

        let v_memory = VMemory::new(v_backend, &memory_requirements, config.memory_property);

        unsafe {
            v_backend
                .v_device
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

    pub fn map_memory(&mut self, v_backend: &VBackend) -> VMemoryState {
        self.v_memory.map(v_backend, self.config.size)
    }

    pub fn unmap_memory(&mut self, v_backend: &VBackend) -> VMemoryState {
        self.v_memory.unmap(v_backend)
    }

    pub fn is_host_visible(&self) -> bool {
        self.config
            .memory_property
            .contains(vk::MemoryPropertyFlags::HOST_VISIBLE)
    }

    pub fn copy_to_buffer(&self, v_backend: &VBackend, data: *const u8, size: u64) {
        if self.is_host_visible() {
            v_backend.v_memory_manager.copy_data_to_memory(
                &v_backend.v_device,
                self.v_memory.memory,
                data,
                size,
            );
            return;
        }
        let staging_buffer = VBuffer::new(
            v_backend,
            VBufferConfig {
                size: size,
                usage: vk::BufferUsageFlags::TRANSFER_SRC,
                sharing_mode: vk::SharingMode::EXCLUSIVE,
                queue_families: None,
                memory_property: vk::MemoryPropertyFlags::HOST_VISIBLE
                    | vk::MemoryPropertyFlags::HOST_COHERENT,
            },
        );
        staging_buffer.copy_to_buffer(v_backend, data, size);
        v_backend
            .v_memory_manager
            .run_single_cmd_submit(&v_backend.v_device, false, |cmd| {
                let copy_regions = [vk::BufferCopy::default().size(size)];
                unsafe {
                    v_backend.v_device.device.cmd_copy_buffer(
                        cmd,
                        staging_buffer.buffer,
                        self.buffer,
                        &copy_regions,
                    );
                }
            });
        staging_buffer.destroy(v_backend);
    }

    pub fn destroy(&self, v_backend: &VBackend) {
        self.v_memory.free(v_backend);
        unsafe {
            v_backend.v_device.device.destroy_buffer(self.buffer, None);
        }
    }
}


