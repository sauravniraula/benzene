use crate::core::ecs::types::Id;

pub fn get_random_id() -> Id {
    rand::random()
}
