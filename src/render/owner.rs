use std::sync::Arc;

use crate::{backend::vcontext::Vcontext, render::geometry::RenderGeometry};

pub struct RenderOwner {
    vcontext: Arc<Vcontext>,
    render_geometry: RenderGeometry,
}

impl RenderOwner {
    pub fn new(vcontext: Arc<Vcontext>) -> Self {
        let render_geometry = RenderGeometry::new(vcontext.clone());

        Self {
            vcontext,
            render_geometry,
        }
    }
}
