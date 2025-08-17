use ash::vk::Extent2D;
use glfw::{Action, Key, WindowEvent};
use nalgebra::{Matrix4, Point3, Vector3};
use std::{collections::HashSet, time::Instant};

pub struct CameraUniform {
    pub view: Matrix4<f32>,
    pub projection: Matrix4<f32>,
}
pub struct Camera {
    position: Point3<f32>,
    target: Point3<f32>,
    speed: f32,
    yaw: f32,
    pitch: f32,
    mouse_sensitivity: f32,
    last_cursor_pos: Option<(f64, f64)>,
    pressed_keys: HashSet<Key>,
    last_update_time: Instant,
}

impl Camera {
    pub fn new() -> Self {
        let position = Point3::<f32>::new(2.0, 0.0, 2.0);
        let target = Point3::<f32>::new(0.0, 0.0, 0.0);
        let forward = (target - position).normalize();
        let yaw: f32 = forward.z.atan2(forward.x);
        let pitch: f32 = forward.y.asin();

        Self {
            position,
            target,
            speed: 3.0,
            yaw,
            pitch,
            mouse_sensitivity: 0.002,
            last_cursor_pos: None,
            pressed_keys: HashSet::new(),
            last_update_time: Instant::now(),
        }
    }

    fn translate(&mut self, delta: Vector3<f32>) {
        self.position = self.position + delta;
        self.target = self.target + delta;
    }

    fn basis_vectors(&self) -> (Vector3<f32>, Vector3<f32>, Vector3<f32>) {
        let world_up = Vector3::new(0.0, 1.0, 0.0);
        let mut forward = (self.target - self.position).normalize();
        // Prevent NaNs if position == target; default forward to -Z
        if !forward.iter().all(|c| c.is_finite()) {
            forward = Vector3::new(0.0, 0.0, -1.0);
        }
        let mut right = forward.cross(&world_up).normalize();
        if !right.iter().all(|c| c.is_finite()) {
            right = Vector3::new(1.0, 0.0, 0.0);
        }
        let up = right.cross(&forward).normalize();
        (forward, right, up)
    }

    fn update_target_from_angles(&mut self) {
        let cp = self.pitch.cos();
        let forward = Vector3::new(cp * self.yaw.cos(), self.pitch.sin(), cp * self.yaw.sin());
        self.target = self.position + forward;
    }

    pub fn get_speed(&self) -> f32 { self.speed }

    pub fn move_view_relative(&mut self, input: Vector3<f32>, scale: f32) {
        if input == Vector3::new(0.0, 0.0, 0.0) { return; }
        let (forward, right, up) = self.basis_vectors();
        let movement = right * input.x + up * input.y + forward * input.z;
        self.translate(movement.normalize() * scale);
    }

    pub fn handle_window_event(&mut self, event: &WindowEvent) {
        match event {
            WindowEvent::Key(key, _, Action::Press, _) | WindowEvent::Key(key, _, Action::Repeat, _) => {
                self.pressed_keys.insert(*key);
            }
            WindowEvent::Key(key, _, Action::Release, _) => {
                self.pressed_keys.remove(key);
            }
            WindowEvent::CursorPos(x, y) => {
                if let Some((lx, ly)) = self.last_cursor_pos {
                    let dx = (*x - lx) as f32;
                    let dy = (*y - ly) as f32;
                    self.yaw += dx * self.mouse_sensitivity;
                    self.pitch -= dy * self.mouse_sensitivity;
                    let limit = std::f32::consts::FRAC_PI_2 - 0.01;
                    if self.pitch > limit { self.pitch = limit; }
                    if self.pitch < -limit { self.pitch = -limit; }
                    if self.yaw > std::f32::consts::PI { self.yaw -= 2.0 * std::f32::consts::PI; }
                    if self.yaw < -std::f32::consts::PI { self.yaw += 2.0 * std::f32::consts::PI; }
                    self.update_target_from_angles();
                }
                self.last_cursor_pos = Some((*x, *y));
            }
            _ => {}
        }
    }

    pub fn update(&mut self, _frame_index: usize, image_extent: Extent2D) {
        // Time since last update for smooth movement
        let now = Instant::now();
        let dt = (now - self.last_update_time).as_secs_f32();
        self.last_update_time = now;

        // Build input vector from currently pressed keys
        let mut input = Vector3::new(0.0, 0.0, 0.0);
        if self.pressed_keys.contains(&Key::A) { input.x -= 1.0; }
        if self.pressed_keys.contains(&Key::D) { input.x += 1.0; }
        if self.pressed_keys.contains(&Key::Q) { input.y -= 1.0; }
        if self.pressed_keys.contains(&Key::E) { input.y += 1.0; }
        if self.pressed_keys.contains(&Key::S) { input.z -= 1.0; }
        if self.pressed_keys.contains(&Key::W) { input.z += 1.0; }

        if input != Vector3::new(0.0, 0.0, 0.0) {
            let speed_multiplier = if self.pressed_keys.contains(&Key::LeftShift) || self.pressed_keys.contains(&Key::RightShift) { 5.0 } else { 1.0 };
            let step = self.speed * speed_multiplier * dt;
            self.move_view_relative(input, step);
        }

        let _ = image_extent; // kept for signature compatibility
    }

    pub fn get_uniform(&self, image_extent: Extent2D) -> CameraUniform {
        let view = Matrix4::<f32>::look_at_rh(
            &self.position,
            &self.target,
            &Vector3::<f32>::new(0.0, 1.0, 0.0),
        );
        let mut projection = Matrix4::<f32>::new_perspective(
            (image_extent.width as f32) / (image_extent.height as f32),
            45_f32.to_radians(),
            0.0,
            100.0,
        );
        projection[(1, 1)] *= -1.0;
        CameraUniform { view, projection }
    }

    pub fn destroy(&self) {}
}
