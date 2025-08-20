use ash::vk::Extent2D;
use glfw::{Action, Key, WindowEvent};
use nalgebra::{Matrix4, Point3, Vector3};
use std::collections::HashSet;
pub struct Camera {
    position: Point3<f32>,
    speed: f32,
    yaw: f32,
    pitch: f32,
    rotation_speed: f32,
    pressed_keys: HashSet<Key>,
    dirty: bool,
}

impl Camera {
    pub fn new() -> Self {
        let position = Point3::<f32>::new(20.0, 20.0, 20.0);
        let look_at = Point3::<f32>::new(0.0, 0.0, 0.0);
        let forward = (look_at - position).normalize();
        let yaw: f32 = forward.z.atan2(forward.x);
        let pitch: f32 = forward.y.asin();

        Self {
            position,
            speed: 5.0,
            yaw,
            pitch,
            rotation_speed: 1.5,
            pressed_keys: HashSet::new(),
            dirty: true,
        }
    }

    fn translate(&mut self, delta: Vector3<f32>) {
        self.position = self.position + delta;
        self.dirty = true;
    }

    fn basis_vectors(&self) -> (Vector3<f32>, Vector3<f32>, Vector3<f32>) {
        let world_up = Vector3::new(0.0, 1.0, 0.0);
        let cos_pitch = self.pitch.cos();
        let forward = Vector3::new(
            cos_pitch * self.yaw.cos(),
            self.pitch.sin(),
            cos_pitch * self.yaw.sin(),
        );
        let right = forward.cross(&world_up).normalize();
        let up = right.cross(&forward).normalize();
        (forward, right, up)
    }

    pub fn move_view_relative(&mut self, input: Vector3<f32>, scale: f32) {
        if input == Vector3::new(0.0, 0.0, 0.0) {
            return;
        }
        let (forward, right, up) = self.basis_vectors();
        let movement = right * input.x + up * input.y + forward * input.z;
        self.translate(movement.normalize() * scale);
    }

    pub fn handle_window_event(&mut self, event: &WindowEvent) {
        match event {
            WindowEvent::Key(key, _, Action::Press, _)
            | WindowEvent::Key(key, _, Action::Repeat, _) => {
                self.pressed_keys.insert(*key);
                // Movement effect is applied in update(); we'll mark dirty there if moved
            }
            WindowEvent::Key(key, _, Action::Release, _) => {
                self.pressed_keys.remove(key);
            }
            _ => {}
        }
    }

    pub fn update(&mut self, _frame_index: usize, image_extent: Extent2D, dt: f32) {
        // Build input vector from currently pressed keys
        let mut input = Vector3::new(0.0, 0.0, 0.0);
        if self.pressed_keys.contains(&Key::A) {
            input.x -= 1.0;
        }
        if self.pressed_keys.contains(&Key::D) {
            input.x += 1.0;
        }
        if self.pressed_keys.contains(&Key::LeftControl) {
            input.y -= 1.0;
        }
        if self.pressed_keys.contains(&Key::Space) {
            input.y += 1.0;
        }
        if self.pressed_keys.contains(&Key::S) {
            input.z -= 1.0;
        }
        if self.pressed_keys.contains(&Key::W) {
            input.z += 1.0;
        }

        if input != Vector3::new(0.0, 0.0, 0.0) {
            let speed_multiplier = if self.pressed_keys.contains(&Key::LeftShift)
                || self.pressed_keys.contains(&Key::RightShift)
            {
                5.0
            } else {
                1.0
            };
            let step = self.speed * speed_multiplier * dt;
            self.move_view_relative(input, step);
            self.dirty = true;
        }

        // Arrow keys control yaw/pitch like mouse look
        let mut rotated = false;
        if self.pressed_keys.contains(&Key::Left) {
            self.yaw -= self.rotation_speed * dt;
            rotated = true;
        }
        if self.pressed_keys.contains(&Key::Right) {
            self.yaw += self.rotation_speed * dt;
            rotated = true;
        }
        if self.pressed_keys.contains(&Key::Up) {
            self.pitch += self.rotation_speed * dt;
            rotated = true;
        }
        if self.pressed_keys.contains(&Key::Down) {
            self.pitch -= self.rotation_speed * dt;
            rotated = true;
        }
        if rotated {
            let limit = std::f32::consts::FRAC_PI_2 - 0.01;
            if self.pitch > limit {
                self.pitch = limit;
            }
            if self.pitch < -limit {
                self.pitch = -limit;
            }
            if self.yaw > std::f32::consts::PI {
                self.yaw -= 2.0 * std::f32::consts::PI;
            }
            if self.yaw < -std::f32::consts::PI {
                self.yaw += 2.0 * std::f32::consts::PI;
            }
            self.dirty = true;
        }

        let _ = image_extent; // kept for signature compatibility
    }

    pub fn destroy(&self) {}

    pub fn view_projection(&self, image_extent: Extent2D) -> (Matrix4<f32>, Matrix4<f32>) {
        let cos_pitch = self.pitch.cos();
        let forward = Vector3::new(
            cos_pitch * self.yaw.cos(),
            self.pitch.sin(),
            cos_pitch * self.yaw.sin(),
        );
        let view = Matrix4::<f32>::look_at_rh(
            &self.position,
            &(self.position + forward),
            &Vector3::<f32>::new(0.0, 1.0, 0.0),
        );
        let mut projection = Matrix4::<f32>::new_perspective(
            (image_extent.width as f32) / (image_extent.height as f32),
            45_f32.to_radians(),
            0.1,
            100.0,
        );
        projection[(1, 1)] *= -1.0;
        (view, projection)
    }

    pub fn take_dirty(&mut self) -> bool {
        let was_dirty = self.dirty;
        self.dirty = false;
        was_dirty
    }
}
