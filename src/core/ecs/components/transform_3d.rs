use ash::vk::Extent2D;
use nalgebra::{Matrix4, Perspective3, Rotation3, Translation3, UnitQuaternion, Vector3};

#[derive(Clone, Debug)]
pub struct Transform3D {
    pub position: Vector3<f32>,
    pub rotation: Vector3<f32>,
    pub scale: Vector3<f32>,
    pub cached_transform: Matrix4<f32>,
    pub dirty: bool,
}

impl Transform3D {
    pub fn new(position: Vector3<f32>, rotation: Vector3<f32>, scale: Vector3<f32>) -> Self {
        Self {
            position,
            rotation,
            scale,
            cached_transform: Matrix4::identity(),
            dirty: true,
        }
    }
    pub fn new_default() -> Self {
        Self::new(
            Vector3::new(0.0, 0.0, 0.0),
            Vector3::new(0.0, 0.0, 0.0),
            Vector3::new(1.0, 1.0, 1.0),
        )
    }

    pub fn get_unit_quaternion(&self) -> UnitQuaternion<f32> {
        UnitQuaternion::from_euler_angles(self.rotation.x, self.rotation.y, self.rotation.z)
    }

    pub fn get_transform_3d_view_projection(
        &self,
        image_extent: Extent2D,
    ) -> (Matrix4<f32>, Matrix4<f32>) {
        let rot = self.rotation;
        let pos = self.position;
        let r = Rotation3::from_euler_angles(rot.x, rot.y, rot.z);
        let r_inv = r.inverse();
        let t_inv = Translation3::new(-pos.x, -pos.y, -pos.z);
        let view = r_inv.to_homogeneous() * t_inv.to_homogeneous();

        // Projection (Vulkan NDC requires Y flip)
        let aspect = (image_extent.width as f32).max(1.0) / (image_extent.height as f32).max(1.0);
        let fovy = std::f32::consts::FRAC_PI_3; // 60 degrees
        let znear = 0.1_f32;
        let zfar = 100.0_f32;
        let mut projection = Perspective3::new(aspect, fovy, znear, zfar).to_homogeneous();
        projection[(1, 1)] *= -1.0;

        (view, projection)
    }
}
