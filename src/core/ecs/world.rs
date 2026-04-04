use std::collections::HashMap;

use crate::core::ecs::{
    components::{
        Camera3D, MeshRenderer, Name, PointLight3D, Transform3D,
        directional_light_3d::DirectionalLight3D, spot_light_3d::SpotLight3D,
    },
    entities::Entity,
};

pub struct World {
    next_entity: u32,
    pub names: HashMap<Entity, Name>,
    pub transforms: HashMap<Entity, Transform3D>,
    pub cameras: HashMap<Entity, Camera3D>,
    pub point_lights: HashMap<Entity, PointLight3D>,
    pub directional_lights: HashMap<Entity, DirectionalLight3D>,
    pub spot_lights: HashMap<Entity, SpotLight3D>,
    pub mesh_renderers: HashMap<Entity, MeshRenderer>,
}

impl World {
    pub fn new() -> Self {
        Self {
            next_entity: 0,
            names: HashMap::new(),
            transforms: HashMap::new(),
            cameras: HashMap::new(),
            point_lights: HashMap::new(),
            directional_lights: HashMap::new(),
            spot_lights: HashMap::new(),
            mesh_renderers: HashMap::new(),
        }
    }

    pub fn spawn(&mut self) -> Entity {
        let entity = Entity(self.next_entity);
        self.next_entity += 1;
        entity
    }

    pub fn despawn(&mut self, entity: Entity) {
        self.names.remove(&entity);
        self.transforms.remove(&entity);
        self.cameras.remove(&entity);
        self.point_lights.remove(&entity);
        self.directional_lights.remove(&entity);
        self.spot_lights.remove(&entity);
        self.mesh_renderers.remove(&entity);
    }
}
