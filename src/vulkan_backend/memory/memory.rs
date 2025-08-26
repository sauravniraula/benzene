use ash::vk;

use crate::vulkan_backend::{
    device::{VDevice, VPhysicalDevice},
    memory::VMemoryManager,
};

#[derive(Clone)]
pub enum VMemoryState {
    UNMAPPED,
    MAPPED(u64, u64, *mut u8),
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

    pub fn map(
        &mut self,
        v_device: &VDevice,
        v_memory_manager: &VMemoryManager,
        offset: u64,
        size: u64,
    ) -> VMemoryState {
        match self.state {
            VMemoryState::UNMAPPED => {
                let mapped_at = v_memory_manager.map_memory(v_device, self.memory, offset, size);
                self.state = VMemoryState::MAPPED(offset, size, mapped_at);
                VMemoryState::MAPPED(offset, size, mapped_at)
            }
            VMemoryState::MAPPED(_offset, _size, _) => {
                if _offset == offset && _size == size {
                    return self.state.clone();
                }
                self.unmap(v_device, v_memory_manager);
                self.map(v_device, v_memory_manager, offset, size)
            },
        }
    }

    pub fn unmap(&mut self, v_device: &VDevice, v_memory_manager: &VMemoryManager) -> VMemoryState {
        match self.state {
            VMemoryState::UNMAPPED => VMemoryState::UNMAPPED,
            VMemoryState::MAPPED(_, __, ___) => {
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
