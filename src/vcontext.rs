use std::ffi::CStr;

use ash;
use ash_window;
use winit;

use crate::{
    constants::{SHADER_OUTPUT_DIR, SHADER_SOURCE_DIR},
    log_info,
};

unsafe extern "system" fn vulkan_debug_callback(
    message_severity: ash::vk::DebugUtilsMessageSeverityFlagsEXT,
    message_type: ash::vk::DebugUtilsMessageTypeFlagsEXT,
    p_callback_data: *const ash::vk::DebugUtilsMessengerCallbackDataEXT<'_>,
    _user_data: *mut std::os::raw::c_void,
) -> ash::vk::Bool32 {
    log_info!(
        "Vulkan Debug Callback: severity={:?}, type={:?}, message={}",
        message_severity,
        message_type,
        unsafe { CStr::from_ptr((*p_callback_data).p_message) }.to_string_lossy()
    );

    ash::vk::FALSE
}

pub struct Vcontext {}

impl Vcontext {
    pub fn new(
        mut extensions: Vec<&CStr>,
        mut layers: Vec<&CStr>,
        display_handle: winit::raw_window_handle::DisplayHandle,
        window_handle: winit::raw_window_handle::WindowHandle,
    ) -> Self {
        let entry = unsafe { ash::Entry::load().expect("unable to load vulkan") };
        let app_info =
            ash::vk::ApplicationInfo::default().api_version(ash::vk::make_api_version(0, 1, 3, 0));

        extensions.push(ash::ext::debug_utils::NAME);
        layers.push(c"VK_LAYER_KHRONOS_validation");

        let enabled_extensions: Vec<*const i8> =
            extensions.iter().map(|each| each.as_ptr()).collect();
        let enabled_layers: Vec<*const i8> = layers.iter().map(|each| each.as_ptr()).collect();
        let instance_create_info = ash::vk::InstanceCreateInfo::default()
            .application_info(&app_info)
            .enabled_extension_names(&enabled_extensions)
            .enabled_layer_names(&enabled_layers);
        let instance = unsafe {
            entry
                .create_instance(&instance_create_info, None)
                .expect("unable to create instance")
        };

        let debug_info = ash::vk::DebugUtilsMessengerCreateInfoEXT::default()
            .message_severity(
                ash::vk::DebugUtilsMessageSeverityFlagsEXT::WARNING
                    | ash::vk::DebugUtilsMessageSeverityFlagsEXT::INFO
                    | ash::vk::DebugUtilsMessageSeverityFlagsEXT::ERROR
                    | ash::vk::DebugUtilsMessageSeverityFlagsEXT::VERBOSE,
            )
            .message_type(
                ash::vk::DebugUtilsMessageTypeFlagsEXT::GENERAL
                    | ash::vk::DebugUtilsMessageTypeFlagsEXT::VALIDATION
                    | ash::vk::DebugUtilsMessageTypeFlagsEXT::PERFORMANCE,
            )
            .pfn_user_callback(Some(vulkan_debug_callback));

        let debug_utils_loader = ash::ext::debug_utils::Instance::new(&entry, &instance);
        unsafe {
            debug_utils_loader
                .create_debug_utils_messenger(&debug_info, None)
                .unwrap()
        };

        let surface = unsafe {
            ash_window::create_surface(
                &entry,
                &instance,
                display_handle.as_raw(),
                window_handle.as_raw(),
                None,
            )
            .expect("failed to create surface")
        };
        let surface_instance = ash::khr::surface::Instance::new(&entry, &instance);

        let device_extensions = vec![ash::vk::KHR_SWAPCHAIN_NAME];
        let physical_device = pick_physical_device(&instance, &device_extensions);

        let (
            device,
            graphics_queue_index,
            compute_queue_index,
            transfer_queue_index,
            graphics_queue,
            compute_queue,
            transfer_queue,
        ) = create_logical_device(
            &instance,
            physical_device,
            &surface_instance,
            surface,
            &device_extensions,
        );

        let (surface_format, present_mode, surface_extent, current_transform, image_count) =
            choose_surface_details(physical_device, &surface_instance, surface);

        let swapchain_device = ash::khr::swapchain::Device::new(&instance, &device);
        let (swapchain, swapchain_images) = create_swapchain(
            &swapchain_device,
            surface,
            surface_format,
            present_mode,
            surface_extent,
            current_transform,
            image_count,
        );
        let swapchain_image_views = create_image_views(
            &device,
            &swapchain_images,
            surface_format.format,
            ash::vk::ImageSubresourceRange::default()
                .layer_count(1)
                .level_count(1)
                .aspect_mask(ash::vk::ImageAspectFlags::COLOR),
        );

        let pipeline = create_graphics_pipeline(&device, surface_format);

        Self {}
    }
}

fn pick_physical_device(
    instance: &ash::Instance,
    extensions: &Vec<&CStr>,
) -> ash::vk::PhysicalDevice {
    let mut pdws: Vec<_> = unsafe {
        instance
            .enumerate_physical_devices()
            .expect("unable to enumerate physical devices")
    }
    .iter()
    .map(|&each| {
        let properties = unsafe { instance.get_physical_device_properties(each) };
        let features = unsafe { instance.get_physical_device_features(each) };
        let queue_families = unsafe { instance.get_physical_device_queue_family_properties(each) };
        let supported_extensions = unsafe {
            instance
                .enumerate_device_extension_properties(each)
                .expect("unable to enumerate device extensions")
        };

        let mut score = 0_i32;

        if properties.device_type == ash::vk::PhysicalDeviceType::DISCRETE_GPU {
            score += 1000;
        }
        score += properties.limits.max_image_dimension2_d as i32;

        if features.geometry_shader == ash::vk::FALSE {
            score = -1;
        }

        if properties.api_version < ash::vk::make_api_version(0, 1, 3, 0) {
            score = -1;
        }

        if queue_families
            .iter()
            .all(|qf| qf.queue_flags.contains(ash::vk::QueueFlags::GRAPHICS) == false)
        {
            score = -1;
        }

        if !extensions.iter().all(|required| {
            supported_extensions
                .iter()
                .any(|supported| supported.extension_name_as_c_str().unwrap() == required)
        }) {
            score = -1;
        }

        let mut vk13 = ash::vk::PhysicalDeviceVulkan13Features::default();
        let mut eds = ash::vk::PhysicalDeviceExtendedDynamicStateFeaturesEXT::default();
        let mut features2 = ash::vk::PhysicalDeviceFeatures2::default()
            .push_next(&mut vk13)
            .push_next(&mut eds);
        unsafe {
            instance.get_physical_device_features2(each, &mut features2);
        }

        if vk13.dynamic_rendering == ash::vk::FALSE || eds.extended_dynamic_state == ash::vk::FALSE
        {
            score = -1;
        }

        (each, properties, score)
    })
    .collect();

    pdws.sort_by_key(|(_, __, score)| std::cmp::Reverse(*score));
    let selected = pdws.first().expect("no suitable physical device found");
    if selected.2 < 0 {
        panic!("no suitable physical device found");
    }
    log_info!(
        "Selected physical device: {:?}",
        selected.1.device_name_as_c_str().unwrap()
    );
    selected.0
}

fn create_logical_device(
    instance: &ash::Instance,
    physical_device: ash::vk::PhysicalDevice,
    surface_instance: &ash::khr::surface::Instance,
    surface: ash::vk::SurfaceKHR,
    extensions: &Vec<&CStr>,
) -> (
    ash::Device,
    u32,
    u32,
    u32,
    ash::vk::Queue,
    ash::vk::Queue,
    ash::vk::Queue,
) {
    let queue_families =
        unsafe { instance.get_physical_device_queue_family_properties(physical_device) };

    let mut graphics = None;
    let mut transfer = None;
    let mut compute = None;

    for (i, qf) in queue_families.iter().enumerate() {
        let idx = i as u32;

        if graphics.is_none() && qf.queue_flags.contains(ash::vk::QueueFlags::GRAPHICS) {
            let supports_present = unsafe {
                surface_instance
                    .get_physical_device_surface_support(physical_device, idx, surface)
                    .unwrap()
            };
            if supports_present {
                graphics = Some(idx);
            }
        }

        if compute.is_none()
            && qf.queue_flags.contains(ash::vk::QueueFlags::COMPUTE)
            && Some(idx) != graphics
        {
            compute = Some(idx);
        }

        if transfer.is_none()
            && qf.queue_flags.contains(ash::vk::QueueFlags::TRANSFER)
            && Some(idx) != graphics
            && Some(idx) != compute
        {
            transfer = Some(idx);
        }
    }

    let graphics_queue_index = graphics.expect("No graphics queue found");
    let transfer_queue_index = transfer.unwrap_or(graphics_queue_index);
    let compute_queue_index = compute.unwrap_or(graphics_queue_index);

    let compute_same_as_graphics = graphics_queue_index == compute_queue_index;
    let transfer_same_as_graphics = graphics_queue_index == transfer_queue_index;

    let mut queue_create_infos = vec![
        ash::vk::DeviceQueueCreateInfo::default()
            .queue_family_index(graphics_queue_index)
            .queue_priorities(&[0.5_f32]),
    ];
    if !compute_same_as_graphics {
        queue_create_infos.push(
            ash::vk::DeviceQueueCreateInfo::default()
                .queue_family_index(compute_queue_index)
                .queue_priorities(&[0.5_f32]),
        );
    }
    if !transfer_same_as_graphics {
        queue_create_infos.push(
            ash::vk::DeviceQueueCreateInfo::default()
                .queue_family_index(transfer_queue_index)
                .queue_priorities(&[0.5_f32]),
        );
    }

    let mut vk13 = ash::vk::PhysicalDeviceVulkan13Features::default().dynamic_rendering(true);
    let mut eds = ash::vk::PhysicalDeviceExtendedDynamicStateFeaturesEXT::default()
        .extended_dynamic_state(true);
    let mut features2 = ash::vk::PhysicalDeviceFeatures2::default()
        .push_next(&mut vk13)
        .push_next(&mut eds);

    let enabled_extensions: Vec<*const i8> = extensions.iter().map(|each| each.as_ptr()).collect();
    let create_info = ash::vk::DeviceCreateInfo::default()
        .push_next(&mut features2)
        .queue_create_infos(&queue_create_infos)
        .enabled_extension_names(&enabled_extensions);

    let device = unsafe {
        instance
            .create_device(physical_device, &create_info, None)
            .expect("unable to create device")
    };

    let graphics_queue = unsafe { device.get_device_queue(graphics_queue_index, 0) };
    let compute_queue = unsafe { device.get_device_queue(graphics_queue_index, 0) };
    let transfer_queue = unsafe { device.get_device_queue(graphics_queue_index, 0) };

    log_info!("Graphics queue index: {}", graphics_queue_index);
    log_info!("Compute queue index: {}", compute_queue_index);
    log_info!("Transfer queue index: {}", transfer_queue_index);

    (
        device,
        graphics_queue_index,
        compute_queue_index,
        transfer_queue_index,
        graphics_queue,
        compute_queue,
        transfer_queue,
    )
}

fn choose_surface_details(
    physical_device: ash::vk::PhysicalDevice,
    surface_instance: &ash::khr::surface::Instance,
    surface: ash::vk::SurfaceKHR,
) -> (
    ash::vk::SurfaceFormatKHR,
    ash::vk::PresentModeKHR,
    ash::vk::Extent2D,
    ash::vk::SurfaceTransformFlagsKHR,
    u32,
) {
    let surface_formats = unsafe {
        surface_instance
            .get_physical_device_surface_formats(physical_device, surface)
            .expect("unable to get supported surface formats")
    };
    let surface_capabilities = unsafe {
        surface_instance
            .get_physical_device_surface_capabilities(physical_device, surface)
            .expect("unable to get surface capabilities")
    };
    let present_modes = unsafe {
        surface_instance
            .get_physical_device_surface_present_modes(physical_device, surface)
            .expect("unable to get supported present modes")
    };

    let surface_format = surface_formats
        .iter()
        .find(|&&e| {
            e.format == ash::vk::Format::B8G8R8A8_SRGB
                && e.color_space == ash::vk::ColorSpaceKHR::SRGB_NONLINEAR
        })
        .expect("unable to select surface format");

    let present_mode = present_modes
        .iter()
        .find(|&&e| e == ash::vk::PresentModeKHR::FIFO)
        .expect("unable to select present mode");

    let surface_extent = surface_capabilities.current_extent;
    let current_transform = surface_capabilities.current_transform;
    let image_count = surface_capabilities.min_image_count + 1;

    log_info!("Selected surface format: {:?}", surface_format);
    log_info!("Selected present mode: {:?}", present_mode);
    log_info!("Selected surface resolution: {:?}", surface_extent);
    log_info!("Surface transform: {:?}", current_transform);
    log_info!("Selected image count: {:?}", image_count);

    (
        *surface_format,
        *present_mode,
        surface_extent,
        current_transform,
        image_count,
    )
}

fn create_swapchain(
    swapchain_device: &ash::khr::swapchain::Device,
    surface: ash::vk::SurfaceKHR,
    surface_format: ash::vk::SurfaceFormatKHR,
    present_mode: ash::vk::PresentModeKHR,
    surface_extent: ash::vk::Extent2D,
    current_transform: ash::vk::SurfaceTransformFlagsKHR,
    image_count: u32,
) -> (ash::vk::SwapchainKHR, Vec<ash::vk::Image>) {
    let create_info = ash::vk::SwapchainCreateInfoKHR::default()
        .surface(surface)
        .min_image_count(image_count)
        .image_format(surface_format.format)
        .image_color_space(surface_format.color_space)
        .image_extent(surface_extent)
        .image_array_layers(1)
        .image_usage(ash::vk::ImageUsageFlags::COLOR_ATTACHMENT)
        .image_sharing_mode(ash::vk::SharingMode::EXCLUSIVE)
        .pre_transform(current_transform)
        .composite_alpha(ash::vk::CompositeAlphaFlagsKHR::OPAQUE)
        .present_mode(present_mode)
        .clipped(true);

    let swapchain = unsafe {
        swapchain_device
            .create_swapchain(&create_info, None)
            .expect("unable to create swapchain")
    };
    let swapchain_images = unsafe {
        swapchain_device
            .get_swapchain_images(swapchain)
            .expect("unable to get swapchain images")
    };
    (swapchain, swapchain_images)
}

fn create_image_views(
    device: &ash::Device,
    images: &Vec<ash::vk::Image>,
    format: ash::vk::Format,
    subresource_range: ash::vk::ImageSubresourceRange,
) -> Vec<ash::vk::ImageView> {
    let image_views: Vec<_> = images
        .iter()
        .map(|&e| {
            let create_info = ash::vk::ImageViewCreateInfo::default()
                .image(e)
                .view_type(ash::vk::ImageViewType::TYPE_2D)
                .format(format)
                .subresource_range(subresource_range);

            unsafe {
                device
                    .create_image_view(&create_info, None)
                    .expect("unable to create image view")
            }
        })
        .collect();
    image_views
}

fn create_graphics_pipeline(
    device: &ash::Device,
    surface_format: ash::vk::SurfaceFormatKHR,
) -> ash::vk::Pipeline {
    let vs_path = compiled_spirv_path_for_source("assets/shaders/test.vert");
    let fs_path = compiled_spirv_path_for_source("assets/shaders/test.frag");

    log_info!("{}", vs_path);
    log_info!("{}", fs_path);

    let vs_code = load_file_as_vec_u32(&vs_path);
    let fs_code = load_file_as_vec_u32(&fs_path);

    let vs_module = unsafe {
        device
            .create_shader_module(
                &ash::vk::ShaderModuleCreateInfo::default().code(&vs_code),
                None,
            )
            .expect("unable to create shader module")
    };
    let fs_module = unsafe {
        device
            .create_shader_module(
                &ash::vk::ShaderModuleCreateInfo::default().code(&fs_code),
                None,
            )
            .expect("unable to create shader module")
    };

    let vs_stage = ash::vk::PipelineShaderStageCreateInfo::default()
        .stage(ash::vk::ShaderStageFlags::VERTEX)
        .module(vs_module)
        .name(c"main");

    let fs_stage = ash::vk::PipelineShaderStageCreateInfo::default()
        .stage(ash::vk::ShaderStageFlags::VERTEX)
        .module(fs_module)
        .name(c"main");

    let shader_stages = [vs_stage, fs_stage];

    let dynamic_state = ash::vk::PipelineDynamicStateCreateInfo::default().dynamic_states(&[
        ash::vk::DynamicState::VIEWPORT,
        ash::vk::DynamicState::SCISSOR,
    ]);

    let vertex_input_state = ash::vk::PipelineVertexInputStateCreateInfo::default();

    let input_assembly_state = ash::vk::PipelineInputAssemblyStateCreateInfo::default()
        .topology(ash::vk::PrimitiveTopology::TRIANGLE_LIST);

    // let viewport = ash::vk::Viewport::default()
    //     .x(0_f32)
    //     .y(0_f32)
    //     .width(surface_extent.width as f32)
    //     .height(surface_extent.height as f32)
    //     .min_depth(0_f32)
    //     .max_depth(1_f32);

    // let scissor = ash::vk::Rect2D::default()
    //     .offset(ash::vk::Offset2D::default().x(0).y(0))
    //     .extent(*surface_extent);

    let viewport_state = ash::vk::PipelineViewportStateCreateInfo::default()
        .viewport_count(1)
        .scissor_count(1);

    let rasterization_state = ash::vk::PipelineRasterizationStateCreateInfo::default()
        .depth_clamp_enable(false)
        .rasterizer_discard_enable(false)
        .polygon_mode(ash::vk::PolygonMode::FILL)
        .cull_mode(ash::vk::CullModeFlags::BACK)
        .front_face(ash::vk::FrontFace::CLOCKWISE)
        .depth_bias_enable(false)
        .line_width(1_f32);

    let multisampling_state = ash::vk::PipelineMultisampleStateCreateInfo::default()
        .rasterization_samples(ash::vk::SampleCountFlags::TYPE_1)
        .sample_shading_enable(false);

    let color_blend_attachments = [ash::vk::PipelineColorBlendAttachmentState::default()
        .blend_enable(false)
        .color_write_mask(ash::vk::ColorComponentFlags::RGBA)];

    let color_blend_state = ash::vk::PipelineColorBlendStateCreateInfo::default()
        .logic_op_enable(false)
        .logic_op(ash::vk::LogicOp::COPY)
        .attachments(&color_blend_attachments);

    let pipeline_layout = unsafe {
        device
            .create_pipeline_layout(&ash::vk::PipelineLayoutCreateInfo::default(), None)
            .expect("unable to create pipeline layout")
    };

    let color_attachment_formats = [surface_format.format];
    let mut rendering_info = ash::vk::PipelineRenderingCreateInfo::default()
        .color_attachment_formats(&color_attachment_formats);

    let create_info = ash::vk::GraphicsPipelineCreateInfo::default()
        .push_next(&mut rendering_info)
        .stages(&shader_stages)
        .vertex_input_state(&vertex_input_state)
        .input_assembly_state(&input_assembly_state)
        .rasterization_state(&rasterization_state)
        .multisample_state(&multisampling_state)
        .color_blend_state(&color_blend_state)
        .dynamic_state(&dynamic_state)
        .viewport_state(&viewport_state)
        .layout(pipeline_layout);

    let pipelines = unsafe {
        device
            .create_graphics_pipelines(ash::vk::PipelineCache::null(), &[create_info], None)
            .expect("unable to create pipelines")
    };
    pipelines[0]
}

pub fn load_file_as_vec_u32(file_path: &str) -> Vec<u32> {
    let u8_bytes: Vec<u8> = std::fs::read(file_path).expect("failed to load file");
    let u32_bytes: Vec<u32> = u8_bytes
        .chunks_exact(4)
        .map(|chunk| u32::from_le_bytes(chunk.try_into().expect("failed to convert u8 to u32")))
        .collect();
    u32_bytes
}

pub fn compiled_spirv_path_for_source(source_path: &str) -> String {
    let prefix = format!("{}/", SHADER_SOURCE_DIR);
    let rel = source_path
        .strip_prefix(prefix.as_str())
        .unwrap_or(source_path);
    format!("{}/{}.spv", SHADER_OUTPUT_DIR, rel)
}
