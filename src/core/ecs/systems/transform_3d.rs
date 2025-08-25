use nalgebra::{Matrix4, Translation3, UnitQuaternion, Vector3};

use crate::core::ecs::components::Transform3D;

pub fn translate_transform_3d(t: &mut Transform3D, delta: Vector3<f32>) {
    t.position += delta;
    t.dirty = true;
}

// delta contains (roll, pitch, yaw)
pub fn rotate_transform_3d(t: &mut Transform3D, delta: Vector3<f32>) {
    t.rotation += delta;
    t.dirty = true;
}

// Multiply scale by given factors
pub fn scale_transform_3d_mul(t: &mut Transform3D, factors: Vector3<f32>) {
    t.scale.component_mul_assign(&factors);
    t.dirty = true;
}

// Set scale to an absolute value
pub fn scale_transform_3d_set(t: &mut Transform3D, new_scale: Vector3<f32>) {
    t.scale = new_scale;
    t.dirty = true;
}

pub fn update_transform_3d_matrix(t: &mut Transform3D) {
    let r = UnitQuaternion::from_euler_angles(t.rotation.x, t.rotation.y, t.rotation.z);
    let s = Matrix4::new_nonuniform_scaling(&t.scale);
    let tr = Translation3::new(t.position.x, t.position.y, t.position.z);
    t.cached_transform = tr.to_homogeneous() * r.to_homogeneous() * s;
    t.dirty = false;
}

pub fn update_transforms_3d(transforms: &mut [Transform3D]) {
    for t in transforms.iter_mut() {
        if t.dirty {
            update_transform_3d_matrix(t);
        }
    }
}
