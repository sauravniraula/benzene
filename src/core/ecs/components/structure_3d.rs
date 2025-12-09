use crate::core::gpu::model::Model;
use crate::vulkan_backend::backend::VBackend;

pub struct Structure3D {
    pub model: Model,
}

impl Structure3D {
    pub fn new(model: Model) -> Self {
        Self { model }
    }

    pub fn from_obj(v_backend: &VBackend, obj_path: &str) -> Self {
        Self {
            model: Model::from_obj(v_backend, obj_path),
        }
    }

    pub fn destroy(&self, v_backend: &VBackend) {
        self.model.destroy(v_backend);
    }
}
