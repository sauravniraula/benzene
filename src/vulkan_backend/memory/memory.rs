use ash::vk;

use crate::vulkan_backend::{
    device::{VDevice, VPhysicalDevice},
    memory::VMemoryManager,
};

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
        v_memory_manager: &VMemoryManager,
        v_physical_device: &VPhysicalDevice,
        v_device: &VDevice,
        requirements: &vk::MemoryRequirements,
        properties: vk::MemoryPropertyFlags,
    ) -> Self {
        let memory = v_memory_manager.allocate_memory(
            v_physical_device,
            v_device,
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

    pub fn map(&mut self, v_device: &VDevice, v_memory_manager: &VMemoryManager, size: u64) -> VMemoryState {
        match self.state {
            VMemoryState::UNMAPPED => {
                let mapped_at = v_memory_manager.map_memory(v_device, self.memory, size);
                self.state = VMemoryState::MAPPED(mapped_at);
                VMemoryState::MAPPED(mapped_at)
            }
            VMemoryState::MAPPED(addr) => VMemoryState::MAPPED(addr),
        }
    }

    pub fn unmap(&mut self, v_device: &VDevice, v_memory_manager: &VMemoryManager) -> VMemoryState {
        match self.state {
            VMemoryState::UNMAPPED => VMemoryState::UNMAPPED,
            VMemoryState::MAPPED(_) => {
                v_memory_manager.unmap_memory(v_device, self.memory);
                self.state = VMemoryState::UNMAPPED;
                VMemoryState::UNMAPPED
            }
        }
    }

    pub fn free(&self, v_device: &VDevice, v_memory_manager: &VMemoryManager) {
        v_memory_manager.free_memory(v_device, self.memory);
    }
}
