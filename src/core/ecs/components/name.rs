pub struct Name {
    pub value: String,
}

impl Name {
    pub fn new(value: &str) -> Self {
        Self {
            value: value.into(),
        }
    }
}
