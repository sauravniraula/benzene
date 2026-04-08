use std::collections::HashSet;

use nalgebra::{Matrix4, Perspective3, Rotation3, Translation3, Vector3, Vector4};
use slotmap::{SecondaryMap, SlotMap, new_key_type};
use winit::{
    event::{ElementState, MouseButton, WindowEvent},
    keyboard::{KeyCode, PhysicalKey},
};

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

    pub fn view_projection(&self, width: u32, height: u32) -> (Matrix4<f32>, Matrix4<f32>) {
        let rotation_inverse = self.rotation_matrix().inverse();
        let translation_inverse =
            Translation3::new(-self.position.x, -self.position.y, -self.position.z)
                .to_homogeneous();
        let view = rotation_inverse.to_homogeneous() * translation_inverse;

        let aspect = (width.max(1) as f32) / (height.max(1) as f32);
        let mut projection =
            Perspective3::new(aspect, std::f32::consts::FRAC_PI_3, 0.1, 10_000.0).to_homogeneous();
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
}

impl Camera {
    pub fn new(speed: f32, rotation_speed: f32) -> Self {
        Self {
            speed,
            rotation_speed,
        }
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

#[derive(Clone, Debug)]
pub struct CameraBundle {
    pub name: Option<Name>,
    pub transform: Transform,
    pub camera: Camera,
}

impl CameraBundle {
    pub fn new(transform: Transform, camera: Camera) -> Self {
        Self {
            name: None,
            transform,
            camera,
        }
    }

    pub fn named(mut self, name: impl Into<String>) -> Self {
        self.name = Some(Name::new(name));
        self
    }
}

#[derive(Clone, Debug)]
pub struct DirectionalLightBundle {
    pub name: Option<Name>,
    pub transform: Transform,
    pub light: DirectionalLight,
}

impl DirectionalLightBundle {
    pub fn new(transform: Transform, light: DirectionalLight) -> Self {
        Self {
            name: None,
            transform,
            light,
        }
    }

    pub fn named(mut self, name: impl Into<String>) -> Self {
        self.name = Some(Name::new(name));
        self
    }
}

#[derive(Clone, Debug)]
pub struct PointLightBundle {
    pub name: Option<Name>,
    pub transform: Transform,
    pub light: PointLight,
}

impl PointLightBundle {
    pub fn new(transform: Transform, light: PointLight) -> Self {
        Self {
            name: None,
            transform,
            light,
        }
    }

    pub fn named(mut self, name: impl Into<String>) -> Self {
        self.name = Some(Name::new(name));
        self
    }
}

#[derive(Clone, Debug)]
pub struct SpotLightBundle {
    pub name: Option<Name>,
    pub transform: Transform,
    pub light: SpotLight,
}

impl SpotLightBundle {
    pub fn new(transform: Transform, light: SpotLight) -> Self {
        Self {
            name: None,
            transform,
            light,
        }
    }

    pub fn named(mut self, name: impl Into<String>) -> Self {
        self.name = Some(Name::new(name));
        self
    }
}

#[derive(Clone, Debug)]
pub struct MeshInstanceBundle {
    pub name: Option<Name>,
    pub transform: Transform,
    pub mesh_instance: MeshInstance,
}

impl MeshInstanceBundle {
    pub fn new(transform: Transform, mesh_instance: MeshInstance) -> Self {
        Self {
            name: None,
            transform,
            mesh_instance,
        }
    }

    pub fn named(mut self, name: impl Into<String>) -> Self {
        self.name = Some(Name::new(name));
        self
    }
}

#[derive(Clone, Debug, Default)]
pub struct FrameInput {
    pressed_keys: HashSet<KeyCode>,
    look_delta: (f32, f32),
}

impl FrameInput {
    pub fn is_key_pressed(&self, key: KeyCode) -> bool {
        self.pressed_keys.contains(&key)
    }

    pub fn look_delta(&self) -> (f32, f32) {
        self.look_delta
    }
}

#[derive(Debug, Default)]
pub struct InputState {
    pressed_keys: HashSet<KeyCode>,
    last_cursor_position: Option<(f64, f64)>,
    look_delta: (f32, f32),
    cursor_captured: bool,
}

impl InputState {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn handle_window_event(&mut self, event: &WindowEvent) {
        match event {
            WindowEvent::KeyboardInput { event, .. } => {
                if let PhysicalKey::Code(key) = event.physical_key {
                    match event.state {
                        ElementState::Pressed => {
                            self.pressed_keys.insert(key);
                        }
                        ElementState::Released => {
                            self.pressed_keys.remove(&key);
                        }
                    }
                }
            }
            WindowEvent::MouseInput { state, button, .. } => {
                if *button == MouseButton::Left {
                    self.cursor_captured = *state == ElementState::Pressed;
                    self.last_cursor_position = None;
                    self.look_delta = (0.0, 0.0);
                }
            }
            WindowEvent::CursorMoved { position, .. } => {
                if !self.cursor_captured {
                    return;
                }

                if let Some((last_x, last_y)) = self.last_cursor_position {
                    self.look_delta.1 += (position.x - last_x) as f32;
                    self.look_delta.0 += (position.y - last_y) as f32;
                }

                self.last_cursor_position = Some((position.x, position.y));
            }
            WindowEvent::Focused(false) => {
                self.cursor_captured = false;
                self.last_cursor_position = None;
                self.look_delta = (0.0, 0.0);
            }
            _ => {}
        }
    }

    pub fn frame_input(&mut self) -> FrameInput {
        let frame = FrameInput {
            pressed_keys: self.pressed_keys.clone(),
            look_delta: self.look_delta,
        };
        self.look_delta = (0.0, 0.0);
        frame
    }

    pub fn cursor_captured(&self) -> bool {
        self.cursor_captured
    }
}

pub struct Scene {
    entities: SlotMap<Entity, ()>,
    pub(crate) names: SecondaryMap<Entity, Name>,
    pub(crate) transforms: SecondaryMap<Entity, Transform>,
    pub(crate) cameras: SecondaryMap<Entity, Camera>,
    pub(crate) point_lights: SecondaryMap<Entity, PointLight>,
    pub(crate) directional_lights: SecondaryMap<Entity, DirectionalLight>,
    pub(crate) spot_lights: SecondaryMap<Entity, SpotLight>,
    pub(crate) mesh_instances: SecondaryMap<Entity, MeshInstance>,
    active_camera: Option<Entity>,
    ambient_color: Vector4<f32>,
}

impl Scene {
    pub fn new() -> Self {
        Self {
            entities: SlotMap::with_key(),
            names: SecondaryMap::new(),
            transforms: SecondaryMap::new(),
            cameras: SecondaryMap::new(),
            point_lights: SecondaryMap::new(),
            directional_lights: SecondaryMap::new(),
            spot_lights: SecondaryMap::new(),
            mesh_instances: SecondaryMap::new(),
            active_camera: None,
            ambient_color: Vector4::new(0.1, 0.1, 0.1, 0.15),
        }
    }

    pub fn spawn_empty(&mut self) -> Entity {
        self.entities.insert(())
    }

    pub fn despawn(&mut self, entity: Entity) {
        self.entities.remove(entity);
        self.names.remove(entity);
        self.transforms.remove(entity);
        self.cameras.remove(entity);
        self.point_lights.remove(entity);
        self.directional_lights.remove(entity);
        self.spot_lights.remove(entity);
        self.mesh_instances.remove(entity);
        if self.active_camera == Some(entity) {
            self.active_camera = None;
        }
    }

    pub fn spawn_camera(&mut self, bundle: CameraBundle) -> Entity {
        let entity = self.spawn_empty();
        self.insert_camera_bundle(entity, bundle);
        self.active_camera.get_or_insert(entity);
        entity
    }

    pub fn spawn_directional_light(&mut self, bundle: DirectionalLightBundle) -> Entity {
        let entity = self.spawn_empty();
        self.insert_directional_light_bundle(entity, bundle);
        entity
    }

    pub fn spawn_point_light(&mut self, bundle: PointLightBundle) -> Entity {
        let entity = self.spawn_empty();
        self.insert_point_light_bundle(entity, bundle);
        entity
    }

    pub fn spawn_spot_light(&mut self, bundle: SpotLightBundle) -> Entity {
        let entity = self.spawn_empty();
        self.insert_spot_light_bundle(entity, bundle);
        entity
    }

    pub fn spawn_mesh_instance(&mut self, bundle: MeshInstanceBundle) -> Entity {
        let entity = self.spawn_empty();
        self.insert_mesh_instance_bundle(entity, bundle);
        entity
    }

    pub fn insert_name(&mut self, entity: Entity, name: Name) {
        self.names.insert(entity, name);
    }

    pub fn insert_transform(&mut self, entity: Entity, transform: Transform) {
        self.transforms.insert(entity, transform);
    }

    pub fn insert_camera(&mut self, entity: Entity, camera: Camera) {
        self.cameras.insert(entity, camera);
    }

    pub fn insert_point_light(&mut self, entity: Entity, light: PointLight) {
        self.point_lights.insert(entity, light);
    }

    pub fn insert_directional_light(&mut self, entity: Entity, light: DirectionalLight) {
        self.directional_lights.insert(entity, light);
    }

    pub fn insert_spot_light(&mut self, entity: Entity, light: SpotLight) {
        self.spot_lights.insert(entity, light);
    }

    pub fn insert_mesh_instance_component(&mut self, entity: Entity, mesh: MeshInstance) {
        self.mesh_instances.insert(entity, mesh);
    }

    pub fn insert_camera_bundle(&mut self, entity: Entity, bundle: CameraBundle) {
        if let Some(name) = bundle.name {
            self.insert_name(entity, name);
        }
        self.insert_transform(entity, bundle.transform);
        self.insert_camera(entity, bundle.camera);
    }

    pub fn insert_directional_light_bundle(
        &mut self,
        entity: Entity,
        bundle: DirectionalLightBundle,
    ) {
        if let Some(name) = bundle.name {
            self.insert_name(entity, name);
        }
        self.insert_transform(entity, bundle.transform);
        self.insert_directional_light(entity, bundle.light);
    }

    pub fn insert_point_light_bundle(&mut self, entity: Entity, bundle: PointLightBundle) {
        if let Some(name) = bundle.name {
            self.insert_name(entity, name);
        }
        self.insert_transform(entity, bundle.transform);
        self.insert_point_light(entity, bundle.light);
    }

    pub fn insert_spot_light_bundle(&mut self, entity: Entity, bundle: SpotLightBundle) {
        if let Some(name) = bundle.name {
            self.insert_name(entity, name);
        }
        self.insert_transform(entity, bundle.transform);
        self.insert_spot_light(entity, bundle.light);
    }

    pub fn insert_mesh_instance_bundle(&mut self, entity: Entity, bundle: MeshInstanceBundle) {
        if let Some(name) = bundle.name {
            self.insert_name(entity, name);
        }
        self.insert_transform(entity, bundle.transform);
        self.insert_mesh_instance_component(entity, bundle.mesh_instance);
    }

    pub fn set_active_camera(&mut self, entity: Entity) {
        self.active_camera = Some(entity);
    }

    pub fn active_camera(&self) -> Option<Entity> {
        self.active_camera
    }

    pub fn ambient_color(&self) -> Vector4<f32> {
        self.ambient_color
    }

    pub fn set_ambient_color(&mut self, ambient_color: Vector4<f32>) {
        self.ambient_color = ambient_color;
    }

    pub fn transform_mut(&mut self, entity: Entity) -> Option<&mut Transform> {
        self.transforms.get_mut(entity)
    }

    pub fn update(&mut self, dt: f32, input: &FrameInput) {
        if let Some(active_camera) = self.active_camera {
            if let (Some(camera), Some(transform)) = (
                self.cameras.get(active_camera),
                self.transforms.get_mut(active_camera),
            ) {
                apply_camera_input(camera, transform, input, dt);
            }
        }

        for (_, transform) in self.transforms.iter_mut() {
            if transform.dirty {
                transform.update_cached_matrix();
            }
        }
    }

    pub(crate) fn active_camera_components(&self) -> Option<(Entity, &Transform, &Camera)> {
        let entity = self.active_camera?;
        let transform = self.transforms.get(entity)?;
        let camera = self.cameras.get(entity)?;
        Some((entity, transform, camera))
    }
}

impl Default for Scene {
    fn default() -> Self {
        Self::new()
    }
}

fn apply_camera_input(camera: &Camera, transform: &mut Transform, input: &FrameInput, dt: f32) {
    let (pitch_delta, yaw_delta) = input.look_delta();
    let rotation = transform.rotation_matrix();
    let forward = (rotation * Vector3::<f32>::new(0.0, 0.0, -1.0)).normalize();
    let up = Vector3::y();
    let right = forward.cross(&up).normalize();

    let mut position_delta = Vector3::new(0.0, 0.0, 0.0);
    let directions = [
        (KeyCode::KeyW, forward),
        (KeyCode::KeyS, -forward),
        (KeyCode::KeyA, -right),
        (KeyCode::KeyD, right),
        (KeyCode::Space, up),
        (KeyCode::AltLeft, -up),
    ];

    for (key, direction) in directions {
        if input.is_key_pressed(key) {
            position_delta += direction;
        }
    }

    let rotation_delta = Vector3::new(
        pitch_delta * dt * camera.rotation_speed,
        yaw_delta * dt * camera.rotation_speed,
        0.0,
    );

    if position_delta != Vector3::zeros() || rotation_delta != Vector3::zeros() {
        transform.position += position_delta * dt * camera.speed;
        transform.rotation += rotation_delta;
        transform.dirty = true;
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use winit::{
        dpi::PhysicalPosition,
        event::{DeviceId, WindowEvent},
        keyboard::KeyCode,
    };

    #[test]
    fn scene_update_refreshes_dirty_transform_cache() {
        let mut scene = Scene::new();
        let entity = scene.spawn_empty();
        scene.insert_transform(entity, Transform::identity());

        let transform = scene
            .transform_mut(entity)
            .expect("entity should have transform");
        transform.position = Vector3::new(1.0, 2.0, 3.0);
        transform.dirty = true;

        scene.update(0.016, &FrameInput::default());

        let transform = scene
            .transforms
            .get(entity)
            .expect("transform should exist");
        assert!(!transform.dirty);
        assert_eq!(transform.cached_matrix[(0, 3)], 1.0);
        assert_eq!(transform.cached_matrix[(1, 3)], 2.0);
        assert_eq!(transform.cached_matrix[(2, 3)], 3.0);
    }

    #[test]
    fn input_state_only_accumulates_look_delta_when_captured() {
        let mut input = InputState::new();
        let device_id = DeviceId::dummy();

        input.handle_window_event(&WindowEvent::CursorMoved {
            device_id,
            position: PhysicalPosition::new(10.0, 20.0),
        });
        assert_eq!(input.frame_input().look_delta(), (0.0, 0.0));

        input.handle_window_event(&WindowEvent::MouseInput {
            device_id,
            state: ElementState::Pressed,
            button: MouseButton::Left,
        });
        input.handle_window_event(&WindowEvent::CursorMoved {
            device_id,
            position: PhysicalPosition::new(10.0, 20.0),
        });
        input.handle_window_event(&WindowEvent::CursorMoved {
            device_id,
            position: PhysicalPosition::new(14.0, 32.0),
        });

        let frame = input.frame_input();
        assert_eq!(frame.look_delta(), (12.0, 4.0));
    }

    #[test]
    fn frame_input_reports_pressed_keys() {
        let mut input = InputState::new();
        input.pressed_keys.insert(KeyCode::KeyW);

        assert!(input.frame_input().is_key_pressed(KeyCode::KeyW));
    }
}
