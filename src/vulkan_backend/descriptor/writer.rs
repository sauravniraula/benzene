use ash::vk;

use crate::vulkan_backend::device::VDevice;

pub enum PendingDescriptorWrite {
    Buffer {
        set: vk::DescriptorSet,
        d_type: vk::DescriptorType,
        binding: u32,
        array_index: u32,
        buffer: vk::Buffer,
        offset: u32,
        range: vk::DeviceSize,
    },
    Image {
        set: vk::DescriptorSet,
        d_type: vk::DescriptorType,
        binding: u32,
        array_index: u32,
        view: vk::ImageView,
        sampler: vk::Sampler,
        layout: vk::ImageLayout,
    },
}

pub struct VDescriptorWriteBatch {
    pending: Vec<PendingDescriptorWrite>,
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
        set: vk::DescriptorSet,
        d_type: vk::DescriptorType,
        binding: u32,
        buffer: vk::Buffer,
        offset: u32,
        range: vk::DeviceSize,
    ) {
        self.pending.push(PendingDescriptorWrite::Buffer {
            set,
            d_type,
            binding,
            array_index: 0,
            buffer,
            offset,
            range,
        });
    }

    pub fn queue_image(
        &mut self,
        set: vk::DescriptorSet,
        d_type: vk::DescriptorType,
        binding: u32,
        view: vk::ImageView,
        sampler: vk::Sampler,
        layout: vk::ImageLayout,
    ) {
        self.pending.push(PendingDescriptorWrite::Image {
            set,
            d_type,
            binding,
            array_index: 0,
            view,
            sampler,
            layout,
        })
    }

    pub fn flush(self, v_device: &VDevice) {
        let mut buffer_infos: Vec<vk::DescriptorBufferInfo> = vec![];
        let mut image_infos: Vec<vk::DescriptorImageInfo> = vec![];
        let mut writes: Vec<vk::WriteDescriptorSet> = vec![];

        for each in self.pending.iter() {
            match each {
                PendingDescriptorWrite::Buffer {
                    buffer,
                    offset,
                    range,
                    ..
                } => {
                    buffer_infos.push(
                        vk::DescriptorBufferInfo::default()
                            .buffer(*buffer)
                            .offset(*offset as vk::DeviceSize)
                            .range(*range),
                    );
                }
                PendingDescriptorWrite::Image {
                    view,
                    sampler,
                    layout,
                    ..
                } => {
                    image_infos.push(
                        vk::DescriptorImageInfo::default()
                            .image_layout(*layout)
                            .image_view(*view)
                            .sampler(*sampler),
                    );
                }
            }
        }

        let mut buffer_idx: usize = 0;
        let mut image_idx: usize = 0;

        for each in self.pending {
            match each {
                PendingDescriptorWrite::Buffer {
                    set,
                    d_type,
                    binding,
                    array_index,
                    ..
                } => {
                    writes.push(
                        vk::WriteDescriptorSet::default()
                            .dst_set(set)
                            .dst_binding(binding)
                            .dst_array_element(array_index)
                            .descriptor_type(d_type)
                            .descriptor_count(1)
                            .buffer_info(&buffer_infos[buffer_idx..buffer_idx + 1]),
                    );
                    buffer_idx += 1;
                }
                PendingDescriptorWrite::Image {
                    set,
                    d_type,
                    binding,
                    array_index,
                    ..
                } => {
                    writes.push(
                        vk::WriteDescriptorSet::default()
                            .dst_set(set)
                            .dst_binding(binding)
                            .dst_array_element(array_index)
                            .descriptor_type(d_type)
                            .descriptor_count(1)
                            .image_info(&image_infos[image_idx..image_idx + 1]),
                    );
                    image_idx += 1;
                }
            }
        }

        unsafe {
            v_device.device.update_descriptor_sets(&writes, &[]);
        }
    }
}
