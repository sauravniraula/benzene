use crate::core::ModelBuilder;
use crate::vulkan_backend::vertex_input::Vertex3D;

pub struct Plane;

impl ModelBuilder for Plane {
    fn geometry() -> (Vec<Vertex3D>, Vec<u32>) {
        let color = [1.0, 1.0, 1.0];

        let vertices: Vec<Vertex3D> = vec![
            Vertex3D {
                pos: [-2.0, 2.0, 0.0],
                color,
                normal: [0.0, 1.0, 0.0],
            },
            Vertex3D {
                pos: [2.0, -2.0, 0.0],
                color,
                normal: [0.0, 1.0, 0.0],
            },
            Vertex3D {
                pos: [-2.0, -2.0, 0.0],
                color,
                normal: [0.0, 1.0, 0.0],
            },
        ];

        let indices: Vec<u32> = vec![0, 2, 1];

        (vertices, indices)
    }
}
