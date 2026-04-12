use winit::window::Window;

use crate::{
    assets::{AssetManager, MaterialId, MeshId, TextureId},
    error::Result,
    render::Renderer,
    scene::Scene,
    ui::EguiFrame,
};

pub struct Engine {
    assets: AssetManager,
    renderer: Renderer,
}

impl Engine {
    pub fn new(window: &Window) -> Result<Self> {
        let renderer = Renderer::new(window)?;
        let assets = AssetManager::new(renderer.context(), renderer.material_layout())?;
        Ok(Self { assets, renderer })
    }

    pub fn create_scene(&self) -> Scene {
        Scene::new()
    }

    pub fn render(&mut self, scene: &Scene) -> Result<()> {
        self.renderer.render(scene, &self.assets)
    }

    pub fn render_with_egui(&mut self, scene: &Scene, egui_frame: &EguiFrame) -> Result<()> {
        self.renderer
            .render_with_egui(scene, &self.assets, egui_frame)
    }

    pub fn resize(&mut self, window: &Window) -> Result<()> {
        self.renderer.resize(window)
    }

    pub fn load_mesh_obj(&mut self, path: &str) -> Result<MeshId> {
        self.assets.load_mesh_obj(path)
    }

    pub fn load_texture(&mut self, path: &str) -> Result<TextureId> {
        self.assets.load_texture(path)
    }

    pub fn create_material(&mut self, texture: TextureId) -> Result<MaterialId> {
        self.assets.create_material(texture)
    }

    pub fn assets(&self) -> &AssetManager {
        &self.assets
    }

    pub fn assets_mut(&mut self) -> &mut AssetManager {
        &mut self.assets
    }

    pub fn renderer(&self) -> &Renderer {
        &self.renderer
    }

    pub fn renderer_mut(&mut self) -> &mut Renderer {
        &mut self.renderer
    }
}

impl Drop for Engine {
    fn drop(&mut self) {
        self.renderer.wait_idle();
    }
}
