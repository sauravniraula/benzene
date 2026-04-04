use std::collections::HashMap;

use crate::{
    core::gpu::{model::Model, texture::ImageTexture},
    vulkan_backend::backend::VBackend,
};

pub type MeshHandle = u32;
pub type TextureHandle = u32;
pub type MaterialHandle = usize;

pub struct AssetStore {
    next_mesh_handle: MeshHandle,
    next_texture_handle: TextureHandle,
    pub default_material: MaterialHandle,
    meshes: HashMap<MeshHandle, Model>,
    textures: HashMap<TextureHandle, ImageTexture>,
}

impl AssetStore {
    pub fn new() -> Self {
        Self {
            next_mesh_handle: 0,
            next_texture_handle: 0,
            default_material: 0,
            meshes: HashMap::new(),
            textures: HashMap::new(),
        }
    }

    pub fn insert_mesh(&mut self, mesh: Model) -> MeshHandle {
        let handle = self.next_mesh_handle;
        self.next_mesh_handle += 1;
        self.meshes.insert(handle, mesh);
        handle
    }

    pub fn get_mesh(&self, handle: MeshHandle) -> Option<&Model> {
        self.meshes.get(&handle)
    }

    pub fn insert_texture(&mut self, texture: ImageTexture) -> TextureHandle {
        let handle = self.next_texture_handle;
        self.next_texture_handle += 1;
        self.textures.insert(handle, texture);
        handle
    }

    pub fn get_texture(&self, handle: TextureHandle) -> Option<&ImageTexture> {
        self.textures.get(&handle)
    }

    pub fn remove_texture(&mut self, handle: TextureHandle) -> Option<ImageTexture> {
        self.textures.remove(&handle)
    }

    pub fn destroy(&mut self, v_backend: &VBackend) {
        for (_, mesh) in self.meshes.drain() {
            mesh.destroy(v_backend);
        }

        for (_, texture) in self.textures.drain() {
            texture.destroy(v_backend);
        }
    }
}
