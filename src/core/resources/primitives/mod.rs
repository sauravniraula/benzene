pub mod cube;
pub mod plane;

use crate::core::gpu::model::Model;
use crate::vulkan_backend::backend::VBackend;
use crate::vulkan_backend::vertex_input::Vertex3D;

pub trait ModelBuilder {
    fn geometry() -> (Vec<Vertex3D>, Vec<u32>);

    fn create_model(v_backend: &VBackend) -> Model {
        let (vertices, indices) = Self::geometry();
        Model::new(v_backend, &vertices, &indices)
    }
}


