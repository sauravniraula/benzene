pub struct WindowConfig {
    pub title: String,
    pub width: u32,
    pub height: u32,
}

impl WindowConfig {
    pub fn default() -> Self {
        Self {
            title: "My Window".into(),
            width: 1000,
            height: 600,
        }
    }
}
