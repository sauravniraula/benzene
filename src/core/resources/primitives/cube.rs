use nalgebra::Vector3;

use crate::{
    core::gpu::model::Model,
    vulkan_backend::{backend::VBackend, vertex_input::Vertex3D},
};

pub struct Cube;

impl Cube {
    pub fn geometry() -> (Vec<Vertex3D>, Vec<u32>) {
        let half = 0.5f32;
        let front_color = Vector3::new(1.0, 0.0, 0.0);
        let back_color = Vector3::new(0.0, 1.0, 0.0);
        let left_color = Vector3::new(0.0, 0.0, 1.0);
        let right_color = Vector3::new(1.0, 1.0, 0.0);
        let bottom_color = Vector3::new(1.0, 0.0, 1.0);
        let top_color = Vector3::new(0.0, 1.0, 1.0);

        let vertices: Vec<Vertex3D> = vec![
            Vertex3D { pos: Vector3::new(-half, -half,  half), color: front_color },
            Vertex3D { pos: Vector3::new( half, -half,  half), color: front_color },
            Vertex3D { pos: Vector3::new( half,  half,  half), color: front_color },
            Vertex3D { pos: Vector3::new(-half,  half,  half), color: front_color },
            Vertex3D { pos: Vector3::new(-half, -half, -half), color: back_color },
            Vertex3D { pos: Vector3::new( half, -half, -half), color: back_color },
            Vertex3D { pos: Vector3::new( half,  half, -half), color: back_color },
            Vertex3D { pos: Vector3::new(-half,  half, -half), color: back_color },
            Vertex3D { pos: Vector3::new(-half, -half,  half), color: left_color },
            Vertex3D { pos: Vector3::new(-half,  half,  half), color: left_color },
            Vertex3D { pos: Vector3::new(-half,  half, -half), color: left_color },
            Vertex3D { pos: Vector3::new(-half, -half, -half), color: left_color },
            Vertex3D { pos: Vector3::new( half, -half,  half), color: right_color },
            Vertex3D { pos: Vector3::new( half, -half, -half), color: right_color },
            Vertex3D { pos: Vector3::new( half,  half, -half), color: right_color },
            Vertex3D { pos: Vector3::new( half,  half,  half), color: right_color },
            Vertex3D { pos: Vector3::new(-half, -half,  half), color: bottom_color },
            Vertex3D { pos: Vector3::new(-half, -half, -half), color: bottom_color },
            Vertex3D { pos: Vector3::new( half, -half, -half), color: bottom_color },
            Vertex3D { pos: Vector3::new( half, -half,  half), color: bottom_color },
            Vertex3D { pos: Vector3::new(-half,  half,  half), color: top_color },
            Vertex3D { pos: Vector3::new( half,  half,  half), color: top_color },
            Vertex3D { pos: Vector3::new( half,  half, -half), color: top_color },
            Vertex3D { pos: Vector3::new(-half,  half, -half), color: top_color },
        ];

        let indices: Vec<u32> = vec![
            0, 1, 2, 0, 2, 3,
            4, 6, 5, 4, 7, 6,
            8, 9, 10, 8, 10, 11,
            12, 13, 14, 12, 14, 15,
            16, 17, 18, 16, 18, 19,
            20, 21, 22, 20, 22, 23,
        ];

        (vertices, indices)
    }

    pub fn create_model(v_backend: &VBackend) -> Model {
        let (vertices, indices) = Self::geometry();
        Model::new(v_backend, &vertices, &indices)
    }
}


