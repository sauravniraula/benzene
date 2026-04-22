use std::{cell::RefCell, ffi::CStr};

use ash::{self};
use ash_window;
use winit;

use crate::{backend::image::create_image_views, log_info};

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

pub struct VcontextState {
    pub swapchain: ash::vk::SwapchainKHR,
    pub swapchain_images: Vec<ash::vk::Image>,
    pub swapchain_image_views: Vec<ash::vk::ImageView>,
    pub surface_capabilities: ash::vk::SurfaceCapabilitiesKHR,
    pub image_count: u32,
}

pub struct Vcontext {
    pub entry: ash::Entry,
    pub instance: ash::Instance,
    pub debug_utils_loader: ash::ext::debug_utils::Instance,
    pub debug_messenger: ash::vk::DebugUtilsMessengerEXT,
    pub surface_instance: ash::khr::surface::Instance,
    pub surface: ash::vk::SurfaceKHR,
    pub physical_device: ash::vk::PhysicalDevice,
    pub memory_properties: ash::vk::PhysicalDeviceMemoryProperties,
    pub device: ash::Device,
    pub graphics_queue_index: u32,
    pub compute_queue_index: u32,
    pub transfer_queue_index: u32,
    pub graphics_queue: ash::vk::Queue,
    pub compute_queue: ash::vk::Queue,
    pub transfer_queue: ash::vk::Queue,
    pub surface_format: ash::vk::SurfaceFormatKHR,
    pub present_mode: ash::vk::PresentModeKHR,
    pub swapchain_device: ash::khr::swapchain::Device,
    pub state: RefCell<VcontextState>,
}

impl Drop for Vcontext {
    fn drop(&mut self) {
        unsafe {
            let _ = self.device.device_wait_idle();
            let state = self.state.borrow();

            for &each in &state.swapchain_image_views {
                self.device.destroy_image_view(each, None);
            }
            self.swapchain_device
                .destroy_swapchain(state.swapchain, None);

            self.device.destroy_device(None);
            self.surface_instance.destroy_surface(self.surface, None);
            self.debug_utils_loader
                .destroy_debug_utils_messenger(self.debug_messenger, None);
            self.instance.destroy_instance(None);
        }
    }
}

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
        let debug_messenger = unsafe {
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

        let memory_properties =
            unsafe { instance.get_physical_device_memory_properties(physical_device) };

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

        let surface_format = choose_surface_format(physical_device, &surface_instance, surface);
        let present_mode = choose_present_mode(physical_device, &surface_instance, surface);

        let swapchain_device = ash::khr::swapchain::Device::new(&instance, &device);

        let (swapchain, swapchain_images, swapchain_image_views, surface_capabilities, image_count) =
            create_swapchain(
                physical_device,
                &device,
                &swapchain_device,
                &surface_instance,
                surface,
                surface_format,
                present_mode,
            );

        let state = RefCell::new(VcontextState {
            swapchain: swapchain,
            swapchain_images: swapchain_images,
            swapchain_image_views: swapchain_image_views,
            surface_capabilities: surface_capabilities,
            image_count: image_count,
        });

        Self {
            entry,
            instance,
            debug_utils_loader,
            debug_messenger,
            surface_instance,
            surface,
            physical_device,
            memory_properties,
            device,
            graphics_queue_index,
            compute_queue_index,
            transfer_queue_index,
            graphics_queue,
            compute_queue,
            transfer_queue,
            surface_format,
            present_mode,
            swapchain_device,
            state,
        }
    }

    pub fn recreate_swapchain(&self) {
        let mut state = self.state.borrow_mut();

        unsafe {
            state.swapchain_image_views.iter().for_each(|&e| {
                self.device.destroy_image_view(e, None);
            });
            self.swapchain_device
                .destroy_swapchain(state.swapchain, None);
        }

        let (swapchain, swapchain_images, swapchain_image_views, surface_capabilities, image_count) =
            create_swapchain(
                self.physical_device,
                &self.device,
                &self.swapchain_device,
                &self.surface_instance,
                self.surface,
                self.surface_format,
                self.present_mode,
            );
        state.swapchain = swapchain;
        state.swapchain_images = swapchain_images;
        state.swapchain_image_views = swapchain_image_views;
        state.surface_capabilities = surface_capabilities;
        state.image_count = image_count;
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

        if vk13.dynamic_rendering == ash::vk::FALSE
            || eds.extended_dynamic_state == ash::vk::FALSE
            || vk13.synchronization2 == ash::vk::FALSE
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

    let mut vk13 = ash::vk::PhysicalDeviceVulkan13Features::default()
        .dynamic_rendering(true)
        .synchronization2(true);
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
    let compute_queue = unsafe { device.get_device_queue(compute_queue_index, 0) };
    let transfer_queue = unsafe { device.get_device_queue(transfer_queue_index, 0) };

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

fn choose_surface_format(
    physical_device: ash::vk::PhysicalDevice,
    surface_instance: &ash::khr::surface::Instance,
    surface: ash::vk::SurfaceKHR,
) -> ash::vk::SurfaceFormatKHR {
    let surface_formats = unsafe {
        surface_instance
            .get_physical_device_surface_formats(physical_device, surface)
            .expect("unable to get supported surface formats")
    };
    let surface_format = surface_formats
        .iter()
        .find(|&&e| {
            e.format == ash::vk::Format::B8G8R8A8_SRGB
                && e.color_space == ash::vk::ColorSpaceKHR::SRGB_NONLINEAR
        })
        .expect("unable to select surface format");
    log_info!("Selected surface format: {:?}", surface_format);
    *surface_format
}

fn choose_present_mode(
    physical_device: ash::vk::PhysicalDevice,
    surface_instance: &ash::khr::surface::Instance,
    surface: ash::vk::SurfaceKHR,
) -> ash::vk::PresentModeKHR {
    let present_modes = unsafe {
        surface_instance
            .get_physical_device_surface_present_modes(physical_device, surface)
            .expect("unable to get supported present modes")
    };
    let present_mode = present_modes
        .iter()
        .find(|&&e| e == ash::vk::PresentModeKHR::FIFO)
        .expect("unable to select present mode");
    log_info!("Selected present mode: {:?}", present_mode);
    *present_mode
}

fn create_swapchain(
    physical_device: ash::vk::PhysicalDevice,
    device: &ash::Device,
    swapchain_device: &ash::khr::swapchain::Device,
    surface_instance: &ash::khr::surface::Instance,
    surface: ash::vk::SurfaceKHR,
    surface_format: ash::vk::SurfaceFormatKHR,
    present_mode: ash::vk::PresentModeKHR,
) -> (
    ash::vk::SwapchainKHR,
    Vec<ash::vk::Image>,
    Vec<ash::vk::ImageView>,
    ash::vk::SurfaceCapabilitiesKHR,
    u32,
) {
    let surface_capabilities = unsafe {
        surface_instance
            .get_physical_device_surface_capabilities(physical_device, surface)
            .expect("unable to get surface capabilities")
    };
    let image_count = surface_capabilities.min_image_count + 1;

    let create_info = ash::vk::SwapchainCreateInfoKHR::default()
        .surface(surface)
        .min_image_count(image_count)
        .image_format(surface_format.format)
        .image_color_space(surface_format.color_space)
        .image_extent(surface_capabilities.current_extent)
        .image_array_layers(1)
        .image_usage(ash::vk::ImageUsageFlags::COLOR_ATTACHMENT)
        .image_sharing_mode(ash::vk::SharingMode::EXCLUSIVE)
        .pre_transform(surface_capabilities.current_transform)
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

    let swapchain_image_views = create_image_views(
        device,
        &swapchain_images,
        surface_format.format,
        ash::vk::ImageSubresourceRange::default()
            .layer_count(1)
            .level_count(1)
            .aspect_mask(ash::vk::ImageAspectFlags::COLOR),
    );

    (
        swapchain,
        swapchain_images,
        swapchain_image_views,
        surface_capabilities,
        image_count,
    )
}
