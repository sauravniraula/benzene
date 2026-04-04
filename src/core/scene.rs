use ash::vk;
use nalgebra::Vector4;
use std::{array::from_fn, mem::size_of};

use crate::{
    core::{
        assets::AssetStore,
        ecs::{
            components::{
                Camera3D, MeshRenderer, Name, PointLight3D, Transform3D,
                directional_light_3d::DirectionalLight3D, spot_light_3d::SpotLight3D,
            },
            entities::Entity,
            resources::SceneResources,
            systems::{camera_3d_apply_input, update_transform_3d_matrix},
            types::{CursorMovedEvent, KeyboardInputEvent},
            world::World,
        },
        gpu::{
            directional_light_uniform::{DirectionalLightUniform, DirectionalLightUniformObject},
            global_uniform::GlobalUniform,
            materials_manager::MaterialsManager,
            point_light_uniform::{PointLightUniform, PointLightUniformObject},
            scene_render::{DrawableSceneElement, RecordableScene, SceneRenderer},
            shadow_mapping::ShadowMapping,
            spot_light_uniform::{SpotLightUniform, SpotLightUniformObject},
        },
        model_push_constant::ModelPushConstant,
    },
    vulkan_backend::{
        backend::VBackend,
        backend_event::VBackendEvent,
        descriptor::{
            VDescriptorPool, VDescriptorSet, VDescriptorWriteBatch,
            config::{VDescriptorPoolConfig, VDescriptorPoolTypeConfig},
        },
        device::VDevice,
    },
};

pub struct Scene {
    default_descriptor_pool: VDescriptorPool,

    pub global_uniform_set: VDescriptorSet,
    pub lights_set: VDescriptorSet,
    pub world: World,
    pub resources: SceneResources,
    pub shadow_mapping: ShadowMapping,

    global_uniform: GlobalUniform,
    point_light_uniform: PointLightUniform,
    directional_light_uniform: DirectionalLightUniform,
    spot_light_uniform: SpotLightUniform,

    is_extent_dirty: bool,
    has_point_light_3d_changed: bool,
    has_directional_light_3d_changed: bool,
    has_spot_light_3d_changed: bool,
    current_extent: vk::Extent2D,
}

impl Scene {
    pub fn new(v_backend: &VBackend, scene_renderer: &SceneRenderer) -> Self {
        let default_descriptor_pool = VDescriptorPool::new(
            &v_backend.v_device,
            VDescriptorPoolConfig {
                types: vec![
                    VDescriptorPoolTypeConfig {
                        descriptor_type: vk::DescriptorType::UNIFORM_BUFFER,
                        count: 4,
                    },
                    VDescriptorPoolTypeConfig {
                        descriptor_type: vk::DescriptorType::COMBINED_IMAGE_SAMPLER,
                        count: 1,
                    },
                ],
                max_sets: 3,
            },
        );

        let global_uniform_set = VDescriptorSet::new(
            &v_backend.v_device,
            &default_descriptor_pool,
            scene_renderer.get_global_uniform_layout(),
        );
        let lights_set = VDescriptorSet::new(
            &v_backend.v_device,
            &default_descriptor_pool,
            scene_renderer.get_lights_uniform_layout(),
        );

        let global_uniform = GlobalUniform::new(v_backend, 1);
        let point_light_uniform = PointLightUniform::new(v_backend);
        let directional_light_uniform = DirectionalLightUniform::new(v_backend);
        let spot_light_uniform = SpotLightUniform::new(v_backend);

        {
            let mut batch = VDescriptorWriteBatch::new();
            global_uniform.queue_descriptor_writes(&global_uniform_set, &mut batch);
            point_light_uniform.queue_descriptor_writes(&lights_set, &mut batch);
            directional_light_uniform.queue_descriptor_writes(&lights_set, &mut batch);
            spot_light_uniform.queue_descriptor_writes(&lights_set, &mut batch);
            batch.flush(&v_backend.v_device);
        }

        let resources = SceneResources::new(Vector4::new(0.1, 0.1, 0.1, 0.15));

        let mut scene = Self {
            default_descriptor_pool,
            global_uniform_set,
            lights_set,
            world: World::new(),
            resources,
            shadow_mapping: ShadowMapping::new(),
            global_uniform,
            point_light_uniform,
            directional_light_uniform,
            spot_light_uniform,
            is_extent_dirty: false,
            has_point_light_3d_changed: false,
            has_directional_light_3d_changed: false,
            has_spot_light_3d_changed: false,
            current_extent: v_backend.v_swapchain.image_extent,
        };

        scene
            .global_uniform
            .update_ambient_color(v_backend, 0, &scene.resources.ambient_color);

        scene
    }

    pub fn handle_keyboard_input(&mut self, event: &KeyboardInputEvent) {
        self.resources.input.handle_keyboard_input(event);
    }

    pub fn handle_cursor_moved(&mut self, event: &CursorMovedEvent) {
        self.resources.input.handle_cursor_moved(event);
    }

    pub fn reset_cursor_tracking(&mut self) {
        self.resources.input.reset_cursor_tracking();
    }

    pub fn handle_backend_event(&mut self, event: &VBackendEvent) {
        if let VBackendEvent::UpdateFramebuffers(_, v_swapchain) = event {
            self.current_extent = v_swapchain.image_extent;
            self.is_extent_dirty = true;
        }
    }

    pub fn spawn_entity(&mut self) -> Entity {
        self.world.spawn()
    }

    pub fn get_transform_3d_component(&mut self, entity: Entity) -> &mut Transform3D {
        self.world
            .transforms
            .get_mut(&entity)
            .expect("failed to get transform 3d component from entity")
    }

    pub fn add_name_component(&mut self, entity: Entity, name: Name) {
        self.world.names.insert(entity, name);
    }

    pub fn add_transform_3d_component(&mut self, entity: Entity, transform_3d: Transform3D) {
        self.world.transforms.insert(entity, transform_3d);
    }

    pub fn add_camera_3d_component(&mut self, entity: Entity, camera: Camera3D) {
        self.world.cameras.insert(entity, camera);
        if self.resources.active_camera.is_none() {
            self.resources.active_camera = Some(entity);
        }
    }

    pub fn set_active_camera(&mut self, entity: Entity) {
        self.resources.active_camera = Some(entity);
    }

    pub fn add_point_light_3d_component(&mut self, entity: Entity, point_light: PointLight3D) {
        self.world.point_lights.insert(entity, point_light);
        self.has_point_light_3d_changed = true;
    }

    pub fn add_directional_light_3d_component(
        &mut self,
        entity: Entity,
        directional_light: DirectionalLight3D,
    ) {
        self.world
            .directional_lights
            .insert(entity, directional_light);
        self.has_directional_light_3d_changed = true;
    }

    pub fn add_spot_light_3d_component(&mut self, entity: Entity, spot_light: SpotLight3D) {
        self.world.spot_lights.insert(entity, spot_light);
        self.has_spot_light_3d_changed = true;
    }

    pub fn add_mesh_renderer_component(&mut self, entity: Entity, mesh_renderer: MeshRenderer) {
        self.world.mesh_renderers.insert(entity, mesh_renderer);
    }

    pub fn mark_directional_light_3d_dirty(&mut self) {
        self.has_directional_light_3d_changed = true;
    }

    pub fn pre_render(&mut self, v_backend: &VBackend, dt: f32) {
        self.update_global_uniform(v_backend, dt);

        for (entity, transform) in self.world.transforms.iter_mut() {
            if transform.dirty {
                update_transform_3d_matrix(transform);

                if self.world.point_lights.contains_key(entity) {
                    self.has_point_light_3d_changed = true;
                }
                if self.world.directional_lights.contains_key(entity) {
                    self.has_directional_light_3d_changed = true;
                }
                if self.world.spot_lights.contains_key(entity) {
                    self.has_spot_light_3d_changed = true;
                }
            }
        }

        self.update_point_light_uniform(v_backend);
        self.update_directional_light_uniform(v_backend);
        self.update_spot_light_uniform(v_backend);
    }

    fn update_global_uniform(&mut self, v_backend: &VBackend, dt: f32) {
        let Some(active_camera) = self.resources.active_camera else {
            return;
        };

        let Some(camera) = self.world.cameras.get(&active_camera) else {
            return;
        };

        let transform = self
            .world
            .transforms
            .get_mut(&active_camera)
            .expect("active camera is missing a transform");

        if self.is_extent_dirty
            || self.resources.input.has_pending_camera_input()
            || transform.dirty
        {
            camera_3d_apply_input(camera, transform, &mut self.resources.input, dt);

            let (view, projection) =
                transform.get_transform_3d_view_projection(self.current_extent);
            self.global_uniform.update_view(v_backend, 0, &view);
            self.global_uniform
                .update_projection(v_backend, 0, &projection);
            self.is_extent_dirty = false;
        }
    }

    fn update_point_light_uniform(&mut self, v_backend: &VBackend) {
        if !self.has_point_light_3d_changed {
            return;
        }

        let mut points = from_fn(|_| Vector4::new(0.0, 0.0, 0.0, 0.0));
        let mut colors = from_fn(|_| Vector4::new(0.0, 0.0, 0.0, 0.0));

        for (index, (entity, point_light)) in self.world.point_lights.iter().enumerate() {
            if index >= 16 {
                break;
            }

            if let Some(transform) = self.world.transforms.get(entity) {
                let p = transform.position;
                points[index] = Vector4::new(p.x, p.y, p.z, 1.0);
                colors[index] = point_light.color;
            }
        }

        let data = PointLightUniformObject { points, colors };
        self.point_light_uniform.update_all(v_backend, &data);
        self.has_point_light_3d_changed = false;
    }

    fn update_directional_light_uniform(&mut self, v_backend: &VBackend) {
        if !self.has_directional_light_3d_changed {
            return;
        }

        let mut directions = from_fn(|_| Vector4::new(0.0, 0.0, 0.0, 0.0));
        let mut colors = from_fn(|_| Vector4::new(0.0, 0.0, 0.0, 0.0));

        for (index, (entity, light)) in self.world.directional_lights.iter().enumerate() {
            if index >= 16 {
                break;
            }

            if let Some(transform) = self.world.transforms.get(entity) {
                let direction = transform.get_rotation3() * nalgebra::Vector3::new(0.0, 0.0, -1.0);
                directions[index] = Vector4::new(direction.x, direction.y, direction.z, 1.0);
                colors[index] = light.color;
            }
        }

        let data = DirectionalLightUniformObject { directions, colors };
        self.directional_light_uniform.update_all(v_backend, &data);
        self.has_directional_light_3d_changed = false;
    }

    fn update_spot_light_uniform(&mut self, v_backend: &VBackend) {
        if !self.has_spot_light_3d_changed {
            return;
        }

        let mut positions = from_fn(|_| Vector4::new(0.0, 0.0, 0.0, 0.0));
        let mut directions = from_fn(|_| Vector4::new(0.0, 0.0, 0.0, 0.0));
        let mut colors = from_fn(|_| Vector4::new(0.0, 0.0, 0.0, 0.0));

        for (index, (entity, light)) in self.world.spot_lights.iter().enumerate() {
            if index >= 16 {
                break;
            }

            if let Some(transform) = self.world.transforms.get(entity) {
                let direction = transform.get_rotation3() * nalgebra::Vector3::new(0.0, 0.0, -1.0);
                positions[index] = Vector4::new(
                    transform.position.x,
                    transform.position.y,
                    transform.position.z,
                    1.0,
                );
                directions[index] = Vector4::new(direction.x, direction.y, direction.z, 0.0);
                colors[index] = light.color;
            }
        }

        let data = SpotLightUniformObject {
            positions,
            directions,
            colors,
        };
        self.spot_light_uniform.update_all(v_backend, &data);
        self.has_spot_light_3d_changed = false;
    }

    pub fn destroy(&mut self, v_backend: &VBackend) {
        self.global_uniform.destroy(v_backend);
        self.point_light_uniform.destroy(v_backend);
        self.directional_light_uniform.destroy(v_backend);
        self.spot_light_uniform.destroy(v_backend);
        self.shadow_mapping.destroy(v_backend);
        self.default_descriptor_pool.destroy(&v_backend.v_device);
    }
}

impl RecordableScene for Scene {
    fn record_scene(
        &self,
        v_device: &VDevice,
        cmd: vk::CommandBuffer,
        assets: &AssetStore,
        materials_m: &MaterialsManager,
        scene_r: &SceneRenderer,
    ) {
        unsafe {
            v_device.device.cmd_bind_descriptor_sets(
                cmd,
                vk::PipelineBindPoint::GRAPHICS,
                *scene_r.get_pipeline_layout(),
                0,
                &[self.global_uniform_set.set, self.lights_set.set],
                &[],
            );
        }

        for (entity, mesh_renderer) in self.world.mesh_renderers.iter() {
            let Some(transform) = self.world.transforms.get(entity) else {
                continue;
            };
            let Some(mesh) = assets.get_mesh(mesh_renderer.mesh) else {
                continue;
            };

            let material_handle = mesh_renderer.material.unwrap_or(assets.default_material);

            unsafe {
                v_device.device.cmd_bind_descriptor_sets(
                    cmd,
                    vk::PipelineBindPoint::GRAPHICS,
                    *scene_r.get_pipeline_layout(),
                    2,
                    &[materials_m.get_set_at(material_handle).set],
                    &[],
                );
            }

            let push = ModelPushConstant {
                transform: transform.cached_transform,
            };
            let data = unsafe {
                std::slice::from_raw_parts(
                    (&push as *const ModelPushConstant) as *const u8,
                    size_of::<ModelPushConstant>(),
                )
            };

            unsafe {
                v_device.device.cmd_push_constants(
                    cmd,
                    *scene_r.get_pipeline_layout(),
                    vk::ShaderStageFlags::VERTEX | vk::ShaderStageFlags::FRAGMENT,
                    0,
                    data,
                );
            }

            mesh.draw(v_device, cmd);
        }
    }
}
