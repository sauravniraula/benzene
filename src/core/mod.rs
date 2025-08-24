pub mod engine;
pub mod scene;
pub mod camera;
pub mod primitives;
pub mod gpu;
pub mod model_push_constant;

pub use engine::GameEngine;

use crate::vulkan_backend::{backend::VBackend, vertex_input::Vertex3D};
use crate::core::gpu::model::Model;

pub trait ModelBuilder {
	fn geometry() -> (Vec<Vertex3D>, Vec<u32>);

	fn create_model(v_backend: &VBackend) -> Model {
		let (vertices, indices) = Self::geometry();
		Model::new(v_backend, &vertices, &indices)
	}
}
