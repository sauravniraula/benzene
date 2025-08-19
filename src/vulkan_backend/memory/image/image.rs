use crate::vulkan_backend::{
    backend::VBackend,
    device::VDevice,
    memory::{VBuffer, VBufferConfig, VMemory, config::VImageConfig},
};
use ash::vk;

pub struct VImage {
    pub image: vk::Image,
    pub v_memory: VMemory,
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

        let v_memory = VMemory::new(v_backend, &memory_requirements, config.memory_property);

        unsafe {
            v_backend
                .v_device
                .device
                .bind_image_memory(image, v_memory.memory, 0)
                .expect("failed to bind buffer memory")
        };

        Self {
            image,
            v_memory,
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
            .run_single_cmd_submit(&v_backend.v_device, true, |cmd| {
                self.transition_layout(
                    &v_backend.v_device,
                    cmd,
                    vk::ImageLayout::UNDEFINED,
                    vk::ImageLayout::TRANSFER_DST_OPTIMAL,
                    vk::PipelineStageFlags::TOP_OF_PIPE,
                    vk::PipelineStageFlags::TRANSFER,
                    vk::AccessFlags::empty(),
                    vk::AccessFlags::TRANSFER_WRITE,
                );

                let subresource_layer = vk::ImageSubresourceLayers::default()
                    .aspect_mask(vk::ImageAspectFlags::COLOR)
                    .mip_level(0)
                    .base_array_layer(0)
                    .layer_count(1);

                let region = vk::BufferImageCopy::default()
                    .buffer_offset(0)
                    .buffer_row_length(0)
                    .buffer_image_height(0)
                    .image_offset(vk::Offset3D { x: 0, y: 0, z: 0 })
                    .image_extent(self.config.extent)
                    .image_subresource(subresource_layer);

                unsafe {
                    v_backend.v_device.device.cmd_copy_buffer_to_image(
                        cmd,
                        staging_buffer.buffer,
                        self.image,
                        vk::ImageLayout::TRANSFER_DST_OPTIMAL,
                        &[region],
                    )
                };

                self.transition_layout(
                    &v_backend.v_device,
                    cmd,
                    vk::ImageLayout::TRANSFER_DST_OPTIMAL,
                    vk::ImageLayout::SHADER_READ_ONLY_OPTIMAL,
                    vk::PipelineStageFlags::TRANSFER,
                    vk::PipelineStageFlags::FRAGMENT_SHADER,
                    vk::AccessFlags::TRANSFER_WRITE,
                    vk::AccessFlags::SHADER_READ,
                );
            });

        staging_buffer.destroy(v_backend);
    }

    pub fn transition_layout(
        &self,
        v_device: &VDevice,
        cmd: vk::CommandBuffer,
        old_layout: vk::ImageLayout,
        new_layout: vk::ImageLayout,
        src_stage: vk::PipelineStageFlags,
        dst_stage: vk::PipelineStageFlags,
        src_access: vk::AccessFlags,
        dst_access: vk::AccessFlags,
    ) {
        let subresource_range = vk::ImageSubresourceRange::default()
            .aspect_mask(vk::ImageAspectFlags::COLOR)
            .base_array_layer(0)
            .base_mip_level(0)
            .layer_count(1)
            .level_count(1);

        let barrier = vk::ImageMemoryBarrier::default()
            .image(self.image)
            .old_layout(old_layout)
            .new_layout(new_layout)
            .subresource_range(subresource_range)
            .src_access_mask(src_access)
            .dst_access_mask(dst_access)
            .src_queue_family_index(vk::QUEUE_FAMILY_IGNORED)
            .dst_queue_family_index(vk::QUEUE_FAMILY_IGNORED);

        unsafe {
            v_device.device.cmd_pipeline_barrier(
                cmd,
                src_stage,
                dst_stage,
                vk::DependencyFlags::empty(),
                &[],
                &[],
                &[barrier],
            )
        };
    }

    pub fn destroy(&self, v_backend: &VBackend) {
        unsafe {
            self.v_memory.free(v_backend);
            v_backend.v_device.device.destroy_image(self.image, None);
        }
    }
}


