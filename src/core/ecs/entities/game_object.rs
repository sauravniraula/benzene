use rand::Rng;

use crate::core::ecs::types::EntityId;

#[derive(Clone)]
pub struct GameObject {
    id: EntityId,
    pub name: String,
}

impl GameObject {
    pub fn new(name: &str) -> Self {
        return Self {
            id: rand::rng().random(),
            name: name.into(),
        };
    }

    pub fn get_id(&self) -> &[u8; 64] {
        &self.id
    }
}
