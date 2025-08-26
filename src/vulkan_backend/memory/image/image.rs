use crate::vulkan_backend::{
    device::{VDevice, VPhysicalDevice},
    memory::{VBuffer, VBufferConfig, VMemory, VMemoryManager},
};
use crate::vulkan_backend::memory::image::config::VImageConfig;
use ash::vk;

pub enum VImageOwnership {
    Owned,
    External,
}

pub struct VImage {
    pub image: vk::Image,
    pub v_memory: Option<VMemory>,
    pub memory_requirements: Option<vk::MemoryRequirements>,
    pub config: VImageConfig,
    pub ownership: VImageOwnership,
}

impl VImage {
    pub fn new(
        v_device: &VDevice,
        v_physical_device: &VPhysicalDevice,
        v_memory_manager: &VMemoryManager,
        config: VImageConfig,
    ) -> Self {
        assert!(
            config.sharing_mode != vk::SharingMode::CONCURRENT || config.queue_families.is_some(),
            "Queue families must be provided on CONCURRENT Sharing Mode"
        );
        let mut image_info = vk::ImageCreateInfo::default()
            .image_type(vk::ImageType::TYPE_2D)
            .extent(config.extent)
            .mip_levels(config.mip_levels)
            .array_layers(config.array_layers)
            .format(config.format)
            .tiling(config.tiling)
            .initial_layout(config.initial_layout)
            .usage(config.usage)
            .sharing_mode(config.sharing_mode)
            .samples(config.samples);

        let queue_families: Vec<u32>;
        if config.sharing_mode == vk::SharingMode::CONCURRENT {
            queue_families = config.queue_families.clone().unwrap();
            image_info = image_info.queue_family_indices(&queue_families);
        }

        let image = unsafe { v_device.device.create_image(&image_info, None).expect("failed to create image") };

        let memory_requirements = unsafe { v_device.device.get_image_memory_requirements(image) };

        let v_memory = VMemory::new(
            v_memory_manager,
            v_physical_device,
            v_device,
            &memory_requirements,
            config.memory_property,
        );

        unsafe { v_device.device.bind_image_memory(image, v_memory.memory, 0).expect("failed to bind buffer memory") };

        Self {
            image,
            v_memory: Some(v_memory),
            memory_requirements: Some(memory_requirements),
            config,
            ownership: VImageOwnership::Owned,
        }
    }

    pub fn from_external(
        image: vk::Image,
        config: VImageConfig,
    ) -> Self {
        Self {
            image,
            v_memory: None,
            memory_requirements: None,
            config,
            ownership: VImageOwnership::External,
        }
    }

    pub fn copy_to_image(
        &self,
        v_device: &VDevice,
        v_physical_device: &VPhysicalDevice,
        v_memory_manager: &VMemoryManager,
        data: *const u8,
        size: u64,
    ) {
        let mut staging_buffer = VBuffer::new(
            v_device,
            v_physical_device,
            v_memory_manager,
            VBufferConfig {
                size,
                usage: vk::BufferUsageFlags::TRANSFER_SRC,
                sharing_mode: vk::SharingMode::EXCLUSIVE,
                queue_families: None,
                memory_property: vk::MemoryPropertyFlags::HOST_VISIBLE
                    | vk::MemoryPropertyFlags::HOST_COHERENT,
            },
        );
        staging_buffer.copy_to_buffer(v_device, v_physical_device, v_memory_manager, 0, size, data);
        v_memory_manager.run_single_cmd_submit(v_device, true, |cmd| {
            self.transition_layout(
                v_device,
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
                v_device
                    .device
                    .cmd_copy_buffer_to_image(
                        cmd,
                        staging_buffer.buffer,
                        self.image,
                        vk::ImageLayout::TRANSFER_DST_OPTIMAL,
                        &[region],
                    )
            };

            self.transition_layout(
                v_device,
                cmd,
                vk::ImageLayout::TRANSFER_DST_OPTIMAL,
                vk::ImageLayout::SHADER_READ_ONLY_OPTIMAL,
                vk::PipelineStageFlags::TRANSFER,
                vk::PipelineStageFlags::FRAGMENT_SHADER,
                vk::AccessFlags::TRANSFER_WRITE,
                vk::AccessFlags::SHADER_READ,
            );
        });

        staging_buffer.destroy(v_device, v_memory_manager);
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

    pub fn destroy(&self, v_device: &VDevice, v_memory_manager: &VMemoryManager) {
        unsafe {
            match self.ownership {
                VImageOwnership::Owned => {
                    if let Some(v_memory) = &self.v_memory {
                        v_memory.free(v_device, v_memory_manager);
                    }
                    v_device.device.destroy_image(self.image, None);
                }
                VImageOwnership::External => {}
            }
        }
    }
}
