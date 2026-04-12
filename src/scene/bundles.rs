use super::{Camera, DirectionalLight, MeshInstance, Name, PointLight, SpotLight, Transform};

#[derive(Clone, Debug)]
pub struct CameraBundle {
    pub name: Option<Name>,
    pub transform: Transform,
    pub camera: Camera,
}

impl CameraBundle {
    pub fn new(transform: Transform, camera: Camera) -> Self {
        Self {
            name: None,
            transform,
            camera,
        }
    }

    pub fn named(mut self, name: impl Into<String>) -> Self {
        self.name = Some(Name::new(name));
        self
    }
}

#[derive(Clone, Debug)]
pub struct DirectionalLightBundle {
    pub name: Option<Name>,
    pub transform: Transform,
    pub light: DirectionalLight,
}

impl DirectionalLightBundle {
    pub fn new(transform: Transform, light: DirectionalLight) -> Self {
        Self {
            name: None,
            transform,
            light,
        }
    }

    pub fn named(mut self, name: impl Into<String>) -> Self {
        self.name = Some(Name::new(name));
        self
    }
}

#[derive(Clone, Debug)]
pub struct PointLightBundle {
    pub name: Option<Name>,
    pub transform: Transform,
    pub light: PointLight,
}

impl PointLightBundle {
    pub fn new(transform: Transform, light: PointLight) -> Self {
        Self {
            name: None,
            transform,
            light,
        }
    }

    pub fn named(mut self, name: impl Into<String>) -> Self {
        self.name = Some(Name::new(name));
        self
    }
}

#[derive(Clone, Debug)]
pub struct SpotLightBundle {
    pub name: Option<Name>,
    pub transform: Transform,
    pub light: SpotLight,
}

impl SpotLightBundle {
    pub fn new(transform: Transform, light: SpotLight) -> Self {
        Self {
            name: None,
            transform,
            light,
        }
    }

    pub fn named(mut self, name: impl Into<String>) -> Self {
        self.name = Some(Name::new(name));
        self
    }
}

#[derive(Clone, Debug)]
pub struct MeshInstanceBundle {
    pub name: Option<Name>,
    pub transform: Transform,
    pub mesh_instance: MeshInstance,
}

impl MeshInstanceBundle {
    pub fn new(transform: Transform, mesh_instance: MeshInstance) -> Self {
        Self {
            name: None,
            transform,
            mesh_instance,
        }
    }

    pub fn named(mut self, name: impl Into<String>) -> Self {
        self.name = Some(Name::new(name));
        self
    }
}
