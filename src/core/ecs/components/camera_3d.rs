pub struct Camera3D {
    pub speed: f32,
    pub rotation_speed: f32,
}

impl Camera3D {
    pub fn new(speed: f32, rotation_speed: f32) -> Self {
        Self {
            speed,
            rotation_speed,
        }
    }

    pub fn new_default() -> Self {
        Self::new(5.0, 0.1)
    }
}
