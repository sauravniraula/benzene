use nalgebra::{Vector2, Vector3};

use super::ModelBuilder;
use crate::vulkan_backend::vertex_input::Vertex3D;

pub struct Plane;

impl ModelBuilder for Plane {
    fn geometry() -> (Vec<Vertex3D>, Vec<u32>) {
        let color = Vector3::new(1.0, 1.0, 1.0);

        let vertices: Vec<Vertex3D> = vec![
            Vertex3D {
                pos: Vector3::new(-10.0, 0.0, -10.0),
                color,
                tex_coord: Vector2::<f32>::new(0.0, 0.0),
            },
            Vertex3D {
                pos: Vector3::new(10.0, 0.0, -10.0),
                color,
                tex_coord: Vector2::<f32>::new(0.0, 1.0),
            },
            Vertex3D {
                pos: Vector3::new(10.0, 0.0, 10.0),
                color,
                tex_coord: Vector2::<f32>::new(1.0, 1.0),
            },
            Vertex3D {
                pos: Vector3::new(-10.0, 0.0, 10.0),
                color,
                tex_coord: Vector2::<f32>::new(1.0, 0.0),
            },
            Vertex3D {
                pos: Vector3::new(-10.0, 10.0, -10.0),
                color,
                tex_coord: Vector2::<f32>::new(0.0, 0.0),
            },
            Vertex3D {
                pos: Vector3::new(10.0, 10.0, -10.0),
                color,
                tex_coord: Vector2::<f32>::new(0.0, 1.0),
            },
            Vertex3D {
                pos: Vector3::new(10.0, 10.0, 10.0),
                color,
                tex_coord: Vector2::<f32>::new(1.0, 1.0),
            },
            Vertex3D {
                pos: Vector3::new(-10.0, 10.0, 10.0),
                color,
                tex_coord: Vector2::<f32>::new(1.0, 0.0),
            },
        ];

        let indices: Vec<u32> = vec![0, 1, 2, 2, 3, 0, 4, 5, 6, 6, 7, 4];

        (vertices, indices)
    }
}
