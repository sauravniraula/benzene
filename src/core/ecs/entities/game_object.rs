use crate::core::ecs::types::Id;

#[derive(Clone)]
pub struct GameObject {
    id: Id,
    pub name: String,
}

impl GameObject {
    pub fn new(name: &str) -> Self {
        return Self {
            id: rand::random(),
            name: name.into(),
        };
    }

    pub fn get_id(&self) -> &[u8; 64] {
        &self.id
    }
}
