use std::sync::Arc;

use crate::{
    backend::{mesh::Mesh, render_loop::RenderContext, vcontext::Vcontext, vertex_3d::Vertex3D},
    render::geometry::RenderGeometry,
};

pub struct RenderOwner {
    vcontext: Arc<Vcontext>,
    render_geometry: RenderGeometry,
    a_mesh: Mesh,
    b_mesh: Mesh,
}

impl RenderOwner {
    pub fn new(vcontext: Arc<Vcontext>) -> Self {
        let render_geometry = RenderGeometry::new(vcontext.clone());

        let a_vertices = vec![
            Vertex3D {
                pos: glam::Vec3 {
                    x: -0.5,
                    y: 0.5,
                    z: 0.0,
                },
                color: glam::Vec3 {
                    x: 1.0,
                    y: 0.0,
                    z: 0.0,
                },
            },
            Vertex3D {
                pos: glam::Vec3 {
                    x: 0.0,
                    y: -0.5,
                    z: 0.0,
                },
                color: glam::Vec3 {
                    x: 0.0,
                    y: 1.0,
                    z: 0.0,
                },
            },
            Vertex3D {
                pos: glam::Vec3 {
                    x: 0.5,
                    y: 0.5,
                    z: 0.0,
                },
                color: glam::Vec3 {
                    x: 0.0,
                    y: 0.0,
                    z: 1.0,
                },
            },
        ];

        let b_vertices = vec![
            Vertex3D {
                pos: glam::Vec3 {
                    x: -0.5,
                    y: 0.2,
                    z: 1.0,
                },
                color: glam::Vec3 {
                    x: 1.0,
                    y: 0.0,
                    z: 0.0,
                },
            },
            Vertex3D {
                pos: glam::Vec3 {
                    x: 0.0,
                    y: -0.8,
                    z: 1.0,
                },
                color: glam::Vec3 {
                    x: 0.0,
                    y: 1.0,
                    z: 0.0,
                },
            },
            Vertex3D {
                pos: glam::Vec3 {
                    x: 0.5,
                    y: 0.2,
                    z: 1.0,
                },
                color: glam::Vec3 {
                    x: 0.0,
                    y: 0.0,
                    z: 1.0,
                },
            },
        ];

        let a_mesh = Mesh::new(&vcontext, a_vertices);
        let b_mesh = Mesh::new(&vcontext, b_vertices);

        Self {
            vcontext,
            render_geometry,
            a_mesh,
            b_mesh,
        }
    }

    pub fn render(&self, context: RenderContext) {
        let device = &self.vcontext.device;
        let cmd = context.cmd;
        unsafe {
            device.cmd_bind_pipeline(
                cmd,
                ash::vk::PipelineBindPoint::GRAPHICS,
                self.render_geometry.pipeline,
            );
            device.cmd_bind_vertex_buffers(cmd, 0, &[self.a_mesh.vertex_buffer], &[0]);
            device.cmd_draw(cmd, self.a_mesh.vertices.len() as u32, 1, 0, 0);
            device.cmd_bind_vertex_buffers(cmd, 0, &[self.b_mesh.vertex_buffer], &[0]);
            device.cmd_draw(cmd, self.b_mesh.vertices.len() as u32, 1, 0, 0);
        }
    }
}

impl Drop for RenderOwner {
    fn drop(&mut self) {
        self.a_mesh.drop(&self.vcontext);
        self.b_mesh.drop(&self.vcontext);
    }
}
