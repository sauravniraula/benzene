mod egui_pass;
mod frame_resources;
mod geometry_pass;

use std::{mem::size_of, sync::Arc};

use ash::vk;
use nalgebra::{Matrix4, Vector4};
use winit::window::Window;

use crate::{
    assets::AssetManager,
    error::{EngineError, Result},
    render::vulkan::VContext,
    scene::{Scene, Transform},
    ui::{EguiFrame, SceneViewport},
};

use self::{egui_pass::EguiPass, frame_resources::FrameResources, geometry_pass::GeometryPass};

#[repr(C)]
#[derive(Clone, Copy)]
struct GlobalUniform {
    view: Matrix4<f32>,
    projection: Matrix4<f32>,
    ambient_color: Vector4<f32>,
}

#[repr(C)]
#[derive(Clone, Copy)]
struct PointLightUniform {
    points: [Vector4<f32>; 16],
    colors: [Vector4<f32>; 16],
}

#[repr(C)]
#[derive(Clone, Copy)]
struct DirectionalLightUniform {
    directions: [Vector4<f32>; 16],
    colors: [Vector4<f32>; 16],
}

#[repr(C)]
#[derive(Clone, Copy)]
struct SpotLightUniform {
    positions: [Vector4<f32>; 16],
    directions: [Vector4<f32>; 16],
    colors: [Vector4<f32>; 16],
}

#[repr(C)]
#[derive(Clone, Copy)]
struct ModelPushConstants {
    transform: Matrix4<f32>,
}

pub struct Renderer {
    egui: EguiPass,
    geometry: GeometryPass,
    frames: Vec<FrameResources>,
    context: Arc<VContext>,
}

impl Renderer {
    pub fn new(window: &Window) -> Result<Self> {
        let context = VContext::new(window)?;
        let mut geometry = GeometryPass::new(
            Arc::clone(&context),
            context.swapchain_format(),
            context.depth_format(),
        )?;
        geometry.rebuild_framebuffers(&context)?;
        let egui = EguiPass::new(
            Arc::clone(&context),
            geometry.render_pass,
            context.swapchain_format(),
        )?;

        let mut frames = Vec::with_capacity(crate::constants::MAX_FRAMES_IN_FLIGHT);
        for _ in 0..crate::constants::MAX_FRAMES_IN_FLIGHT {
            frames.push(FrameResources::new(
                Arc::clone(&context),
                geometry.global_layout,
                geometry.lights_layout,
                geometry.scene_descriptor_pool,
            )?);
        }

        Ok(Self {
            egui,
            geometry,
            frames,
            context,
        })
    }

    pub fn material_layout(&self) -> vk::DescriptorSetLayout {
        self.geometry.material_layout()
    }

    pub(crate) fn context(&self) -> Arc<VContext> {
        Arc::clone(&self.context)
    }

    pub fn wait_idle(&self) {
        self.context.wait_idle();
    }

    pub fn resize(&mut self, window: &Window) -> Result<()> {
        self.context.recreate_swapchain(window)?;
        if !self.context.is_zero_sized() {
            self.geometry.rebuild_framebuffers(&self.context)?;
        }
        Ok(())
    }

    pub fn render(&mut self, scene: &Scene, assets: &AssetManager) -> Result<()> {
        self.render_frame(scene, assets, None)
    }

    pub fn render_with_egui(
        &mut self,
        scene: &Scene,
        assets: &AssetManager,
        egui_frame: &EguiFrame,
    ) -> Result<()> {
        self.render_frame(scene, assets, Some(egui_frame))
    }

    fn render_frame(
        &mut self,
        scene: &Scene,
        assets: &AssetManager,
        egui_frame: Option<&EguiFrame>,
    ) -> Result<()> {
        if self.context.is_zero_sized() {
            return Ok(());
        }

        let (frame_index, image_index, command_buffer) = self.context.begin_frame()?;
        let extent = self.context.extent();
        let scene_viewport = egui_frame
            .map(|frame| frame.scene_viewport)
            .unwrap_or_else(|| SceneViewport::full(extent.width, extent.height))
            .clamped(extent.width, extent.height);
        self.frames[frame_index].write_scene_uniforms(
            scene,
            scene_viewport.width,
            scene_viewport.height,
        );

        self.geometry.begin(
            command_buffer,
            self.geometry.framebuffers[image_index],
            extent,
            scene_viewport,
        );

        unsafe {
            let frame = &self.frames[frame_index];
            self.context.device().cmd_bind_descriptor_sets(
                command_buffer,
                vk::PipelineBindPoint::GRAPHICS,
                self.geometry.pipeline_layout,
                0,
                &[frame.global_descriptor_set, frame.lights_descriptor_set],
                &[],
            );
        }

        let device = self.context.device();
        for (entity, mesh_instance) in scene.mesh_instances.iter() {
            if !scene.is_visible(entity) {
                continue;
            }
            let Some(transform) = scene.transforms.get(entity) else {
                continue;
            };
            let Some(mesh) = assets.mesh(mesh_instance.mesh) else {
                continue;
            };
            let material_id = mesh_instance.material.unwrap_or(assets.default_material());
            let Some(material_set) = assets.material_descriptor_set(material_id) else {
                continue;
            };

            unsafe {
                device.cmd_bind_descriptor_sets(
                    command_buffer,
                    vk::PipelineBindPoint::GRAPHICS,
                    self.geometry.pipeline_layout,
                    2,
                    &[material_set],
                    &[],
                );
            }

            let push_constants = ModelPushConstants {
                transform: current_transform_matrix(transform),
            };
            let push_bytes = unsafe {
                std::slice::from_raw_parts(
                    (&push_constants as *const ModelPushConstants) as *const u8,
                    size_of::<ModelPushConstants>(),
                )
            };

            unsafe {
                device.cmd_push_constants(
                    command_buffer,
                    self.geometry.pipeline_layout,
                    vk::ShaderStageFlags::VERTEX | vk::ShaderStageFlags::FRAGMENT,
                    0,
                    push_bytes,
                );
                device.cmd_bind_vertex_buffers(command_buffer, 0, &[mesh.vertex_buffer], &[0]);
                device.cmd_bind_index_buffer(
                    command_buffer,
                    mesh.index_buffer,
                    0,
                    vk::IndexType::UINT32,
                );
                device.cmd_draw_indexed(command_buffer, mesh.index_count, 1, 0, 0, 0);
            }
        }

        if let Some(egui_frame) = egui_frame {
            self.egui
                .draw(frame_index, command_buffer, extent, egui_frame)?;
        }

        self.geometry.end(command_buffer);
        self.context.end_frame(image_index, command_buffer)
    }
}

fn current_transform_matrix(transform: &Transform) -> Matrix4<f32> {
    if transform.dirty {
        let rotation = transform.rotation_matrix();
        let scale = Matrix4::new_nonuniform_scaling(&transform.scale);
        let translation = nalgebra::Translation3::new(
            transform.position.x,
            transform.position.y,
            transform.position.z,
        )
        .to_homogeneous();
        translation * rotation.to_homogeneous() * scale
    } else {
        transform.cached_matrix
    }
}

fn create_mapped_uniform_buffer<T>(
    context: &Arc<VContext>,
) -> Result<(vk::Buffer, vk::DeviceMemory, *mut T)> {
    let (buffer, memory) = context.create_buffer(
        size_of::<T>() as u64,
        vk::BufferUsageFlags::UNIFORM_BUFFER,
        vk::MemoryPropertyFlags::HOST_VISIBLE | vk::MemoryPropertyFlags::HOST_COHERENT,
    )?;
    let mapped = unsafe {
        context
            .device()
            .map_memory(
                memory,
                0,
                size_of::<T>() as u64,
                vk::MemoryMapFlags::empty(),
            )
            .map_err(|result| EngineError::vk("mapping uniform buffer", result))?
    } as *mut T;
    Ok((buffer, memory, mapped))
}

fn allocate_descriptor_set(
    device: &ash::Device,
    descriptor_pool: vk::DescriptorPool,
    layout: vk::DescriptorSetLayout,
) -> Result<vk::DescriptorSet> {
    let allocate_info = vk::DescriptorSetAllocateInfo::default()
        .descriptor_pool(descriptor_pool)
        .set_layouts(std::slice::from_ref(&layout));
    unsafe {
        device
            .allocate_descriptor_sets(&allocate_info)
            .map_err(|result| EngineError::vk("allocating descriptor set", result))
            .map(|sets| sets[0])
    }
}

fn create_descriptor_set_layout(
    device: &ash::Device,
    bindings: &[vk::DescriptorSetLayoutBinding<'_>],
) -> Result<vk::DescriptorSetLayout> {
    let layout_info = vk::DescriptorSetLayoutCreateInfo::default().bindings(bindings);
    unsafe {
        device
            .create_descriptor_set_layout(&layout_info, None)
            .map_err(|result| EngineError::vk("creating descriptor set layout", result))
    }
}

fn is_srgb_format(format: vk::Format) -> bool {
    matches!(
        format,
        vk::Format::R8G8B8A8_SRGB | vk::Format::B8G8R8A8_SRGB | vk::Format::A8B8G8R8_SRGB_PACK32
    )
}
