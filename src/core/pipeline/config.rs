pub struct VPipelineInfoConfig {
    pub vertex_shader_file: String,
    pub fragment_shader_file: String,
}

impl VPipelineInfoConfig {
    pub fn default() -> Self {
        Self {
            vertex_shader_file: "src/shaders/shader.vert".into(),
            fragment_shader_file: "src/shaders/shader.frag".into(),
        }
    }
}
