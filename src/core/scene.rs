use crate::{
    core::{
        ecs::{
            components::{
                Camera3D, Material3D, PointLight3D, Structure3D, Transform3D,
                directional_light_3d::DirectionalLight3D, spot_light_3d::SpotLight3D,
            },
            entities::game_object::GameObject,
            systems::{
                camera_3d_compute_transform, camera_3d_handle_cm_event, camera_3d_handle_ki_event,
                update_transform_3d_matrix,
            },
            types::{CursorMovedEvent, KeyboardInputEvent},
        },
        gpu::{
            directional_light_uniform::DirectionalLightUniform,
            global_uniform::GlobalUniform,
            materials_manager::MaterialsManager,
            point_light_uniform::PointLightUniform,
            scene_render::{DrawableSceneElement, RecordableScene, SceneRenderer},
            shadow_mapping::ShadowMapping,
            spot_light_uniform::SpotLightUniform,
            texture::ImageTexture,
        },
        model_push_constant::ModelPushConstant,
    },
    shared::types::Id,
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
use ash::vk;
use nalgebra::Vector4;
use std::collections::HashMap;
use std::mem::size_of;
use winit::{event::ElementState, keyboard::PhysicalKey};

pub struct Scene {
    default_descriptor_pool: VDescriptorPool,

    // Default Descriptor Sets
    pub global_uniform_set: VDescriptorSet,
    pub lights_set: VDescriptorSet,

    // ECS
    active_camera: Option<Id>,
    global_uniform: GlobalUniform,
    point_light_uniform: PointLightUniform,
    directional_light_uniform: DirectionalLightUniform,
    spot_light_uniform: SpotLightUniform,
    entities: Vec<GameObject>,
    transform_3d_components: HashMap<Id, Transform3D>,
    camera_3d_components: HashMap<Id, Camera3D>,
    point_light_3d_components: HashMap<Id, PointLight3D>,
    directional_light_3d_components: HashMap<Id, DirectionalLight3D>,
    spot_light_3d_components: HashMap<Id, SpotLight3D>,
    structure_3d_components: HashMap<Id, Structure3D>,
    material_3d_components: HashMap<Id, Material3D>,

    // Defaults
    texture: ImageTexture,

    // Shadow Mapping
    pub shadow_mapping: ShadowMapping,

    // Status
    is_extent_dirty: bool,
    has_point_light_3d_changed: bool,
    has_directional_light_3d_changed: bool,
    has_spot_light_3d_changed: bool,

    // Others
    current_extent: vk::Extent2D,
    ambient_color: Vector4<f32>,
}

impl Scene {
    pub fn new(v_backend: &VBackend, scene_renderer: &SceneRenderer) -> Self {
        // Default descriptor pool
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
                max_sets: 3 as u32,
            },
        );

        // Create default descriptor sets
        let global_uniform_set = VDescriptorSet::new(
            &v_backend.v_device,
            &default_descriptor_pool,
            &scene_renderer.get_global_uniform_layout(),
        );
        let lights_set = VDescriptorSet::new(
            &v_backend.v_device,
            &default_descriptor_pool,
            &scene_renderer.get_lights_uniform_layout(),
        );

        // Attaching to descriptor sets
        let global_uniform = GlobalUniform::new(v_backend, 1);
        let point_light_uniform = PointLightUniform::new(v_backend);
        let directional_light_uniform = DirectionalLightUniform::new(v_backend);
        let spot_light_uniform = SpotLightUniform::new(v_backend);
        let texture = ImageTexture::empty(v_backend, vk::Format::R8G8B8A8_SRGB);
        {
            let mut batch = VDescriptorWriteBatch::new();
            global_uniform.queue_descriptor_writes(&global_uniform_set, &mut batch);
            point_light_uniform.queue_descriptor_writes(&lights_set, &mut batch);
            directional_light_uniform.queue_descriptor_writes(&lights_set, &mut batch);
            spot_light_uniform.queue_descriptor_writes(&lights_set, &mut batch);
            batch.flush(&v_backend.v_device);
        }

        let mut scene = Self {
            default_descriptor_pool,
            global_uniform_set,
            lights_set,
            active_camera: None,
            global_uniform,
            point_light_uniform,
            directional_light_uniform,
            spot_light_uniform,
            entities: Vec::new(),
            transform_3d_components: HashMap::new(),
            camera_3d_components: HashMap::new(),
            point_light_3d_components: HashMap::new(),
            directional_light_3d_components: HashMap::new(),
            spot_light_3d_components: HashMap::new(),
            structure_3d_components: HashMap::new(),
            material_3d_components: HashMap::new(),
            texture,
            shadow_mapping: ShadowMapping::new(),
            is_extent_dirty: false,
            has_point_light_3d_changed: false,
            has_directional_light_3d_changed: false,
            has_spot_light_3d_changed: false,
            current_extent: v_backend.v_swapchain.image_extent,
            ambient_color: Vector4::new(0.1, 0.1, 0.1, 0.15),
        };

        scene
            .global_uniform
            .update_ambient_color(v_backend, 0, &scene.ambient_color);

        scene
    }

    pub fn handle_keyboard_input(&mut self, event: &KeyboardInputEvent) {
        if let Some(active_id) = self.active_camera {
            if let Some(camera) = self.camera_3d_components.get_mut(&active_id) {
                camera_3d_handle_ki_event(camera, event);
            }
        }
    }

    pub fn handle_cursor_moved(&mut self, event: &CursorMovedEvent) {
        if let Some(active_id) = self.active_camera {
            if let Some(camera) = self.camera_3d_components.get_mut(&active_id) {
                camera_3d_handle_cm_event(camera, event);
            }
        }
    }

    pub fn handle_backend_event(&mut self, event: &VBackendEvent) {
        match event {
            VBackendEvent::UpdateFramebuffers(_, v_swapchain) => {
                self.current_extent = v_swapchain.image_extent;
                self.is_extent_dirty = true;
            }
            _ => (),
        }
    }

    pub fn get_transform_3d_component(&mut self, entity: &GameObject) -> &mut Transform3D {
        self.transform_3d_components
            .get_mut(entity.get_id())
            .expect("failed to get transform 3d component from entity")
    }

    pub fn add_game_object(&mut self, entity: GameObject) {
        self.entities.push(entity);
    }

    pub fn add_transform_3d_component(&mut self, entity: &GameObject, transform3d: Transform3D) {
        self.transform_3d_components
            .insert(*entity.get_id(), transform3d);
    }

    pub fn add_camera_3d_component(&mut self, entity: &GameObject, camera: Camera3D) {
        let id = *entity.get_id();
        self.camera_3d_components.insert(id, camera);
        if self.active_camera.is_none() {
            self.active_camera = Some(id);
        }
    }

    pub fn set_active_camera(&mut self, entity: &GameObject) {
        self.active_camera = Some(*entity.get_id());
    }

    pub fn add_point_light_3d_component(&mut self, entity: &GameObject, point_light: PointLight3D) {
        let id = *entity.get_id();
        self.point_light_3d_components.insert(id, point_light);
        self.has_point_light_3d_changed = true;
    }

    pub fn add_directional_light_3d_component(
        &mut self,
        entity: &GameObject,
        directional_light: DirectionalLight3D,
    ) {
        let id = *entity.get_id();
        self.directional_light_3d_components
            .insert(id, directional_light);
        self.has_directional_light_3d_changed = true;
    }

    pub fn add_spot_light_3d_component(&mut self, entity: &GameObject, spot_light: SpotLight3D) {
        let id = *entity.get_id();
        self.spot_light_3d_components.insert(id, spot_light);
        self.has_spot_light_3d_changed = true;
    }

    pub fn add_structure_3d_component(&mut self, entity: &GameObject, structure: Structure3D) {
        let id = *entity.get_id();
        self.structure_3d_components.insert(id, structure);
    }

    pub fn add_material_3d_component(&mut self, entity: &GameObject, material: Material3D) {
        let id = *entity.get_id();
        self.material_3d_components.insert(id, material);
    }

    pub fn mark_directional_light_3d_dirty(&mut self) {
        self.has_directional_light_3d_changed = true;
    }

    pub fn pre_render(&mut self, v_backend: &VBackend, dt: f32) {
        self.update_global_uniform(v_backend, dt);

        // Update dirty Transforms 3D and flag light uniform updates if any light moved
        for (entity_id, t) in self.transform_3d_components.iter_mut() {
            if t.dirty {
                update_transform_3d_matrix(t);
                if self.point_light_3d_components.contains_key(entity_id) {
                    self.has_point_light_3d_changed = true;
                }
            }
        }

        // Update point light uniform if needed
        self.update_point_light_uniform(v_backend);

        // Update directional light uniform if needed
        self.update_directional_light_uniform(v_backend);

        // Update spot light uniform if needed
        self.update_spot_light_uniform(v_backend);
    }

    pub fn update_global_uniform(&mut self, v_backend: &VBackend, dt: f32) {
        if let Some(active_camera_id) = self.active_camera {
            let camera_3d = self
                .camera_3d_components
                .get_mut(&active_camera_id)
                .expect("failed to get active camera from id");
            if self.is_extent_dirty
                || camera_3d.ki_events.len() > 0
                || camera_3d.cm_events.len() > 0
                || camera_3d.transform.dirty
            {
                camera_3d_compute_transform(camera_3d, dt);
                let (view, projection) = camera_3d
                    .transform
                    .get_transform_3d_view_projection(self.current_extent);
                camera_3d.transform.dirty = false;
                self.global_uniform.update_view(v_backend, 0, &view);
                self.global_uniform
                    .update_projection(v_backend, 0, &projection);
            }
        }
    }

    pub fn update_point_light_uniform(&mut self, v_backend: &VBackend) {
        if self.has_point_light_3d_changed {
            let mut index: usize = 0;
            for (entity_id, point_light) in self.point_light_3d_components.iter() {
                if index >= 16 {
                    break;
                }

                if let Some(light_transform) = self.transform_3d_components.get(entity_id) {
                    let p = light_transform.position;
                    let point = Vector4::new(p.x, p.y, p.z, 1.0);
                    self.point_light_uniform
                        .update(v_backend, index, &point, &point_light.color);
                }
                index += 1;
            }
            self.has_point_light_3d_changed = false;
        }
    }

    pub fn update_directional_light_uniform(&mut self, v_backend: &VBackend) {
        if self.has_directional_light_3d_changed {
            let mut index: usize = 0;
            for (entity_id, directional_light) in self.directional_light_3d_components.iter() {
                if index >= 16 {
                    break;
                }

                if let Some(light_transform) = self.transform_3d_components.get(entity_id) {
                    let direction_raw = (light_transform.get_unit_quaternion()
                        * nalgebra::Vector3::new(0.0, 0.0, -1.0))
                    .to_homogeneous();
                    let direction =
                        Vector4::new(direction_raw.x, direction_raw.y, direction_raw.z, 1.0);

                    self.directional_light_uniform.update(
                        v_backend,
                        index,
                        &direction,
                        &directional_light.color,
                    );
                }
                index += 1;
            }
            self.has_directional_light_3d_changed = false;
        }
    }

    pub fn update_spot_light_uniform(&mut self, v_backend: &VBackend) {
        if self.has_spot_light_3d_changed {
            let mut index: usize = 0;
            for (entity_id, spot_light) in self.spot_light_3d_components.iter() {
                if index >= 16 {
                    break;
                }

                if let Some(light_transform) = self.transform_3d_components.get(entity_id) {
                    let position = Vector4::new(
                        light_transform.position.x,
                        light_transform.position.y,
                        light_transform.position.z,
                        1.0,
                    );
                    self.spot_light_uniform.update(
                        v_backend,
                        index,
                        &position,
                        &(light_transform.get_unit_quaternion()
                            * nalgebra::Vector3::new(0.0, 0.0, -1.0))
                        .to_homogeneous(),
                        &spot_light.color,
                    );
                }
                index += 1;
            }
            self.has_spot_light_3d_changed = false;
        }
    }

    pub fn destroy(&self, v_backend: &VBackend) {
        self.global_uniform.destroy(v_backend);
        self.point_light_uniform.destroy(v_backend);
        self.directional_light_uniform.destroy(v_backend);
        self.spot_light_uniform.destroy(v_backend);
        for (_, structure_3d) in self.structure_3d_components.iter() {
            structure_3d.destroy(v_backend);
        }
        self.texture.destroy(v_backend);
        self.default_descriptor_pool.destroy(&v_backend.v_device);
    }
}

impl RecordableScene for Scene {
    fn record_scene(
        &self,
        v_device: &VDevice,
        cmd: vk::CommandBuffer,
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

        for (entity_id, structure_3d) in self.structure_3d_components.iter() {
            if let Some(transform_3d) = self.transform_3d_components.get(entity_id) {
                let material_3d_index = match self.material_3d_components.get(entity_id) {
                    Some(material_3d) => material_3d.manager_index,
                    None => 0,
                };

                unsafe {
                    v_device.device.cmd_bind_descriptor_sets(
                        cmd,
                        vk::PipelineBindPoint::GRAPHICS,
                        *scene_r.get_pipeline_layout(),
                        2,
                        &[materials_m.get_set_at(material_3d_index).set],
                        &[],
                    );
                }

                let push = ModelPushConstant {
                    transform: transform_3d.cached_transform,
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
                structure_3d.model.draw(v_device, cmd);
            }
        }
    }
}
