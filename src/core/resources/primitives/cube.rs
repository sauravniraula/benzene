use nalgebra::Vector3;

use crate::vulkan_backend::vertex_input::Vertex3D;
use super::ModelBuilder;

pub struct Cube;

impl ModelBuilder for Cube {
    fn geometry() -> (Vec<Vertex3D>, Vec<u32>) {
        let half = 0.5f32;
        let color = Vector3::new(0.3, 0.3, 0.3);

        let vertices: Vec<Vertex3D> = vec![
            Vertex3D {
                pos: Vector3::new(-half, -half, half),
                color: color,
            },
            Vertex3D {
                pos: Vector3::new(half, -half, half),
                color: color,
            },
            Vertex3D {
                pos: Vector3::new(half, half, half),
                color: color,
            },
            Vertex3D {
                pos: Vector3::new(-half, half, half),
                color: color,
            },
            Vertex3D {
                pos: Vector3::new(-half, -half, -half),
                color: color,
            },
            Vertex3D {
                pos: Vector3::new(half, -half, -half),
                color: color,
            },
            Vertex3D {
                pos: Vector3::new(half, half, -half),
                color: color,
            },
            Vertex3D {
                pos: Vector3::new(-half, half, -half),
                color: color,
            },
            Vertex3D {
                pos: Vector3::new(-half, -half, half),
                color: color,
            },
            Vertex3D {
                pos: Vector3::new(-half, half, half),
                color: color,
            },
            Vertex3D {
                pos: Vector3::new(-half, half, -half),
                color: color,
            },
            Vertex3D {
                pos: Vector3::new(-half, -half, -half),
                color: color,
            },
            Vertex3D {
                pos: Vector3::new(half, -half, half),
                color: color,
            },
            Vertex3D {
                pos: Vector3::new(half, -half, -half),
                color: color,
            },
            Vertex3D {
                pos: Vector3::new(half, half, -half),
                color: color,
            },
            Vertex3D {
                pos: Vector3::new(half, half, half),
                color: color,
            },
            Vertex3D {
                pos: Vector3::new(-half, -half, half),
                color: color,
            },
            Vertex3D {
                pos: Vector3::new(-half, -half, -half),
                color: color,
            },
            Vertex3D {
                pos: Vector3::new(half, -half, -half),
                color: color,
            },
            Vertex3D {
                pos: Vector3::new(half, -half, half),
                color: color,
            },
            Vertex3D {
                pos: Vector3::new(-half, half, half),
                color: color,
            },
            Vertex3D {
                pos: Vector3::new(half, half, half),
                color: color,
            },
            Vertex3D {
                pos: Vector3::new(half, half, -half),
                color: color,
            },
            Vertex3D {
                pos: Vector3::new(-half, half, -half),
                color: color,
            },
        ];

        let indices: Vec<u32> = vec![
            0, 1, 2, 0, 2, 3, 4, 6, 5, 4, 7, 6, 8, 9, 10, 8, 10, 11, 12, 13, 14, 12, 14, 15, 16,
            17, 18, 16, 18, 19, 20, 21, 22, 20, 22, 23,
        ];

        (vertices, indices)
    }
}
