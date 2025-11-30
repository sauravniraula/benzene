use crate::shared::types::Id;

pub fn get_random_id() -> Id {
    rand::random()
}
