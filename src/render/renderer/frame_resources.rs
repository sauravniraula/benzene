use std::{mem::size_of, sync::Arc};

use ash::vk;
use nalgebra::{Matrix4, Vector4};

use crate::{error::Result, render::vulkan::VContext, scene::Scene};

use super::{
    DirectionalLightUniform, GlobalUniform, PointLightUniform, SpotLightUniform,
    allocate_descriptor_set, create_mapped_uniform_buffer,
};

pub(super) struct FrameResources {
    context: Arc<VContext>,
    global_buffer: vk::Buffer,
    global_memory: vk::DeviceMemory,
    global_mapped: *mut GlobalUniform,
    point_buffer: vk::Buffer,
    point_memory: vk::DeviceMemory,
    point_mapped: *mut PointLightUniform,
    directional_buffer: vk::Buffer,
    directional_memory: vk::DeviceMemory,
    directional_mapped: *mut DirectionalLightUniform,
    spot_buffer: vk::Buffer,
    spot_memory: vk::DeviceMemory,
    spot_mapped: *mut SpotLightUniform,
    pub(super) global_descriptor_set: vk::DescriptorSet,
    pub(super) lights_descriptor_set: vk::DescriptorSet,
}

impl FrameResources {
    pub(super) fn new(
        context: Arc<VContext>,
        global_layout: vk::DescriptorSetLayout,
        lights_layout: vk::DescriptorSetLayout,
        descriptor_pool: vk::DescriptorPool,
    ) -> Result<Self> {
        let (global_buffer, global_memory, global_mapped) =
            create_mapped_uniform_buffer::<GlobalUniform>(&context)?;
        let (point_buffer, point_memory, point_mapped) =
            create_mapped_uniform_buffer::<PointLightUniform>(&context)?;
        let (directional_buffer, directional_memory, directional_mapped) =
            create_mapped_uniform_buffer::<DirectionalLightUniform>(&context)?;
        let (spot_buffer, spot_memory, spot_mapped) =
            create_mapped_uniform_buffer::<SpotLightUniform>(&context)?;

        let global_descriptor_set =
            allocate_descriptor_set(context.device(), descriptor_pool, global_layout)?;
        let lights_descriptor_set =
            allocate_descriptor_set(context.device(), descriptor_pool, lights_layout)?;

        let global_buffer_info = vk::DescriptorBufferInfo::default()
            .buffer(global_buffer)
            .offset(0)
            .range(size_of::<GlobalUniform>() as u64);
        let point_buffer_info = vk::DescriptorBufferInfo::default()
            .buffer(point_buffer)
            .offset(0)
            .range(size_of::<PointLightUniform>() as u64);
        let directional_buffer_info = vk::DescriptorBufferInfo::default()
            .buffer(directional_buffer)
            .offset(0)
            .range(size_of::<DirectionalLightUniform>() as u64);
        let spot_buffer_info = vk::DescriptorBufferInfo::default()
            .buffer(spot_buffer)
            .offset(0)
            .range(size_of::<SpotLightUniform>() as u64);

        let descriptor_writes = [
            vk::WriteDescriptorSet::default()
                .dst_set(global_descriptor_set)
                .dst_binding(0)
                .descriptor_type(vk::DescriptorType::UNIFORM_BUFFER)
                .buffer_info(std::slice::from_ref(&global_buffer_info)),
            vk::WriteDescriptorSet::default()
                .dst_set(lights_descriptor_set)
                .dst_binding(0)
                .descriptor_type(vk::DescriptorType::UNIFORM_BUFFER)
                .buffer_info(std::slice::from_ref(&point_buffer_info)),
            vk::WriteDescriptorSet::default()
                .dst_set(lights_descriptor_set)
                .dst_binding(1)
                .descriptor_type(vk::DescriptorType::UNIFORM_BUFFER)
                .buffer_info(std::slice::from_ref(&directional_buffer_info)),
            vk::WriteDescriptorSet::default()
                .dst_set(lights_descriptor_set)
                .dst_binding(2)
                .descriptor_type(vk::DescriptorType::UNIFORM_BUFFER)
                .buffer_info(std::slice::from_ref(&spot_buffer_info)),
        ];

        unsafe {
            context
                .device()
                .update_descriptor_sets(&descriptor_writes, &[]);
        }

        Ok(Self {
            context,
            global_buffer,
            global_memory,
            global_mapped,
            point_buffer,
            point_memory,
            point_mapped,
            directional_buffer,
            directional_memory,
            directional_mapped,
            spot_buffer,
            spot_memory,
            spot_mapped,
            global_descriptor_set,
            lights_descriptor_set,
        })
    }

    pub(super) fn write_scene_uniforms(&mut self, scene: &Scene, width: u32, height: u32) {
        let mut global = GlobalUniform {
            view: Matrix4::identity(),
            projection: Matrix4::identity(),
            ambient_color: scene.ambient_color(),
        };
        if let Some((_, transform, camera)) = scene.active_camera_components() {
            let (view, projection) = transform.view_projection(camera, width, height);
            global.view = view;
            global.projection = projection;
        }

        unsafe {
            std::ptr::copy_nonoverlapping(&global, self.global_mapped, 1);
        }

        let mut point = PointLightUniform {
            points: [Vector4::zeros(); 16],
            colors: [Vector4::zeros(); 16],
        };
        let mut point_index = 0usize;
        for (entity, light) in scene.point_lights.iter() {
            if point_index >= 16 {
                break;
            }
            if !scene.is_visible(entity) {
                continue;
            }
            if let Some(transform) = scene.transforms.get(entity) {
                point.points[point_index] = Vector4::new(
                    transform.position.x,
                    transform.position.y,
                    transform.position.z,
                    1.0,
                );
                point.colors[point_index] = light.color;
                point_index += 1;
            }
        }
        unsafe {
            std::ptr::copy_nonoverlapping(&point, self.point_mapped, 1);
        }

        let mut directional = DirectionalLightUniform {
            directions: [Vector4::zeros(); 16],
            colors: [Vector4::zeros(); 16],
        };
        let mut directional_index = 0usize;
        for (entity, light) in scene.directional_lights.iter() {
            if directional_index >= 16 {
                break;
            }
            if !scene.is_visible(entity) {
                continue;
            }
            if let Some(transform) = scene.transforms.get(entity) {
                let direction =
                    transform.rotation_matrix() * nalgebra::Vector3::new(0.0, 0.0, -1.0);
                directional.directions[directional_index] =
                    Vector4::new(direction.x, direction.y, direction.z, 1.0);
                directional.colors[directional_index] = light.color;
                directional_index += 1;
            }
        }
        unsafe {
            std::ptr::copy_nonoverlapping(&directional, self.directional_mapped, 1);
        }

        let mut spot = SpotLightUniform {
            positions: [Vector4::zeros(); 16],
            directions: [Vector4::zeros(); 16],
            colors: [Vector4::zeros(); 16],
        };
        let mut spot_index = 0usize;
        for (entity, light) in scene.spot_lights.iter() {
            if spot_index >= 16 {
                break;
            }
            if !scene.is_visible(entity) {
                continue;
            }
            if let Some(transform) = scene.transforms.get(entity) {
                let direction =
                    transform.rotation_matrix() * nalgebra::Vector3::new(0.0, 0.0, -1.0);
                spot.positions[spot_index] = Vector4::new(
                    transform.position.x,
                    transform.position.y,
                    transform.position.z,
                    1.0,
                );
                spot.directions[spot_index] =
                    Vector4::new(direction.x, direction.y, direction.z, 0.0);
                spot.colors[spot_index] = light.color;
                spot_index += 1;
            }
        }
        unsafe {
            std::ptr::copy_nonoverlapping(&spot, self.spot_mapped, 1);
        }
    }
}

impl Drop for FrameResources {
    fn drop(&mut self) {
        unsafe {
            self.context.device().unmap_memory(self.global_memory);
            self.context.device().unmap_memory(self.point_memory);
            self.context.device().unmap_memory(self.directional_memory);
            self.context.device().unmap_memory(self.spot_memory);
        }
        self.context
            .destroy_buffer(self.global_buffer, self.global_memory);
        self.context
            .destroy_buffer(self.point_buffer, self.point_memory);
        self.context
            .destroy_buffer(self.directional_buffer, self.directional_memory);
        self.context
            .destroy_buffer(self.spot_buffer, self.spot_memory);
    }
}
