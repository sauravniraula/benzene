use nalgebra::{Matrix4, Perspective3, Rotation3, Translation3, Vector3, Vector4};
use slotmap::new_key_type;

use crate::assets::{MaterialId, MeshId};

new_key_type! {
    pub struct Entity;
}

#[derive(Clone, Debug)]
pub struct Name {
    pub value: String,
}

impl Name {
    pub fn new(value: impl Into<String>) -> Self {
        Self {
            value: value.into(),
        }
    }
}

#[derive(Clone, Debug)]
pub struct Visibility {
    pub visible: bool,
}

impl Visibility {
    pub fn visible() -> Self {
        Self { visible: true }
    }
}

impl Default for Visibility {
    fn default() -> Self {
        Self::visible()
    }
}

#[derive(Clone, Debug)]
pub struct Transform {
    pub position: Vector3<f32>,
    pub rotation: Vector3<f32>,
    pub scale: Vector3<f32>,
    pub cached_matrix: Matrix4<f32>,
    pub dirty: bool,
}

impl Transform {
    pub fn new(position: Vector3<f32>, rotation: Vector3<f32>, scale: Vector3<f32>) -> Self {
        Self {
            position,
            rotation,
            scale,
            cached_matrix: Matrix4::identity(),
            dirty: true,
        }
    }

    pub fn identity() -> Self {
        Self::new(
            Vector3::new(0.0, 0.0, 0.0),
            Vector3::new(0.0, 0.0, 0.0),
            Vector3::new(1.0, 1.0, 1.0),
        )
    }

    pub fn rotation_matrix(&self) -> Rotation3<f32> {
        Rotation3::from_euler_angles(self.rotation.x, self.rotation.y, self.rotation.z)
    }

    pub fn update_cached_matrix(&mut self) {
        let rotation = self.rotation_matrix();
        let scale = Matrix4::new_nonuniform_scaling(&self.scale);
        let translation =
            Translation3::new(self.position.x, self.position.y, self.position.z).to_homogeneous();
        self.cached_matrix = translation * rotation.to_homogeneous() * scale;
        self.dirty = false;
    }

    pub fn view_projection(
        &self,
        camera: &Camera,
        width: u32,
        height: u32,
    ) -> (Matrix4<f32>, Matrix4<f32>) {
        let rotation_inverse = self.rotation_matrix().inverse();
        let translation_inverse =
            Translation3::new(-self.position.x, -self.position.y, -self.position.z)
                .to_homogeneous();
        let view = rotation_inverse.to_homogeneous() * translation_inverse;

        let aspect = (width.max(1) as f32) / (height.max(1) as f32);
        let fov_y = camera
            .fov_y_radians
            .clamp(1.0f32.to_radians(), 170.0f32.to_radians());
        let near_clip = camera.near_clip.max(0.001);
        let far_clip = camera.far_clip.max(near_clip + 0.001);
        let mut projection = Perspective3::new(aspect, fov_y, near_clip, far_clip).to_homogeneous();
        projection[(1, 1)] *= -1.0;

        (view, projection)
    }
}

impl Default for Transform {
    fn default() -> Self {
        Self::identity()
    }
}

#[derive(Clone, Debug)]
pub struct Camera {
    pub speed: f32,
    pub rotation_speed: f32,
    pub fov_y_radians: f32,
    pub near_clip: f32,
    pub far_clip: f32,
}

impl Camera {
    pub fn new(speed: f32, rotation_speed: f32) -> Self {
        Self {
            speed,
            rotation_speed,
            fov_y_radians: std::f32::consts::FRAC_PI_3,
            near_clip: 0.1,
            far_clip: 10_000.0,
        }
    }

    pub fn with_projection(mut self, fov_y_radians: f32, near_clip: f32, far_clip: f32) -> Self {
        self.fov_y_radians = fov_y_radians;
        self.near_clip = near_clip;
        self.far_clip = far_clip;
        self
    }
}

impl Default for Camera {
    fn default() -> Self {
        Self::new(5.0, 0.1)
    }
}

#[derive(Clone, Debug)]
pub struct PointLight {
    pub color: Vector4<f32>,
}

impl PointLight {
    pub fn new(color: Vector4<f32>) -> Self {
        Self { color }
    }
}

#[derive(Clone, Debug)]
pub struct DirectionalLight {
    pub color: Vector4<f32>,
}

impl DirectionalLight {
    pub fn new(color: Vector4<f32>) -> Self {
        Self { color }
    }
}

#[derive(Clone, Debug)]
pub struct SpotLight {
    pub color: Vector4<f32>,
}

impl SpotLight {
    pub fn new(color: Vector4<f32>) -> Self {
        Self { color }
    }
}

#[derive(Clone, Debug)]
pub struct MeshInstance {
    pub mesh: MeshId,
    pub material: Option<MaterialId>,
}

impl MeshInstance {
    pub fn new(mesh: MeshId, material: Option<MaterialId>) -> Self {
        Self { mesh, material }
    }
}
