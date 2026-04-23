use glam;

pub struct Transform {
    position: glam::Vec3,
    facing: glam::Vec3,
    scale: glam::Vec3,
    transformation_matrix: glam::Mat4,
    is_dirty: bool,
}

impl Transform {
    pub fn new() -> Self {
        Self {
            position: glam::Vec3::ZERO,
            facing: glam::Vec3::Z,
            scale: glam::Vec3::ONE,
            transformation_matrix: glam::Mat4::IDENTITY,
            is_dirty: true,
        }
    }

    pub fn get_transformation_matrix(&mut self) -> glam::Mat4 {
        if !self.is_dirty {
            return self.transformation_matrix;
        }
        let rotation = glam::Quat::look_to_rh(self.facing, glam::Vec3::Y);
        self.transformation_matrix =
            glam::Mat4::from_scale_rotation_translation(self.scale, rotation, self.position);
        self.transformation_matrix
    }
}
