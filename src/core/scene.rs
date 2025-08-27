use crate::{
    constants::MAX_FRAMES_IN_FLIGHT,
    core::{
        ecs::{
            components::{Camera3D, PointLight3D, Structure3D, Transform3D},
            entities::game_object::GameObject,
            systems::{
                camera_3d_compute_transform, camera_3d_handle_window_event,
                get_camera_3d_view_projection, update_transform_3d_matrix,
            },
            types::EntityId,
        },
        gpu::{
            global_uniform::GlobalUniform,
            point_light_uniform::{PointLightUniform, PointLightUniformObject},
            recordable::{Drawable, RecordContext, Recordable},
            scene_render::SceneRender,
            texture::ImageTexture,
        },
        model_push_constant::ModelPushConstant,
    },
    vulkan_backend::{
        backend::VBackend,
        backend_event::VBackendEvent,
        descriptor::{
            VDescriptorPool, VDescriptorSets, VDescriptorWriteBatch,
            config::{VDescriptorPoolConfig, VDescriptorPoolTypeConfig},
        },
    },
};
use ash::vk;
use glfw::WindowEvent;
use nalgebra::Vector4;
use std::collections::HashMap;

pub struct Scene {
    descriptor_pool: VDescriptorPool,
    // Descriptor Sets
    pub global_uniform_sets: VDescriptorSets,
    pub point_light_sets: VDescriptorSets,
    pub image_sampler_sets: VDescriptorSets,

    // ECS
    active_camera: Option<EntityId>,
    global_uniform: GlobalUniform,
    point_light_uniform: PointLightUniform,
    entities: Vec<GameObject>,
    transform_3d_components: HashMap<EntityId, Transform3D>,
    camera_3d_components: HashMap<EntityId, Camera3D>,
    point_light_3d_components: HashMap<EntityId, PointLight3D>,
    structure_3d_components: HashMap<EntityId, Structure3D>,

    // Defaults
    texture: ImageTexture,

    // Status
    is_extent_dirty: bool,
    has_point_light_3d_changed: bool,

    // Others
    current_extent: vk::Extent2D,
    ambient_color: Vector4<f32>,
}

impl Scene {
    pub fn new(v_backend: &VBackend, scene_render: &SceneRender) -> Self {
        // Create descriptor pool using new config model
        let descriptor_pool = VDescriptorPool::new(
            &v_backend.v_device,
            &VDescriptorPoolConfig {
                types: vec![
                    VDescriptorPoolTypeConfig {
                        descriptor_type: vk::DescriptorType::UNIFORM_BUFFER,
                        // count: MAX_FRAMES_IN_FLIGHT as u32,
                        count: 1,
                    },
                    VDescriptorPoolTypeConfig {
                        descriptor_type: vk::DescriptorType::UNIFORM_BUFFER,
                        count: 1,
                    },
                    VDescriptorPoolTypeConfig {
                        descriptor_type: vk::DescriptorType::COMBINED_IMAGE_SAMPLER,
                        count: 1,
                    },
                ],
                max_sets: (MAX_FRAMES_IN_FLIGHT + 1) as u32,
            },
        );

        // Create defautl descriptor sets
        let global_uniform_sets = VDescriptorSets::new(
            &v_backend.v_device,
            &descriptor_pool,
            &scene_render.descriptor_sets_layouts[0],
            1,
        );
        let point_light_sets = VDescriptorSets::new(
            &v_backend.v_device,
            &descriptor_pool,
            &scene_render.descriptor_sets_layouts[1],
            1,
        );
        let image_sampler_sets = VDescriptorSets::new(
            &v_backend.v_device,
            &descriptor_pool,
            &scene_render.descriptor_sets_layouts[2],
            1,
        );

        // Attaching to descriptor sets
        let global_uniform = GlobalUniform::new(v_backend, 1);
        let point_light_uniform = PointLightUniform::new(v_backend);
        let texture = ImageTexture::new(
            v_backend,
            "assets/textures/cracked-dirt512x512.jpg",
            vk::Format::R8G8B8A8_SRGB,
        );
        {
            let mut batch = VDescriptorWriteBatch::new();
            global_uniform.queue_descriptor_writes(&global_uniform_sets, &mut batch);
            point_light_uniform.queue_descriptor_writes(&point_light_sets, &mut batch);
            texture.queue_descriptor_writes(&image_sampler_sets, &mut batch);

            batch.flush(&v_backend.v_device);
        }

        let mut scene = Self {
            descriptor_pool,
            global_uniform_sets,
            point_light_sets,
            image_sampler_sets,
            active_camera: None,
            global_uniform,
            point_light_uniform,
            entities: Vec::new(),
            transform_3d_components: HashMap::new(),
            camera_3d_components: HashMap::new(),
            point_light_3d_components: HashMap::new(),
            structure_3d_components: HashMap::new(),
            texture,
            is_extent_dirty: false,
            has_point_light_3d_changed: false,
            current_extent: v_backend.v_swapchain.image_extent,
            ambient_color: Vector4::new(0.1, 0.1, 0.1, 0.15),
        };

        scene
            .global_uniform
            .update_ambient_color(v_backend, 0, &scene.ambient_color);

        scene
    }

    pub fn handle_window_event(&mut self, event: &WindowEvent) {
        if let Some(active_id) = self.active_camera {
            if let Some(camera) = self.camera_3d_components.get_mut(&active_id) {
                camera_3d_handle_window_event(camera, event);
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

    pub fn add_structure_3d_component(&mut self, entity: &GameObject, structure: Structure3D) {
        let id = *entity.get_id();
        self.structure_3d_components.insert(id, structure);
    }

    pub fn pre_render(&mut self, v_backend: &VBackend, dt: f32) {
        self.update_global_uniform(v_backend, dt);

        // Update dirty Transforms 3D
        for t in self.transform_3d_components.values_mut() {
            if t.dirty {
                update_transform_3d_matrix(t);
            }
        }

        // Update point light uniform if needed
        self.update_point_light_uniform(v_backend);
    }

    pub fn update_global_uniform(&mut self, v_backend: &VBackend, dt: f32) {
        if let Some(active_camera_id) = self.active_camera {
            let camera_3d = self
                .camera_3d_components
                .get_mut(&active_camera_id)
                .expect("failed to get active camera from id");
            if self.is_extent_dirty || camera_3d.active_keys.len() > 0 || camera_3d.transform.dirty
            {
                camera_3d_compute_transform(camera_3d, dt);
                let (view, projection) =
                    get_camera_3d_view_projection(camera_3d, self.current_extent);
                self.global_uniform.update_view(v_backend, 0, &view);
                self.global_uniform
                    .update_projection(v_backend, 0, &projection);
            }
        }
    }

    pub fn update_point_light_uniform(&mut self, v_backend: &VBackend) {
        if self.has_point_light_3d_changed {
            let mut points: [Vector4<f32>; 16] = [Vector4::new(0.0, 0.0, 0.0, 0.0); 16];
            let mut colors: [Vector4<f32>; 16] = [Vector4::new(0.0, 0.0, 0.0, 0.0); 16];

            let mut index: usize = 0;
            for (entity_id, point_light) in self.point_light_3d_components.iter() {
                if index >= 16 {
                    break;
                }

                if let Some(light_transform) = self.transform_3d_components.get(entity_id) {
                    let p = light_transform.position;
                    points[index] = Vector4::new(p.x, p.y, p.z, 1.0);
                }

                colors[index] = point_light.color;

                index += 1;
            }

            let ubo = PointLightUniformObject { points, colors };
            self.point_light_uniform.upload(v_backend, &ubo);

            self.has_point_light_3d_changed = false;
        }
    }

    pub fn destroy(&self, v_backend: &VBackend) {
        self.global_uniform.destroy(v_backend);
        self.point_light_uniform.destroy(v_backend);
        for (_, structure) in self.structure_3d_components.iter() {
            structure.destroy(v_backend);
        }
        self.texture.destroy(v_backend);
        self.descriptor_pool.destroy(&v_backend.v_device);
    }
}

impl Recordable for Scene {
    fn record(&self, ctx: &RecordContext) {
        unsafe {
            ctx.v_device.device.cmd_bind_descriptor_sets(
                ctx.cmd,
                vk::PipelineBindPoint::GRAPHICS,
                ctx.pipeline_infos[0].layout,
                0,
                &[
                    self.global_uniform_sets.sets[0],
                    self.point_light_sets.sets[0],
                    self.image_sampler_sets.sets[0],
                ],
                &[],
            );
        }

        for (entity_id, structure) in self.structure_3d_components.iter() {
            if let Some(transform3d) = self.transform_3d_components.get(entity_id) {
                let push = ModelPushConstant {
                    transform: transform3d.cached_transform,
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
                        ctx.pipeline_infos[0].layout,
                        vk::ShaderStageFlags::VERTEX | vk::ShaderStageFlags::FRAGMENT,
                        0,
                        data,
                    );
                }
                structure.model.draw(ctx.v_device, ctx.cmd);
            }
        }
    }
}
