use std::sync::Arc;

use crate::{
    backend::{mesh::Mesh, render_loop::RenderContext, vcontext::Vcontext},
    render::geometry::RenderGeometry,
};

pub struct RenderOwner {
    vcontext: Arc<Vcontext>,
    render_geometry: RenderGeometry,
    a_mesh: Mesh,
}

impl RenderOwner {
    pub fn new(vcontext: Arc<Vcontext>) -> Self {
        let render_geometry = RenderGeometry::new(vcontext.clone());

        let a_mesh = Mesh::new(vcontext.clone());

        Self {
            vcontext,
            render_geometry,
            a_mesh,
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
        }
    }
}
