use nalgebra::Vector4;
use slotmap::{SecondaryMap, SlotMap};

use super::{
    Camera, CameraBundle, DirectionalLight, DirectionalLightBundle, Entity, FrameInput,
    MeshInstance, MeshInstanceBundle, Name, PointLight, PointLightBundle, SpotLight,
    SpotLightBundle, Transform, Visibility, apply_camera_input,
};

pub struct Scene {
    entities: SlotMap<Entity, ()>,
    pub(crate) names: SecondaryMap<Entity, Name>,
    visibilities: SecondaryMap<Entity, Visibility>,
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
            visibilities: SecondaryMap::new(),
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
        let entity = self.entities.insert(());
        self.visibilities.insert(entity, Visibility::visible());
        entity
    }

    pub fn despawn(&mut self, entity: Entity) {
        self.entities.remove(entity);
        self.names.remove(entity);
        self.visibilities.remove(entity);
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

    pub fn entities(&self) -> impl Iterator<Item = Entity> + '_ {
        self.entities.keys()
    }

    pub fn name(&self, entity: Entity) -> Option<&Name> {
        self.names.get(entity)
    }

    pub fn name_mut(&mut self, entity: Entity) -> Option<&mut Name> {
        self.names.get_mut(entity)
    }

    pub fn is_visible(&self, entity: Entity) -> bool {
        self.visibilities
            .get(entity)
            .map(|visibility| visibility.visible)
            .unwrap_or(true)
    }

    pub fn visibility_mut(&mut self, entity: Entity) -> Option<&mut Visibility> {
        self.visibilities.get_mut(entity)
    }

    pub fn transform(&self, entity: Entity) -> Option<&Transform> {
        self.transforms.get(entity)
    }

    pub fn camera(&self, entity: Entity) -> Option<&Camera> {
        self.cameras.get(entity)
    }

    pub fn camera_mut(&mut self, entity: Entity) -> Option<&mut Camera> {
        self.cameras.get_mut(entity)
    }

    pub fn point_light_mut(&mut self, entity: Entity) -> Option<&mut PointLight> {
        self.point_lights.get_mut(entity)
    }

    pub fn directional_light_mut(&mut self, entity: Entity) -> Option<&mut DirectionalLight> {
        self.directional_lights.get_mut(entity)
    }

    pub fn spot_light_mut(&mut self, entity: Entity) -> Option<&mut SpotLight> {
        self.spot_lights.get_mut(entity)
    }

    pub fn mesh_instance(&self, entity: Entity) -> Option<&MeshInstance> {
        self.mesh_instances.get(entity)
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

#[cfg(test)]
mod tests {
    use nalgebra::Vector3;
    use winit::{
        dpi::PhysicalPosition,
        event::{DeviceId, ElementState, MouseButton, WindowEvent},
        keyboard::KeyCode,
    };

    use super::Scene;
    use crate::scene::{Camera, FrameInput, InputState, Transform};

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

    #[test]
    fn spawned_entities_are_visible_by_default() {
        let mut scene = Scene::new();
        let entity = scene.spawn_empty();

        assert!(scene.is_visible(entity));
        scene.visibility_mut(entity).unwrap().visible = false;
        assert!(!scene.is_visible(entity));
    }

    #[test]
    fn camera_projection_uses_camera_fields() {
        let transform = Transform::identity();
        let narrow = Camera::default().with_projection(45.0f32.to_radians(), 0.1, 1_000.0);
        let wide = Camera::default().with_projection(90.0f32.to_radians(), 0.1, 1_000.0);

        let (_, narrow_projection) = transform.view_projection(&narrow, 1920, 1080);
        let (_, wide_projection) = transform.view_projection(&wide, 1920, 1080);

        assert_ne!(narrow_projection[(1, 1)], wide_projection[(1, 1)]);
    }
}
