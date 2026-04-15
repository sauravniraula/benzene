pub fn create_image_views(
    device: &ash::Device,
    images: &Vec<ash::vk::Image>,
    format: ash::vk::Format,
    subresource_range: ash::vk::ImageSubresourceRange,
) -> Vec<ash::vk::ImageView> {
    let image_views: Vec<_> = images
        .iter()
        .map(|&e| {
            let create_info = ash::vk::ImageViewCreateInfo::default()
                .image(e)
                .view_type(ash::vk::ImageViewType::TYPE_2D)
                .format(format)
                .subresource_range(subresource_range);

            unsafe {
                device
                    .create_image_view(&create_info, None)
                    .expect("unable to create image view")
            }
        })
        .collect();
    image_views
}

pub fn transition_image_layout(
    device: &ash::Device,
    command_buffer: ash::vk::CommandBuffer,
    image: ash::vk::Image,
    old_layout: ash::vk::ImageLayout,
    new_layout: ash::vk::ImageLayout,
    src_access_mask: ash::vk::AccessFlags2,
    dst_access_mask: ash::vk::AccessFlags2,
    src_stage_mask: ash::vk::PipelineStageFlags2,
    dst_stage_mask: ash::vk::PipelineStageFlags2,
) {
    let barriers = [ash::vk::ImageMemoryBarrier2::default()
        .image(image)
        .src_access_mask(src_access_mask)
        .dst_access_mask(dst_access_mask)
        .src_stage_mask(src_stage_mask)
        .dst_stage_mask(dst_stage_mask)
        .old_layout(old_layout)
        .new_layout(new_layout)
        .subresource_range(
            ash::vk::ImageSubresourceRange::default()
                .aspect_mask(ash::vk::ImageAspectFlags::COLOR)
                .layer_count(1)
                .level_count(1),
        )];
    let info = ash::vk::DependencyInfo::default().image_memory_barriers(&barriers);
    unsafe { device.cmd_pipeline_barrier2(command_buffer, &info) };
}
