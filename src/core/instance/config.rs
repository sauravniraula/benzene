pub struct VInstanceConfig {
    pub application_name: String,
    pub extensions: Vec<String>,
    pub layers: Vec<String>,
    pub enable_validation: bool,
    pub enable_debug: bool,
}

impl VInstanceConfig {
    pub fn default() -> Self {
        Self {
            application_name: "Hello Vulkan".into(),
            extensions: vec![],
            layers: vec![],
            enable_validation: true,
            enable_debug: true,
        }
    }
}
