use ash::vk;

use crate::vulkan_backend::device::VDevice;

enum PendingWrite {
    Buffer {
        dst_set: vk::DescriptorSet,
        binding: u32,
        descriptor_type: vk::DescriptorType,
        buffer: vk::Buffer,
        range: vk::DeviceSize,
    },
    Image {
        dst_set: vk::DescriptorSet,
        binding: u32,
        descriptor_type: vk::DescriptorType,
        image_view: vk::ImageView,
        sampler: vk::Sampler,
        image_layout: vk::ImageLayout,
    },
}

pub struct VDescriptorWriteBatch {
    pending: Vec<PendingWrite>,
}

impl Default for VDescriptorWriteBatch {
    fn default() -> Self {
        Self {
            pending: Vec::new(),
        }
    }
}

impl VDescriptorWriteBatch {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn queue_buffer(
        &mut self,
        dst_set: vk::DescriptorSet,
        binding: u32,
        descriptor_type: vk::DescriptorType,
        buffer: vk::Buffer,
        range: vk::DeviceSize,
    ) {
        self.pending.push(PendingWrite::Buffer {
            dst_set,
            binding,
            descriptor_type,
            buffer,
            range,
        });
    }

    pub fn queue_image(
        &mut self,
        dst_set: vk::DescriptorSet,
        binding: u32,
        descriptor_type: vk::DescriptorType,
        image_view: vk::ImageView,
        sampler: vk::Sampler,
        image_layout: vk::ImageLayout,
    ) {
        self.pending.push(PendingWrite::Image {
            dst_set,
            binding,
            descriptor_type,
            image_view,
            sampler,
            image_layout,
        });
    }

    pub fn flush(self, v_device: &VDevice) {
        struct WritePlan {
            dst_set: vk::DescriptorSet,
            binding: u32,
            descriptor_type: vk::DescriptorType,
            is_buffer: bool,
            info_index: usize,
        }

        let (buffer_count, image_count) =
            self.pending
                .iter()
                .fold((0usize, 0usize), |acc, p| match p {
                    PendingWrite::Buffer { .. } => (acc.0 + 1, acc.1),
                    PendingWrite::Image { .. } => (acc.0, acc.1 + 1),
                });

        let mut buffer_infos: Vec<vk::DescriptorBufferInfo> = Vec::with_capacity(buffer_count);
        let mut image_infos: Vec<vk::DescriptorImageInfo> = Vec::with_capacity(image_count);
        let mut plans: Vec<WritePlan> = Vec::with_capacity(self.pending.len());

        for p in &self.pending {
            match *p {
                PendingWrite::Buffer {
                    dst_set,
                    binding,
                    descriptor_type,
                    buffer,
                    range,
                } => {
                    let idx = buffer_infos.len();
                    buffer_infos.push(
                        vk::DescriptorBufferInfo::default()
                            .buffer(buffer)
                            .offset(0)
                            .range(range),
                    );
                    plans.push(WritePlan {
                        dst_set,
                        binding,
                        descriptor_type,
                        is_buffer: true,
                        info_index: idx,
                    });
                }
                PendingWrite::Image {
                    dst_set,
                    binding,
                    descriptor_type,
                    image_view,
                    sampler,
                    image_layout,
                } => {
                    let idx = image_infos.len();
                    image_infos.push(
                        vk::DescriptorImageInfo::default()
                            .image_layout(image_layout)
                            .image_view(image_view)
                            .sampler(sampler),
                    );
                    plans.push(WritePlan {
                        dst_set,
                        binding,
                        descriptor_type,
                        is_buffer: false,
                        info_index: idx,
                    });
                }
            }
        }

        let mut writes: Vec<vk::WriteDescriptorSet> = Vec::with_capacity(plans.len());
        for plan in plans {
            if plan.is_buffer {
                writes.push(
                    vk::WriteDescriptorSet::default()
                        .dst_set(plan.dst_set)
                        .dst_binding(plan.binding)
                        .dst_array_element(0)
                        .descriptor_type(plan.descriptor_type)
                        .descriptor_count(1)
                        .buffer_info(&buffer_infos[plan.info_index..plan.info_index + 1]),
                );
            } else {
                writes.push(
                    vk::WriteDescriptorSet::default()
                        .dst_set(plan.dst_set)
                        .dst_binding(plan.binding)
                        .dst_array_element(0)
                        .descriptor_type(plan.descriptor_type)
                        .descriptor_count(1)
                        .image_info(&image_infos[plan.info_index..plan.info_index + 1]),
                );
            }
        }

        unsafe { v_device.device.update_descriptor_sets(&writes, &[]) };
    }
}
