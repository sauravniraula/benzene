use crate::core::assets::{MaterialHandle, MeshHandle};

pub struct MeshRenderer {
    pub mesh: MeshHandle,
    pub material: Option<MaterialHandle>,
}

impl MeshRenderer {
    pub fn new(mesh: MeshHandle, material: Option<MaterialHandle>) -> Self {
        Self { mesh, material }
    }
}
