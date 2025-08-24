use ash::vk;
use nalgebra::{Matrix4, Rotation3, Translation3, Vector3};

use crate::{
    core::{
        gpu::{
            model::Model,
            recordable::{Drawable, RecordContext, Recordable},
        },
        model_push_constant::ModelPushConstant,
    },
    vulkan_backend::backend::VBackend,
};

pub struct GameObject {
    pub model: Model,
    pub position: Vector3<f32>,
    pub rotation: Vector3<f32>,
    dirty: bool,
    cached_transform: Matrix4<f32>,
}

impl GameObject {
    pub fn new(model: Model) -> Self {
        Self {
            model,
            position: Vector3::new(0.0, 0.0, 0.0),
            rotation: Vector3::new(0.0, 0.0, 0.0),
            dirty: true,
            cached_transform: Matrix4::identity(),
        }
    }

    pub fn from_obj(v_backend: &VBackend, obj_path: &str) -> Self {
        let model = Model::from_obj(v_backend, obj_path);
        Self::new(model)
    }

    pub fn set_position(&mut self, position: Vector3<f32>) {
        self.position = position;
        self.dirty = true;
    }

    pub fn move_by(&mut self, delta: Vector3<f32>) {
        self.position += delta;
        self.dirty = true;
    }

    pub fn set_rotation(&mut self, rotation_radians: Vector3<f32>) {
        self.rotation = rotation_radians;
        self.dirty = true;
    }

    pub fn rotate_by(&mut self, delta_radians: Vector3<f32>) {
        self.rotation += delta_radians;
        self.dirty = true;
    }

    pub fn forward_dir(&self) -> Vector3<f32> {
        // Using yaw (ry) and pitch (rx) to compute forward; roll (rz) does not affect forward
        let (rx, ry, _rz) = (self.rotation.x, self.rotation.y, self.rotation.z);
        let cos_pitch = rx.cos();
        let sin_pitch = rx.sin();
        let cos_yaw = ry.cos();
        let sin_yaw = ry.sin();
        Vector3::new(cos_pitch * sin_yaw, sin_pitch, cos_pitch * cos_yaw).normalize()
    }

    pub fn right_dir(&self) -> Vector3<f32> {
        let world_up = Vector3::new(0.0, 1.0, 0.0);
        self.forward_dir().cross(&world_up).normalize()
    }

    pub fn up_dir(&self) -> Vector3<f32> {
        self.right_dir().cross(&self.forward_dir()).normalize()
    }

    pub fn move_forward(&mut self, distance: f32) {
        self.position += self.forward_dir() * distance;
    }

    pub fn move_right(&mut self, distance: f32) {
        self.position += self.right_dir() * distance;
    }

    pub fn move_up(&mut self, distance: f32) {
        self.position += self.up_dir() * distance;
    }

    fn recompute_transform(&mut self) {
        let rotation =
            Rotation3::from_euler_angles(self.rotation.x, self.rotation.y, self.rotation.z);
        let translation = Translation3::from(self.position);
        self.cached_transform = translation.to_homogeneous() * rotation.to_homogeneous();
        self.dirty = false;
    }

    pub fn transform(&self) -> Matrix4<f32> {
        self.cached_transform
    }

    pub fn pre_render(&mut self) {
        if self.dirty {
            self.recompute_transform();
        }
    }

    pub fn destroy(&self, v_backend: &VBackend) {
        self.model.destroy(v_backend);
    }
}

impl Recordable for GameObject {
    fn record(&self, ctx: &RecordContext) {
        let push = ModelPushConstant {
            transform: self.cached_transform,
        };
        let data = unsafe {
            std::slice::from_raw_parts(
                (&push as *const ModelPushConstant) as *const u8,
                size_of::<ModelPushConstant>(),
            )
        };

        unsafe {
            ctx.v_device.device.cmd_push_constants(
                ctx.cmd,
                ctx.pipeline_layout,
                vk::ShaderStageFlags::VERTEX | vk::ShaderStageFlags::FRAGMENT,
                0,
                data,
            );
        }

        self.model.draw(ctx.v_device, ctx.cmd);
    }
}
