use ash::vk;

use crate::vulkan_backend::backend::VBackend;

pub enum VMemoryState {
    UNMAPPED,
    MAPPED(*mut u8),
}

pub struct VMemory {
    pub memory: vk::DeviceMemory,
    pub state: VMemoryState,
}

impl VMemory {
    pub fn new(
        v_backend: &VBackend,
        requirements: &vk::MemoryRequirements,
        properties: vk::MemoryPropertyFlags,
    ) -> Self {
        let memory = v_backend.v_memory_manager.allocate_memory(
            &v_backend.v_physical_device,
            &v_backend.v_device,
            super::VAllocateMemoryConfig {
                size: requirements.size,
                memory_type: requirements.memory_type_bits,
                properties,
            },
        );
        Self {
            memory,
            state: VMemoryState::UNMAPPED,
        }
    }

    pub fn map(&mut self, v_backend: &VBackend, size: u64) -> VMemoryState {
        match self.state {
            VMemoryState::UNMAPPED => {
                let mapped_at =
                    v_backend
                        .v_memory_manager
                        .map_memory(&v_backend.v_device, self.memory, size);
                self.state = VMemoryState::MAPPED(mapped_at);
                VMemoryState::MAPPED(mapped_at)
            }
            VMemoryState::MAPPED(addr) => VMemoryState::MAPPED(addr),
        }
    }

    pub fn unmap(&mut self, v_backend: &VBackend) -> VMemoryState {
        match self.state {
            VMemoryState::UNMAPPED => VMemoryState::UNMAPPED,
            VMemoryState::MAPPED(_) => {
                v_backend
                    .v_memory_manager
                    .unmap_memory(&v_backend.v_device, self.memory);
                self.state = VMemoryState::UNMAPPED;
                VMemoryState::UNMAPPED
            }
        }
    }

    pub fn free(&self, v_backend: &VBackend) {
        v_backend
            .v_memory_manager
            .free_memory(&v_backend.v_device, self.memory);
    }
}
