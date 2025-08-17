use nalgebra::Vector3;

use crate::{
    core::gpu::model::Model,
    vulkan_backend::{backend::VBackend, vertex_input::Vertex3D},
};

pub struct Plane;

impl Plane {
    pub fn geometry() -> (Vec<Vertex3D>, Vec<u32>) {
        let color = Vector3::new(0.2, 0.7, 0.2);

        let vertices: Vec<Vertex3D> = vec![
            Vertex3D {
                pos: Vector3::new(5.0, 0.0, 5.0),
                color,
            },
            Vertex3D {
                pos: Vector3::new(5.0, 0.0, -5.0),
                color,
            },
            Vertex3D {
                pos: Vector3::new(-5.0, 0.0, 5.0),
                color,
            },
            Vertex3D {
                pos: Vector3::new(-5.0, 0.0, -5.0),
                color,
            },
        ];

        // Two-sided by duplicating with flipped winding to keep BACK culling
        let indices: Vec<u32> = vec![
            0, 1, 2, 3, 2, 1, // top face (CCW when seen from above)
            2, 1, 0, 1, 2, 3, // bottom face (CCW when seen from below)
        ];

        (vertices, indices)
    }

    pub fn create_model(v_backend: &VBackend) -> Model {
        let (vertices, indices) = Self::geometry();
        Model::new(v_backend, &vertices, &indices)
    }
}
