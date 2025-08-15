use crate::{
    constants::MAX_FRAMES_IN_FLIGHT,
    core::game_objects::camera::Camera,
    vulkan_backend::{
        backend::VBackend,
        descriptor::{VDescriptorPool, VDescriptorSets},
    },
};

pub struct Scene {
    v_descriptor_pool: VDescriptorPool,
}

impl Scene {
    pub fn new(v_backend: &VBackend) -> Self {
        // Descriptor Pool
        let v_descriptor_pool = VDescriptorPool::new(&v_backend.v_device, MAX_FRAMES_IN_FLIGHT);

        // Camera
        let camera = Camera::new(v_backend, &v_descriptor_pool);

        Self { v_descriptor_pool }
    }
}
