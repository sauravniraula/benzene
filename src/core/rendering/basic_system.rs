use ash::vk;
use std::marker::PhantomData;

use crate::core::{
    device::VDevice,
    pipeline::{VPipelineInfo, VPipelineInfoConfig},
    rendering::{VRenderingSystem, VRenderingSystemConfig},
    swapchain::VSwapchain,
    vertex_input::{Vertex, bindable::BindableVertexInput},
};

pub struct BasicRenderingSystem<T: BindableVertexInput> {
    v_rendering_system: VRenderingSystem,
    _marker: PhantomData<T>,
}

impl<T> BasicRenderingSystem<T>
where
    T: BindableVertexInput,
{
    pub fn new(v_device: &VDevice, v_swapchain: &VSwapchain) -> Self {
        let v_pipeline_info = VPipelineInfo::new(&v_device, VPipelineInfoConfig::default());

        let v_rendering_system = VRenderingSystem::new(
            v_device,
            v_swapchain,
            VRenderingSystemConfig {
                pipeline_infos: vec![v_pipeline_info],
                binding_descriptions: T::get_binding_descriptions(),
                attribute_descriptions: T::get_attribute_descriptions(),
            },
        );

        Self {
            v_rendering_system,
            _marker: PhantomData,
        }
    }

    pub fn render(
        &self,
        v_device: &VDevice,
        command_buffer: vk::CommandBuffer,
        image_index: usize,
        data: &Vec<T>,
    ) {
        self.v_rendering_system
            .start(v_device, command_buffer, image_index);

        self.v_rendering_system.end(v_device, command_buffer);
    }
}
