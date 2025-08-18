use crate::vulkan_backend::{
    backend::VBackend,
    memory::{VAllocateMemoryConfig, VBuffer, VBufferConfig, config::VImageConfig},
};
use ash::vk;

pub struct VImage {
    pub image: vk::Image,
    pub memory: vk::DeviceMemory,
    pub memory_requirements: vk::MemoryRequirements,
    pub config: VImageConfig,
}

impl VImage {
    pub fn new(v_backend: &VBackend, config: VImageConfig) -> Self {
        assert!(
            config.sharing_mode != vk::SharingMode::CONCURRENT || config.queue_families.is_some(),
            "Queue families must be provided on CONCURRENT Sharing Mode"
        );
        let mut image_info = vk::ImageCreateInfo::default()
            .image_type(vk::ImageType::TYPE_2D)
            .extent(config.extent)
            .mip_levels(1)
            .array_layers(1)
            .format(vk::Format::R8G8B8A8_SRGB)
            .tiling(vk::ImageTiling::OPTIMAL)
            .initial_layout(vk::ImageLayout::UNDEFINED)
            .usage(config.usage)
            .sharing_mode(config.sharing_mode)
            .samples(vk::SampleCountFlags::TYPE_1);

        let queue_families: Vec<u32>;
        if config.sharing_mode == vk::SharingMode::CONCURRENT {
            queue_families = config.queue_families.clone().unwrap();
            image_info = image_info.queue_family_indices(&queue_families);
        }

        let image = unsafe {
            v_backend
                .v_device
                .device
                .create_image(&image_info, None)
                .expect("failed to create image")
        };

        let memory_requirements = unsafe {
            v_backend
                .v_device
                .device
                .get_image_memory_requirements(image)
        };

        let memory = v_backend.v_memory_manager.allocate_memory(
            &v_backend.v_physical_device,
            &v_backend.v_device,
            VAllocateMemoryConfig {
                size: memory_requirements.size,
                memory_type: memory_requirements.memory_type_bits,
                properties: config.memory_property,
            },
        );

        unsafe {
            v_backend
                .v_device
                .device
                .bind_image_memory(image, memory, 0)
                .expect("failed to bind buffer memory")
        };

        Self {
            image,
            memory,
            memory_requirements,
            config,
        }
    }

    pub fn copy_to_image(&self, v_backend: &VBackend, data: *const u8, size: u64) {
        let staging_buffer = VBuffer::new(
            v_backend,
            VBufferConfig {
                size,
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
            .run_single_cmd_submit(&v_backend.v_device, |cmd| {
                // TODO: Image Layout transition and Buffer Copy
            });

        staging_buffer.destroy(v_backend);
    }

    pub fn destroy(&self, v_backend: &VBackend) {
        unsafe { v_backend.v_device.device.destroy_image(self.image, None) };
    }
}
