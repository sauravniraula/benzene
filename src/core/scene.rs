use crate::{
    constants::MAX_FRAMES_IN_FLIGHT,
    core::game_objects::camera::Camera,
    core::game_objects::primitive_cube::PrimitiveCube,
    core::model::VModel,
    vulkan_backend::{
        backend::VBackend,
        descriptor::{VDescriptorLayout, VDescriptorPool, VDescriptorSets},
        device::VDevice,
        rendering::{Drawable, Recordable},
    },
};
use ash::vk;

pub struct Scene {
    v_descriptor_pool: VDescriptorPool,
    camera: Camera,
    models: Vec<VModel>,
}

impl Scene {
    pub fn new(v_backend: &VBackend) -> Self {
        // Descriptor Pool
        let v_descriptor_pool = VDescriptorPool::new(&v_backend.v_device, MAX_FRAMES_IN_FLIGHT);

        // Camera
        let layout = v_backend
            .basic_rendering_system
            .get_descriptor_set_layout_at_binding(0);
        let camera = Camera::new(v_backend, &v_descriptor_pool, layout);
        let (vertices, indices) = PrimitiveCube::geometry();
        let models = vec![VModel::new(v_backend, &vertices, &indices)];

        Self {
            v_descriptor_pool,
            camera,
            models,
        }
    }

    pub fn allocate_uniform_sets(
        &self,
        v_backend: &VBackend,
        layout: &VDescriptorLayout,
        count: usize,
    ) -> VDescriptorSets {
        VDescriptorSets::new(&v_backend.v_device, &self.v_descriptor_pool, layout, count)
    }

    pub fn update(&self, frame_index: usize, image_extent: vk::Extent2D) {
        self.camera.update_uniform_buffer(frame_index, image_extent);
    }

    pub fn destroy(&self, v_backend: &VBackend) {
        self.camera.destroy(v_backend);
        for each in self.models.iter() {
            each.destroy(v_backend);
        }
        self.v_descriptor_pool.destroy(&v_backend.v_device);
    }
}

impl Recordable for Scene {
    fn record(
        &self,
        v_device: &VDevice,
        command_buffer: vk::CommandBuffer,
        frame_index: usize,
        pipeline_layouts: &[vk::PipelineLayout],
    ) {
        unsafe {
            // Bind descriptor set for camera for this frame
            v_device.device.cmd_bind_descriptor_sets(
                command_buffer,
                vk::PipelineBindPoint::GRAPHICS,
                pipeline_layouts[0],
                0,
                &[self.camera.descriptor_set(frame_index)],
                &[],
            );

            for model in self.models.iter() {
                model.draw(v_device, command_buffer);
            }
        }
    }
}
