use ash::ext::debug_utils;
use ash::khr::{surface, swapchain};
use ash::vk::{Offset2D, Rect2D};
use ash::{Device, Entry, Instance, vk};
use std::cell::Cell;
use std::ffi::CString;
use std::u64;

use crate::entities::{CommandBufferState, ComputeDevice, VertexData, Window};
use crate::utils::command_buffer::get_command_buffer_states;
use crate::utils::{load_file_as_vec_u32, vulkan_debug_callback};

pub struct VulkanApp {
    instance: Instance,
    debug_utils_loader: debug_utils::Instance,
    debug_call_back: vk::DebugUtilsMessengerEXT,
    surface: vk::SurfaceKHR,
    surface_loader: surface::Instance,
    device: Device,
    graphics_queue: vk::Queue,
    present_queue: vk::Queue,
    swapchain_loader: swapchain::Device,
    swapchain: vk::SwapchainKHR,
    image_extent: vk::Extent2D,
    images: Vec<vk::Image>,
    image_views: Vec<vk::ImageView>,
    vert_shader_module: vk::ShaderModule,
    frag_shader_module: vk::ShaderModule,
    pipeline_layout: vk::PipelineLayout,
    render_pass: vk::RenderPass,
    pipelines: Vec<vk::Pipeline>,
    framebuffers: Vec<vk::Framebuffer>,
    command_pool: vk::CommandPool,
    max_inflight_images: u32,
    command_buffers: Vec<CommandBufferState>,
    vertices_size: u32,
    vertex_buffers: Vec<vk::Buffer>,
    vertex_buffers_memory: Vec<vk::DeviceMemory>,
    current_frame: Cell<usize>,
}

impl VulkanApp {
    pub fn new(window: &Window, max_inflight_images: u32, enable_validation_layers: bool) -> Self {
        let app_name = c"Hello Vulkan";

        let entry = Entry::linked();

        // Creating Application Info for Create Info
        let app_info = vk::ApplicationInfo::default()
            .application_name(app_name)
            .application_version(0)
            .engine_name(app_name)
            .engine_version(0)
            .api_version(vk::make_api_version(0, 1, 0, 0));

        // Listing GLFW Extensions required for Create Info
        let mut required_glfw_extentions = window
            .get_required_glfw_extensions()
            .expect("failed on window.get_required_glfw_extensions");
        if enable_validation_layers {
            required_glfw_extentions.push("VK_EXT_debug_utils".into());
        }
        let required_glfw_extensions: Vec<*const i8> = required_glfw_extentions
            .iter()
            .map(|s| {
                CString::new(s.as_str())
                    .expect("failed to convert &str to CString")
                    .into_raw() as *const i8
            })
            .collect();

        // Listing Available Layers
        // let available_layers = unsafe {
        //     entry
        //         .enumerate_instance_layer_properties()
        //         .expect("enumerate instance extension properties")
        // };

        // Vulkan Layers for Create Info
        let mut validation_layers = Vec::<*const i8>::new();
        if enable_validation_layers {
            validation_layers.push(c"VK_LAYER_KHRONOS_validation".as_ptr());
        }

        // Listing Supported Extensions
        // let available_extensions = unsafe {
        //     entry
        //         .enumerate_instance_extension_properties(None)
        //         .expect("failed to enumerate instance extension properties")
        // };

        // Creating Create Info for VKInstance
        let create_info = vk::InstanceCreateInfo::default()
            .application_info(&app_info)
            .enabled_extension_names(&required_glfw_extensions)
            .enabled_layer_names(&validation_layers)
            .flags(vk::InstanceCreateFlags::default());

        // Creating VKInstance
        let instance = unsafe {
            entry
                .create_instance(&create_info, None)
                .expect("failed to create instance")
        };

        // Custom Debug Message Callback
        let debug_info = vk::DebugUtilsMessengerCreateInfoEXT::default()
            .message_severity(
                vk::DebugUtilsMessageSeverityFlagsEXT::ERROR
                    | vk::DebugUtilsMessageSeverityFlagsEXT::WARNING, // | vk::DebugUtilsMessageSeverityFlagsEXT::INFO,
            )
            .message_type(
                vk::DebugUtilsMessageTypeFlagsEXT::GENERAL
                    | vk::DebugUtilsMessageTypeFlagsEXT::VALIDATION
                    | vk::DebugUtilsMessageTypeFlagsEXT::PERFORMANCE,
            )
            .pfn_user_callback(Some(vulkan_debug_callback));
        let debug_utils_loader = debug_utils::Instance::new(&entry, &instance);
        let debug_call_back = unsafe {
            debug_utils_loader
                .create_debug_utils_messenger(&debug_info, None)
                .expect("failed creating debug utils messenger")
        };

        // Creating Surface
        let surface = unsafe {
            window
                .get_surface(instance.handle(), None)
                .expect("failed to create surface")
        };
        let surface_loader = surface::Instance::new(&entry, &instance);

        // Listing Physical Devices
        let pdevices = unsafe {
            instance
                .enumerate_physical_devices()
                .expect("failed to enumerate physical devices")
        };
        let mut available_devices: Vec<ComputeDevice> = vec![];
        for each_device in pdevices {
            let device_properties = unsafe { instance.get_physical_device_properties(each_device) };
            let device_queue_properties =
                unsafe { instance.get_physical_device_queue_family_properties(each_device) };
            let mut supported_device_extension_properties = unsafe {
                instance
                    .enumerate_device_extension_properties(each_device)
                    .expect("failed to enumerate device extension properties")
            };
            let supported_device_extensions: Vec<_> = supported_device_extension_properties
                .iter_mut()
                .map(|e| {
                    e.extension_name_as_c_str()
                        .expect("failed to get device extensions as c_str")
                        .to_owned()
                })
                .collect();
            let device_surface_capabilities = unsafe {
                surface_loader
                    .get_physical_device_surface_capabilities(each_device, surface)
                    .expect("failed to get device surface capabilities")
            };
            let device_surface_formats = unsafe {
                surface_loader
                    .get_physical_device_surface_formats(each_device, surface)
                    .expect("failed to get device surface formats")
            };
            let device_surface_present_modes = unsafe {
                surface_loader
                    .get_physical_device_surface_present_modes(each_device, surface)
                    .expect("failed to get device surface present modes")
            };
            let device_queue_flags: Vec<vk::QueueFlags> = device_queue_properties
                .iter()
                .map(|e| e.queue_flags)
                .collect();
            let device_memory_heaps =
                unsafe { instance.get_physical_device_memory_properties(each_device) }.memory_heaps;
            let mut device_memory = 0;
            for each_heap in device_memory_heaps {
                if each_heap.flags != vk::MemoryHeapFlags::DEVICE_LOCAL {
                    continue;
                }
                if device_memory < each_heap.size {
                    device_memory = each_heap.size;
                }
            }
            device_memory = device_memory / 1024 / 1024;
            let compute_device = ComputeDevice::new(
                each_device,
                device_properties
                    .device_name_as_c_str()
                    .expect("failed to fetch device name")
                    .into(),
                device_properties.device_type,
                device_memory as u32,
                device_queue_flags,
                supported_device_extensions,
                device_surface_capabilities,
                device_surface_formats,
                device_surface_present_modes,
            );
            println!("{}", compute_device);
            available_devices.push(compute_device);
        }

        // Select Device
        let (selected_device, graphics_queue_family_index) =
            ComputeDevice::select_device_and_queue(available_devices)
                .expect("failed to select device");
        let present_queue_family_index = selected_device
            .select_present_queue(&surface_loader, surface)
            .expect("failed to select present queue");
        print!("{}", selected_device);

        // Creating Logical Device
        let device_extensions = [c"VK_KHR_swapchain".as_ptr()];
        let device_enabled_features = vk::PhysicalDeviceFeatures::default();
        let queue_priorities = [1.0];
        let mut unique_queue_family_indices: Vec<u32> = vec![graphics_queue_family_index];
        if !unique_queue_family_indices.contains(&present_queue_family_index) {
            unique_queue_family_indices.push(present_queue_family_index);
        }
        let queue_create_infos: Vec<_> = unique_queue_family_indices
            .iter()
            .map(|each| {
                return vk::DeviceQueueCreateInfo::default()
                    .queue_family_index(*each)
                    .queue_priorities(&queue_priorities);
            })
            .collect();
        let device_create_info = vk::DeviceCreateInfo::default()
            .queue_create_infos(&queue_create_infos)
            .enabled_features(&device_enabled_features)
            .enabled_extension_names(&device_extensions);
        let device = unsafe {
            instance
                .create_device(selected_device.pdevice, &device_create_info, None)
                .expect("failed to create logical device")
        };

        // Creating Graphics Queue
        let graphics_queue = unsafe { device.get_device_queue(graphics_queue_family_index, 0) };
        let present_queue = unsafe { device.get_device_queue(present_queue_family_index, 0) };

        // Selecting Surface Format, Presentation Mode and Image Extent for Swapchain
        let selected_surface_format = selected_device.select_surface_format();
        let selected_present_mode = selected_device.select_present_mode();
        let selected_image_extent = selected_device.select_image_extent(&window);

        // Selecting Swapchain image count
        let selected_swapchain_image_count = selected_device.select_swapchain_image_count();

        // Selecting Pre Transform
        let current_transform = selected_device.get_current_transform();

        // Creating Swapchain
        let swapchain_loader = swapchain::Device::new(&instance, &device);
        let mut swapchain_create_info = vk::SwapchainCreateInfoKHR::default()
            .surface(surface)
            .min_image_count(selected_swapchain_image_count)
            .image_format(selected_surface_format.format)
            .image_color_space(selected_surface_format.color_space)
            .image_extent(selected_image_extent)
            .image_array_layers(1)
            .image_usage(vk::ImageUsageFlags::COLOR_ATTACHMENT | vk::ImageUsageFlags::TRANSFER_DST)
            .pre_transform(current_transform)
            .composite_alpha(vk::CompositeAlphaFlagsKHR::OPAQUE)
            .present_mode(selected_present_mode)
            .clipped(true);

        if graphics_queue_family_index != present_queue_family_index {
            swapchain_create_info = swapchain_create_info
                .image_sharing_mode(vk::SharingMode::CONCURRENT)
                .queue_family_indices(&unique_queue_family_indices);
        } else {
            swapchain_create_info =
                swapchain_create_info.image_sharing_mode(vk::SharingMode::EXCLUSIVE);
        }
        let swapchain = unsafe {
            swapchain_loader
                .create_swapchain(&swapchain_create_info, None)
                .expect("failed to create swapchain")
        };

        // Getting Swapchain Image Handles
        let swapchain_images = unsafe {
            swapchain_loader
                .get_swapchain_images(swapchain)
                .expect("failed to get swapchain images")
        };

        // Creating Swapchain Image Views
        let mut image_views: Vec<vk::ImageView> = vec![];
        for &each_image in swapchain_images.iter() {
            let image_view_create_info = vk::ImageViewCreateInfo::default()
                .image(each_image)
                .view_type(vk::ImageViewType::TYPE_2D)
                .format(selected_surface_format.format)
                .components(vk::ComponentMapping {
                    r: vk::ComponentSwizzle::IDENTITY,
                    g: vk::ComponentSwizzle::IDENTITY,
                    b: vk::ComponentSwizzle::IDENTITY,
                    a: vk::ComponentSwizzle::IDENTITY,
                })
                .subresource_range(vk::ImageSubresourceRange {
                    aspect_mask: vk::ImageAspectFlags::COLOR,
                    base_mip_level: 0,
                    level_count: 1,
                    base_array_layer: 0,
                    layer_count: 1,
                });
            let image_view = unsafe {
                device
                    .create_image_view(&image_view_create_info, None)
                    .expect("failed to create image view")
            };
            image_views.push(image_view);
        }

        // Input Data
        let vertices: Vec<VertexData> = vec![
            VertexData {
                position: [-0.5, -0.5],
                color: [1.0, 1.0, 0.0],
            },
            VertexData {
                position: [0.5, 0.5],
                color: [0.0, 1.0, 0.0],
            },
            VertexData {
                position: [-0.5, 0.5],
                color: [0.0, 0.0, 1.0],
            },
        ];
        let binding_descriptions = [VertexData::get_binding_description()];
        let attribute_descriptions = VertexData::get_attribute_descriptions();

        let vertex_buffer_info = vk::BufferCreateInfo::default()
            .size((std::mem::size_of::<VertexData>() * vertices.len()) as u64)
            .usage(vk::BufferUsageFlags::VERTEX_BUFFER)
            .sharing_mode(vk::SharingMode::EXCLUSIVE);
        let vertex_buffer = unsafe {
            device
                .create_buffer(&vertex_buffer_info, None)
                .expect("failed to create vertex buffer")
        };
        let vertex_buffer_mem_req = unsafe { device.get_buffer_memory_requirements(vertex_buffer) };
        let memory_type_bits = vertex_buffer_mem_req.memory_type_bits;
        let required_properties =
            vk::MemoryPropertyFlags::HOST_VISIBLE | vk::MemoryPropertyFlags::HOST_COHERENT;
        let mem_properties =
            unsafe { instance.get_physical_device_memory_properties(selected_device.pdevice) };

        let mut compatible_index: i32 = -1;
        for i in 0..mem_properties.memory_type_count {
            let compatible = (memory_type_bits & (1 << i)) != 0;
            let has_required_properties = mem_properties.memory_types[i as usize]
                .property_flags
                .contains(required_properties);
            if compatible && has_required_properties {
                compatible_index = i as i32;
                break;
            }
        }
        if compatible_index == -1 {
            panic!("Could not find compatible memory type")
        }

        let vb_memory_alloc_info = vk::MemoryAllocateInfo::default()
            .allocation_size(vertex_buffer_mem_req.size)
            .memory_type_index(compatible_index as u32);
        let vertex_buffer_memory = unsafe {
            device
                .allocate_memory(&vb_memory_alloc_info, None)
                .expect("failed to allocate memory")
        };
        unsafe {
            device
                .bind_buffer_memory(vertex_buffer, vertex_buffer_memory, 0)
                .expect("failed to bind buffer memory")
        };

        let gpu_memory_pointer = unsafe {
            device
                .map_memory(
                    vertex_buffer_memory,
                    0,
                    vertex_buffer_info.size,
                    vk::MemoryMapFlags::empty(),
                )
                .expect("failed to map memory")
        };

        unsafe {
            std::ptr::copy_nonoverlapping(
                vertices.as_ptr() as *const u8,
                gpu_memory_pointer as *mut u8,
                vertex_buffer_info.size as usize,
            );
        }

        unsafe {
            device.unmap_memory(vertex_buffer_memory);
        }

        // Configuring Pipeline
        // Vertex Input Stage
        let vertex_input_info = vk::PipelineVertexInputStateCreateInfo::default()
            .vertex_binding_descriptions(&binding_descriptions)
            .vertex_attribute_descriptions(&attribute_descriptions);

        // Input Assembly Stage
        let input_assembly_info = vk::PipelineInputAssemblyStateCreateInfo::default()
            .topology(vk::PrimitiveTopology::TRIANGLE_LIST)
            .primitive_restart_enable(false);

        // Vertex Shader Stage
        let vert_shader_code = load_file_as_vec_u32("src/shaders/shader.vert.spv");
        let vert_shader_module = unsafe {
            device
                .create_shader_module(
                    &vk::ShaderModuleCreateInfo::default().code(&vert_shader_code),
                    None,
                )
                .expect("failed to create shader module")
        };
        let vert_shader_stage_create_info = vk::PipelineShaderStageCreateInfo::default()
            .stage(vk::ShaderStageFlags::VERTEX)
            .module(vert_shader_module)
            .name(c"main");

        // Tessellation Stage

        // Geometry Shader Stage

        // Rasterization Stage
        let rasterizer = vk::PipelineRasterizationStateCreateInfo::default()
            .depth_clamp_enable(false)
            .rasterizer_discard_enable(false)
            .polygon_mode(vk::PolygonMode::FILL)
            .line_width(1.0)
            .cull_mode(vk::CullModeFlags::BACK)
            .front_face(vk::FrontFace::CLOCKWISE)
            .depth_bias_enable(false);

        // Fragment Shader Stage
        let frag_shader_code = load_file_as_vec_u32("src/shaders/shader.frag.spv");
        let frag_shader_module = unsafe {
            device
                .create_shader_module(
                    &vk::ShaderModuleCreateInfo::default().code(&frag_shader_code),
                    None,
                )
                .expect("failed to create shader module")
        };
        let frag_shader_stage_create_info = vk::PipelineShaderStageCreateInfo::default()
            .stage(vk::ShaderStageFlags::FRAGMENT)
            .module(frag_shader_module)
            .name(c"main");

        // Color Blending Stage
        let color_blend_attachments = [vk::PipelineColorBlendAttachmentState::default()
            .color_write_mask(
                vk::ColorComponentFlags::R
                    | vk::ColorComponentFlags::G
                    | vk::ColorComponentFlags::B
                    | vk::ColorComponentFlags::A,
            )
            .blend_enable(false)];
        let color_blending = vk::PipelineColorBlendStateCreateInfo::default()
            .logic_op_enable(false)
            .attachments(&color_blend_attachments);

        // Dynamic State
        let dynamic_state = vk::PipelineDynamicStateCreateInfo::default()
            .dynamic_states(&[vk::DynamicState::VIEWPORT, vk::DynamicState::SCISSOR]);

        //? Can define static viewport and scissor here
        let viewport_state = vk::PipelineViewportStateCreateInfo::default()
            .viewport_count(1)
            .scissor_count(1);

        // Multisampling
        let multisampling = vk::PipelineMultisampleStateCreateInfo::default()
            .sample_shading_enable(false)
            .rasterization_samples(vk::SampleCountFlags::TYPE_1);

        // Pipeline Layout
        let pipeline_layout_info = vk::PipelineLayoutCreateInfo::default();
        let pipeline_layout = unsafe {
            device
                .create_pipeline_layout(&pipeline_layout_info, None)
                .expect("failed to create pipeline layout")
        };

        // Creating render pass
        let render_pass_attachments = [vk::AttachmentDescription::default()
            .format(selected_surface_format.format)
            .samples(vk::SampleCountFlags::TYPE_1)
            .load_op(vk::AttachmentLoadOp::CLEAR)
            .store_op(vk::AttachmentStoreOp::STORE)
            .stencil_load_op(vk::AttachmentLoadOp::DONT_CARE)
            .stencil_store_op(vk::AttachmentStoreOp::DONT_CARE)
            .initial_layout(vk::ImageLayout::UNDEFINED)
            .final_layout(vk::ImageLayout::PRESENT_SRC_KHR)];

        let subpass_attachment_refs = [vk::AttachmentReference::default()
            .attachment(0)
            .layout(vk::ImageLayout::COLOR_ATTACHMENT_OPTIMAL)];

        let subpasses = [vk::SubpassDescription::default()
            .pipeline_bind_point(vk::PipelineBindPoint::GRAPHICS)
            .color_attachments(&subpass_attachment_refs)];

        let subpass_dependencies = [vk::SubpassDependency::default()
            .src_subpass(vk::SUBPASS_EXTERNAL)
            .dst_subpass(0)
            .src_stage_mask(vk::PipelineStageFlags::COLOR_ATTACHMENT_OUTPUT)
            .src_access_mask(vk::AccessFlags::empty())
            .dst_stage_mask(vk::PipelineStageFlags::COLOR_ATTACHMENT_OUTPUT)
            .dst_access_mask(vk::AccessFlags::COLOR_ATTACHMENT_WRITE)];

        let render_pass_info = vk::RenderPassCreateInfo::default()
            .attachments(&render_pass_attachments)
            .subpasses(&subpasses)
            .dependencies(&subpass_dependencies);

        let render_pass = unsafe {
            device
                .create_render_pass(&render_pass_info, None)
                .expect("failed to create render pass")
        };

        // Creating Pipeline
        let shader_stages = [vert_shader_stage_create_info, frag_shader_stage_create_info];
        let pipeline_infos = [vk::GraphicsPipelineCreateInfo::default()
            .vertex_input_state(&vertex_input_info)
            .input_assembly_state(&input_assembly_info)
            .stages(&shader_stages)
            .rasterization_state(&rasterizer)
            .color_blend_state(&color_blending)
            .dynamic_state(&dynamic_state)
            .viewport_state(&viewport_state)
            .multisample_state(&multisampling)
            .layout(pipeline_layout)
            .render_pass(render_pass)
            .subpass(0)];

        let pipelines = unsafe {
            device
                .create_graphics_pipelines(vk::PipelineCache::null(), &pipeline_infos, None)
                .expect("failed to create graphics pipeline")
        };

        // Creating Framebuffers
        let mut framebuffers: Vec<vk::Framebuffer> = vec![];
        for i in 0..image_views.len() {
            let attachments = [image_views[i]];
            let framebuffer_info = vk::FramebufferCreateInfo::default()
                .render_pass(render_pass)
                .attachments(&attachments)
                .width(selected_image_extent.width)
                .height(selected_image_extent.height)
                .layers(1);
            let framebuffer = unsafe {
                device
                    .create_framebuffer(&framebuffer_info, None)
                    .expect("failed to create framebuffer")
            };
            framebuffers.push(framebuffer);
        }

        // Creating Command Buffer
        let command_pool_info = vk::CommandPoolCreateInfo::default()
            .flags(vk::CommandPoolCreateFlags::RESET_COMMAND_BUFFER)
            .queue_family_index(graphics_queue_family_index);
        let command_pool = unsafe {
            device
                .create_command_pool(&command_pool_info, None)
                .expect("failed to create command pool")
        };
        let command_buffers = get_command_buffer_states(&device, command_pool, max_inflight_images);

        // Creating App
        Self {
            instance,
            debug_utils_loader,
            debug_call_back,
            surface,
            surface_loader,
            device,
            graphics_queue,
            present_queue,
            swapchain_loader,
            swapchain,
            image_extent: selected_image_extent,
            images: swapchain_images,
            image_views,
            vert_shader_module,
            frag_shader_module,
            pipeline_layout,
            render_pass,
            pipelines,
            framebuffers,
            command_pool,
            max_inflight_images,
            command_buffers,
            vertices_size: vertex_buffer_mem_req.size as u32,
            vertex_buffers: vec![vertex_buffer],
            vertex_buffers_memory: vec![vertex_buffer_memory],
            current_frame: Cell::new(0),
        }
    }

    pub fn draw_frame(&self) {
        let command_buffer_state = self.command_buffers[self.current_frame.get()];
        let command_buffers = [command_buffer_state.buffer];
        let in_flight_fen = command_buffer_state.in_flight_fen;
        let image_available_sem = command_buffer_state.image_available_sem;
        let render_finished_sem = command_buffer_state.render_finished_sem;

        unsafe {
            self.device
                .wait_for_fences(&[in_flight_fen], true, u64::MAX)
                .expect("failed to wait for fence");

            self.device
                .reset_fences(&[in_flight_fen])
                .expect("failed to reset fence");

            let (image_index, _) = self
                .swapchain_loader
                .acquire_next_image(
                    self.swapchain,
                    u64::MAX,
                    image_available_sem,
                    vk::Fence::null(),
                )
                .expect("failed to acquire next image");

            let framebuffer = self.framebuffers[image_index as usize];
            let render_area = Rect2D {
                offset: Offset2D { x: 0, y: 0 },
                extent: self.image_extent,
            };

            self.device
                .reset_command_buffer(
                    command_buffer_state.buffer,
                    vk::CommandBufferResetFlags::empty(),
                )
                .expect("failed resetting command buffer");

            self.record_command_buffer(
                command_buffer_state.buffer,
                self.render_pass,
                framebuffer,
                render_area,
                self.pipelines[0],
            );

            let submit_wait_semaphores = [image_available_sem];
            let submit_signal_semaphores = [render_finished_sem];
            let submit_info = vk::SubmitInfo::default()
                .command_buffers(&command_buffers)
                .wait_semaphores(&submit_wait_semaphores)
                .signal_semaphores(&submit_signal_semaphores);

            self.device
                .queue_submit(self.graphics_queue, &[submit_info], in_flight_fen)
                .expect("failed to submit queue");

            let present_wait_semaphores = [render_finished_sem];
            let swapchains = [self.swapchain];
            let image_indices = [image_index];
            let present_info = vk::PresentInfoKHR::default()
                .wait_semaphores(&present_wait_semaphores)
                .swapchains(&swapchains)
                .image_indices(&image_indices);

            self.swapchain_loader
                .queue_present(self.present_queue, &present_info)
                .expect("failed to present queue");

            self.current_frame
                .set((self.current_frame.get() + 1) % (self.max_inflight_images as usize));
        };
    }

    pub unsafe fn record_command_buffer(
        &self,
        command_buffer: vk::CommandBuffer,
        render_pass: vk::RenderPass,
        framebuffer: vk::Framebuffer,
        render_area: Rect2D,
        pipeline: vk::Pipeline,
    ) {
        let begin_info = vk::CommandBufferBeginInfo::default();
        unsafe {
            self.device
                .begin_command_buffer(command_buffer, &begin_info)
                .expect("failed to begin command buffer");

            let clear_color_value = vk::ClearColorValue {
                float32: [0.0, 0.0, 0.0, 1.0],
            };
            let clear_values = [vk::ClearValue {
                color: clear_color_value,
            }];

            let render_pass_info = vk::RenderPassBeginInfo::default()
                .render_pass(render_pass)
                .framebuffer(framebuffer)
                .render_area(render_area)
                .clear_values(&clear_values);

            self.device.cmd_begin_render_pass(
                command_buffer,
                &render_pass_info,
                vk::SubpassContents::INLINE,
            );
            self.device.cmd_bind_pipeline(
                command_buffer,
                vk::PipelineBindPoint::GRAPHICS,
                pipeline,
            );

            self.device
                .cmd_bind_vertex_buffers(command_buffer, 0, &self.vertex_buffers, &[0]);

            let viewports = [vk::Viewport::default()
                .x(0.0)
                .y(0.0)
                .width(self.image_extent.width as f32)
                .height(self.image_extent.height as f32)
                .min_depth(0.0)
                .max_depth(1.0)];
            let scissors = [render_area];
            self.device.cmd_set_viewport(command_buffer, 0, &viewports);
            self.device.cmd_set_scissor(command_buffer, 0, &scissors);

            self.device
                .cmd_draw(command_buffer, self.vertices_size, 1, 0, 0);

            self.device.cmd_end_render_pass(command_buffer);
            self.device
                .end_command_buffer(command_buffer)
                .expect("failed to end command buffer");
        }
    }
}

impl Drop for VulkanApp {
    fn drop(&mut self) {
        unsafe {
            for command_buffer in self.command_buffers.iter() {
                command_buffer.destory(&self.device);
            }
            self.device.destroy_command_pool(self.command_pool, None);
            for &framebuffer in self.framebuffers.iter() {
                self.device.destroy_framebuffer(framebuffer, None);
            }

            for &pipeline in self.pipelines.iter() {
                self.device.destroy_pipeline(pipeline, None);
            }

            self.device.destroy_render_pass(self.render_pass, None);
            self.device
                .destroy_pipeline_layout(self.pipeline_layout, None);
            self.device
                .destroy_shader_module(self.frag_shader_module, None);
            self.device
                .destroy_shader_module(self.vert_shader_module, None);

            for &image_view in self.image_views.iter() {
                self.device.destroy_image_view(image_view, None);
            }
            self.swapchain_loader
                .destroy_swapchain(self.swapchain, None);

            // Destroying buffers
            for &each in self.vertex_buffers.iter() {
                self.device.destroy_buffer(each, None);
            }
            for &each in self.vertex_buffers_memory.iter() {
                self.device.free_memory(each, None);
            }

            self.device.destroy_device(None);
            self.surface_loader.destroy_surface(self.surface, None);
            self.debug_utils_loader
                .destroy_debug_utils_messenger(self.debug_call_back, None);
            self.instance.destroy_instance(None)
        };
    }
}
