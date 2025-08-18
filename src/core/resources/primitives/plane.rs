use nalgebra::Vector3;

use crate::vulkan_backend::vertex_input::Vertex3D;
use super::ModelBuilder;

pub struct Plane;

impl ModelBuilder for Plane {
    fn geometry() -> (Vec<Vertex3D>, Vec<u32>) {
        let color = Vector3::new(0.1, 0.1, 0.1);

        let vertices: Vec<Vertex3D> = vec![
            Vertex3D {
                pos: Vector3::new(10.0, 0.0, 10.0),
                color,
            },
            Vertex3D {
                pos: Vector3::new(10.0, 0.0, -10.0),
                color,
            },
            Vertex3D {
                pos: Vector3::new(-10.0, 0.0, 10.0),
                color,
            },
            Vertex3D {
                pos: Vector3::new(-10.0, 0.0, -10.0),
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
}
