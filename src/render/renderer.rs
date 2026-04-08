use std::{mem::size_of, sync::Arc};

use ash::vk;
use memoffset::offset_of;
use nalgebra::{Matrix4, Vector4};
use winit::window::Window;

use crate::{
    assets::{AssetManager, MeshVertex},
    constants::MAX_FRAMES_IN_FLIGHT,
    error::{EngineError, Result},
    render::vulkan::VContext,
    scene::{Scene, Transform},
};

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

struct FrameResources {
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
    global_descriptor_set: vk::DescriptorSet,
    lights_descriptor_set: vk::DescriptorSet,
}

impl FrameResources {
    fn new(
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

    fn write_scene_uniforms(&mut self, scene: &Scene, width: u32, height: u32) {
        let mut global = GlobalUniform {
            view: Matrix4::identity(),
            projection: Matrix4::identity(),
            ambient_color: scene.ambient_color(),
        };
        if let Some((_, transform, _)) = scene.active_camera_components() {
            let (view, projection) = transform.view_projection(width, height);
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
        for (index, (entity, light)) in scene.point_lights.iter().enumerate() {
            if index >= 16 {
                break;
            }
            if let Some(transform) = scene.transforms.get(entity) {
                point.points[index] = Vector4::new(
                    transform.position.x,
                    transform.position.y,
                    transform.position.z,
                    1.0,
                );
                point.colors[index] = light.color;
            }
        }
        unsafe {
            std::ptr::copy_nonoverlapping(&point, self.point_mapped, 1);
        }

        let mut directional = DirectionalLightUniform {
            directions: [Vector4::zeros(); 16],
            colors: [Vector4::zeros(); 16],
        };
        for (index, (entity, light)) in scene.directional_lights.iter().enumerate() {
            if index >= 16 {
                break;
            }
            if let Some(transform) = scene.transforms.get(entity) {
                let direction =
                    transform.rotation_matrix() * nalgebra::Vector3::new(0.0, 0.0, -1.0);
                directional.directions[index] =
                    Vector4::new(direction.x, direction.y, direction.z, 1.0);
                directional.colors[index] = light.color;
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
        for (index, (entity, light)) in scene.spot_lights.iter().enumerate() {
            if index >= 16 {
                break;
            }
            if let Some(transform) = scene.transforms.get(entity) {
                let direction =
                    transform.rotation_matrix() * nalgebra::Vector3::new(0.0, 0.0, -1.0);
                spot.positions[index] = Vector4::new(
                    transform.position.x,
                    transform.position.y,
                    transform.position.z,
                    1.0,
                );
                spot.directions[index] = Vector4::new(direction.x, direction.y, direction.z, 0.0);
                spot.colors[index] = light.color;
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

struct GeometryPass {
    context: Arc<VContext>,
    render_pass: vk::RenderPass,
    pipeline_layout: vk::PipelineLayout,
    pipeline: vk::Pipeline,
    global_layout: vk::DescriptorSetLayout,
    lights_layout: vk::DescriptorSetLayout,
    material_layout: vk::DescriptorSetLayout,
    scene_descriptor_pool: vk::DescriptorPool,
    framebuffers: Vec<vk::Framebuffer>,
}

impl GeometryPass {
    fn new(
        context: Arc<VContext>,
        color_format: vk::Format,
        depth_format: vk::Format,
    ) -> Result<Self> {
        let global_layout = create_descriptor_set_layout(
            context.device(),
            &[vk::DescriptorSetLayoutBinding::default()
                .binding(0)
                .descriptor_count(1)
                .descriptor_type(vk::DescriptorType::UNIFORM_BUFFER)
                .stage_flags(vk::ShaderStageFlags::VERTEX | vk::ShaderStageFlags::FRAGMENT)],
        )?;
        let lights_layout = create_descriptor_set_layout(
            context.device(),
            &[
                vk::DescriptorSetLayoutBinding::default()
                    .binding(0)
                    .descriptor_count(1)
                    .descriptor_type(vk::DescriptorType::UNIFORM_BUFFER)
                    .stage_flags(vk::ShaderStageFlags::FRAGMENT),
                vk::DescriptorSetLayoutBinding::default()
                    .binding(1)
                    .descriptor_count(1)
                    .descriptor_type(vk::DescriptorType::UNIFORM_BUFFER)
                    .stage_flags(vk::ShaderStageFlags::FRAGMENT),
                vk::DescriptorSetLayoutBinding::default()
                    .binding(2)
                    .descriptor_count(1)
                    .descriptor_type(vk::DescriptorType::UNIFORM_BUFFER)
                    .stage_flags(vk::ShaderStageFlags::FRAGMENT),
            ],
        )?;
        let material_layout = create_descriptor_set_layout(
            context.device(),
            &[vk::DescriptorSetLayoutBinding::default()
                .binding(0)
                .descriptor_count(1)
                .descriptor_type(vk::DescriptorType::COMBINED_IMAGE_SAMPLER)
                .stage_flags(vk::ShaderStageFlags::FRAGMENT)],
        )?;

        let attachments = [
            vk::AttachmentDescription::default()
                .format(color_format)
                .samples(vk::SampleCountFlags::TYPE_1)
                .load_op(vk::AttachmentLoadOp::CLEAR)
                .store_op(vk::AttachmentStoreOp::STORE)
                .initial_layout(vk::ImageLayout::UNDEFINED)
                .final_layout(vk::ImageLayout::PRESENT_SRC_KHR),
            vk::AttachmentDescription::default()
                .format(depth_format)
                .samples(vk::SampleCountFlags::TYPE_1)
                .load_op(vk::AttachmentLoadOp::CLEAR)
                .store_op(vk::AttachmentStoreOp::STORE)
                .stencil_load_op(vk::AttachmentLoadOp::DONT_CARE)
                .stencil_store_op(vk::AttachmentStoreOp::DONT_CARE)
                .initial_layout(vk::ImageLayout::UNDEFINED)
                .final_layout(vk::ImageLayout::DEPTH_STENCIL_ATTACHMENT_OPTIMAL),
        ];
        let color_attachment_ref = vk::AttachmentReference::default()
            .attachment(0)
            .layout(vk::ImageLayout::COLOR_ATTACHMENT_OPTIMAL);
        let depth_attachment_ref = vk::AttachmentReference::default()
            .attachment(1)
            .layout(vk::ImageLayout::DEPTH_STENCIL_ATTACHMENT_OPTIMAL);
        let subpass = vk::SubpassDescription::default()
            .pipeline_bind_point(vk::PipelineBindPoint::GRAPHICS)
            .color_attachments(std::slice::from_ref(&color_attachment_ref))
            .depth_stencil_attachment(&depth_attachment_ref);
        let dependencies = [
            vk::SubpassDependency::default()
                .src_subpass(vk::SUBPASS_EXTERNAL)
                .dst_subpass(0)
                .src_stage_mask(vk::PipelineStageFlags::COLOR_ATTACHMENT_OUTPUT)
                .dst_stage_mask(vk::PipelineStageFlags::COLOR_ATTACHMENT_OUTPUT)
                .dst_access_mask(
                    vk::AccessFlags::COLOR_ATTACHMENT_READ
                        | vk::AccessFlags::COLOR_ATTACHMENT_WRITE,
                ),
            vk::SubpassDependency::default()
                .src_subpass(vk::SUBPASS_EXTERNAL)
                .dst_subpass(0)
                .src_stage_mask(vk::PipelineStageFlags::LATE_FRAGMENT_TESTS)
                .dst_stage_mask(vk::PipelineStageFlags::EARLY_FRAGMENT_TESTS)
                .dst_access_mask(vk::AccessFlags::DEPTH_STENCIL_ATTACHMENT_WRITE),
        ];
        let render_pass_info = vk::RenderPassCreateInfo::default()
            .attachments(&attachments)
            .subpasses(std::slice::from_ref(&subpass))
            .dependencies(&dependencies);
        let render_pass = unsafe {
            context
                .device()
                .create_render_pass(&render_pass_info, None)
                .map_err(|result| EngineError::vk("creating geometry render pass", result))?
        };

        let shader_entry = c"main";
        let vertex_module = context.load_shader_module("assets/shaders/shader.vert")?;
        let fragment_module = context.load_shader_module("assets/shaders/shader.frag")?;
        let shader_stages = [
            vk::PipelineShaderStageCreateInfo::default()
                .module(vertex_module)
                .name(shader_entry)
                .stage(vk::ShaderStageFlags::VERTEX),
            vk::PipelineShaderStageCreateInfo::default()
                .module(fragment_module)
                .name(shader_entry)
                .stage(vk::ShaderStageFlags::FRAGMENT),
        ];

        let set_layouts = [global_layout, lights_layout, material_layout];
        let push_constant = vk::PushConstantRange::default()
            .stage_flags(vk::ShaderStageFlags::VERTEX | vk::ShaderStageFlags::FRAGMENT)
            .offset(0)
            .size(size_of::<ModelPushConstants>() as u32);
        let pipeline_layout_info = vk::PipelineLayoutCreateInfo::default()
            .set_layouts(&set_layouts)
            .push_constant_ranges(std::slice::from_ref(&push_constant));
        let pipeline_layout = unsafe {
            context
                .device()
                .create_pipeline_layout(&pipeline_layout_info, None)
                .map_err(|result| EngineError::vk("creating pipeline layout", result))?
        };

        let vertex_binding = [vk::VertexInputBindingDescription::default()
            .binding(0)
            .stride(size_of::<MeshVertex>() as u32)
            .input_rate(vk::VertexInputRate::VERTEX)];
        let vertex_attributes = [
            vk::VertexInputAttributeDescription::default()
                .binding(0)
                .location(0)
                .format(vk::Format::R32G32B32_SFLOAT)
                .offset(offset_of!(MeshVertex, position) as u32),
            vk::VertexInputAttributeDescription::default()
                .binding(0)
                .location(1)
                .format(vk::Format::R32G32B32_SFLOAT)
                .offset(offset_of!(MeshVertex, color) as u32),
            vk::VertexInputAttributeDescription::default()
                .binding(0)
                .location(2)
                .format(vk::Format::R32G32B32_SFLOAT)
                .offset(offset_of!(MeshVertex, normal) as u32),
            vk::VertexInputAttributeDescription::default()
                .binding(0)
                .location(3)
                .format(vk::Format::R32G32_SFLOAT)
                .offset(offset_of!(MeshVertex, uv) as u32),
        ];
        let vertex_input = vk::PipelineVertexInputStateCreateInfo::default()
            .vertex_binding_descriptions(&vertex_binding)
            .vertex_attribute_descriptions(&vertex_attributes);
        let input_assembly = vk::PipelineInputAssemblyStateCreateInfo::default()
            .topology(vk::PrimitiveTopology::TRIANGLE_LIST);
        let viewport_state = vk::PipelineViewportStateCreateInfo::default()
            .viewport_count(1)
            .scissor_count(1);
        let rasterizer = vk::PipelineRasterizationStateCreateInfo::default()
            .line_width(1.0)
            .cull_mode(vk::CullModeFlags::BACK)
            .front_face(vk::FrontFace::COUNTER_CLOCKWISE);
        let multisampling = vk::PipelineMultisampleStateCreateInfo::default()
            .rasterization_samples(vk::SampleCountFlags::TYPE_1);
        let depth_stencil = vk::PipelineDepthStencilStateCreateInfo::default()
            .depth_test_enable(true)
            .depth_write_enable(true)
            .depth_compare_op(vk::CompareOp::LESS);
        let color_blend_attachment = [vk::PipelineColorBlendAttachmentState::default()
            .color_write_mask(vk::ColorComponentFlags::RGBA)
            .blend_enable(false)];
        let color_blend =
            vk::PipelineColorBlendStateCreateInfo::default().attachments(&color_blend_attachment);
        let dynamic_states = [vk::DynamicState::VIEWPORT, vk::DynamicState::SCISSOR];
        let dynamic_state =
            vk::PipelineDynamicStateCreateInfo::default().dynamic_states(&dynamic_states);

        let pipeline_info = vk::GraphicsPipelineCreateInfo::default()
            .stages(&shader_stages)
            .vertex_input_state(&vertex_input)
            .input_assembly_state(&input_assembly)
            .viewport_state(&viewport_state)
            .rasterization_state(&rasterizer)
            .multisample_state(&multisampling)
            .depth_stencil_state(&depth_stencil)
            .color_blend_state(&color_blend)
            .dynamic_state(&dynamic_state)
            .layout(pipeline_layout)
            .render_pass(render_pass)
            .subpass(0);
        let pipeline = unsafe {
            context
                .device()
                .create_graphics_pipelines(
                    vk::PipelineCache::null(),
                    std::slice::from_ref(&pipeline_info),
                    None,
                )
                .map_err(|(_, result)| EngineError::vk("creating graphics pipeline", result))?[0]
        };

        unsafe {
            context.device().destroy_shader_module(vertex_module, None);
            context
                .device()
                .destroy_shader_module(fragment_module, None);
        }

        let pool_sizes = [vk::DescriptorPoolSize::default()
            .ty(vk::DescriptorType::UNIFORM_BUFFER)
            .descriptor_count((MAX_FRAMES_IN_FLIGHT * 4) as u32)];
        let scene_descriptor_pool_info = vk::DescriptorPoolCreateInfo::default()
            .pool_sizes(&pool_sizes)
            .max_sets((MAX_FRAMES_IN_FLIGHT * 2) as u32);
        let scene_descriptor_pool = unsafe {
            context
                .device()
                .create_descriptor_pool(&scene_descriptor_pool_info, None)
                .map_err(|result| EngineError::vk("creating scene descriptor pool", result))?
        };

        Ok(Self {
            context,
            render_pass,
            pipeline_layout,
            pipeline,
            global_layout,
            lights_layout,
            material_layout,
            scene_descriptor_pool,
            framebuffers: Vec::new(),
        })
    }

    fn material_layout(&self) -> vk::DescriptorSetLayout {
        self.material_layout
    }

    fn rebuild_framebuffers(&mut self, context: &VContext) -> Result<()> {
        self.destroy_framebuffers();

        let (swapchain_image_views, depth_view, extent) = context.framebuffer_views();
        for image_view in &swapchain_image_views {
            let attachments = [*image_view, depth_view];
            let framebuffer_info = vk::FramebufferCreateInfo::default()
                .render_pass(self.render_pass)
                .attachments(&attachments)
                .width(extent.width)
                .height(extent.height)
                .layers(1);
            let framebuffer = unsafe {
                self.context
                    .device()
                    .create_framebuffer(&framebuffer_info, None)
                    .map_err(|result| EngineError::vk("creating framebuffer", result))?
            };
            self.framebuffers.push(framebuffer);
        }

        Ok(())
    }

    fn begin(
        &self,
        command_buffer: vk::CommandBuffer,
        framebuffer: vk::Framebuffer,
        extent: vk::Extent2D,
    ) {
        let clear_values = [
            vk::ClearValue {
                color: vk::ClearColorValue {
                    float32: [0.18, 0.22, 0.28, 1.0],
                },
            },
            vk::ClearValue {
                depth_stencil: vk::ClearDepthStencilValue {
                    depth: 1.0,
                    stencil: 0,
                },
            },
        ];
        let render_area = vk::Rect2D {
            offset: vk::Offset2D { x: 0, y: 0 },
            extent,
        };
        let render_pass_info = vk::RenderPassBeginInfo::default()
            .render_pass(self.render_pass)
            .framebuffer(framebuffer)
            .render_area(render_area)
            .clear_values(&clear_values);

        unsafe {
            self.context.device().cmd_begin_render_pass(
                command_buffer,
                &render_pass_info,
                vk::SubpassContents::INLINE,
            );
            let viewport = vk::Viewport::default()
                .x(0.0)
                .y(0.0)
                .width(extent.width as f32)
                .height(extent.height as f32)
                .min_depth(0.0)
                .max_depth(1.0);
            self.context.device().cmd_set_viewport(
                command_buffer,
                0,
                std::slice::from_ref(&viewport),
            );
            self.context.device().cmd_set_scissor(
                command_buffer,
                0,
                std::slice::from_ref(&render_area),
            );
            self.context.device().cmd_bind_pipeline(
                command_buffer,
                vk::PipelineBindPoint::GRAPHICS,
                self.pipeline,
            );
        }
    }

    fn end(&self, command_buffer: vk::CommandBuffer) {
        unsafe {
            self.context.device().cmd_end_render_pass(command_buffer);
        }
    }

    fn destroy_framebuffers(&mut self) {
        unsafe {
            for framebuffer in self.framebuffers.drain(..) {
                self.context.device().destroy_framebuffer(framebuffer, None);
            }
        }
    }
}

impl Drop for GeometryPass {
    fn drop(&mut self) {
        unsafe {
            self.destroy_framebuffers();
            self.context
                .device()
                .destroy_descriptor_pool(self.scene_descriptor_pool, None);
            self.context.device().destroy_pipeline(self.pipeline, None);
            self.context
                .device()
                .destroy_pipeline_layout(self.pipeline_layout, None);
            self.context
                .device()
                .destroy_descriptor_set_layout(self.material_layout, None);
            self.context
                .device()
                .destroy_descriptor_set_layout(self.lights_layout, None);
            self.context
                .device()
                .destroy_descriptor_set_layout(self.global_layout, None);
            self.context
                .device()
                .destroy_render_pass(self.render_pass, None);
        }
    }
}

pub struct Renderer {
    context: Arc<VContext>,
    geometry: GeometryPass,
    frames: Vec<FrameResources>,
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
            context,
            geometry,
            frames,
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
        if self.context.is_zero_sized() {
            return Ok(());
        }

        let (frame_index, image_index, command_buffer) = self.context.begin_frame()?;
        let extent = self.context.extent();
        self.frames[frame_index].write_scene_uniforms(scene, extent.width, extent.height);

        self.geometry.begin(
            command_buffer,
            self.geometry.framebuffers[image_index],
            extent,
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
