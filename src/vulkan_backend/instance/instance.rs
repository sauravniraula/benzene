use ash::ext::debug_utils;
use ash::{Entry, Instance, vk};
use std::ffi::CString;
use winit::raw_window_handle::HasDisplayHandle;
use winit::window::Window;

pub struct VInstance {
    pub entry: Entry,
    pub instance: Instance,
    pub debug_instance: Option<debug_utils::Instance>,
    pub debug_messenger: Option<vk::DebugUtilsMessengerEXT>,
}

impl VInstance {
    pub fn new(window: &Window, config: super::VInstanceConfig) -> Self {
        let entry = Entry::linked();

        let app_name = CString::new(config.application_name).unwrap();
        let app_info = vk::ApplicationInfo::default()
            .api_version(vk::make_api_version(0, 1, 0, 0))
            .application_name(&app_name)
            .application_version(0)
            .engine_name(&app_name)
            .engine_version(0);

        let mut extensions =
            ash_window::enumerate_required_extensions(window.display_handle().unwrap().as_raw())
                .expect("failed to fetch required window extensions")
                .to_vec();

        extensions.extend(config.extensions.iter().map(|e| e.as_ptr() as *const i8));
        let debug_utils_extension = c"VK_EXT_debug_utils";
        if config.enable_debug {
            extensions.push(debug_utils_extension.as_ptr() as *const i8);
        }

        let mut layers = config.layers;
        if config.enable_debug {
            layers.push("VK_LAYER_KHRONOS_validation".into());
        }
        let p_layers: Vec<*const i8> = layers
            .iter()
            .map(|each| CString::new(each.as_str()).unwrap().into_raw() as *const i8)
            .collect();

        let create_info = vk::InstanceCreateInfo::default()
            .application_info(&app_info)
            .enabled_extension_names(&extensions)
            .enabled_layer_names(&p_layers);

        let instance = unsafe {
            entry
                .create_instance(&create_info, None)
                .expect("failed to create instance")
        };

        if config.enable_debug {
            let (debug_instance, debug_messenger) =
                VInstance::setup_debug_messenger(&entry, &instance);
            return Self {
                instance,
                entry,
                debug_instance: Some(debug_instance),
                debug_messenger: Some(debug_messenger),
            };
        } else {
            return Self {
                instance,
                entry,
                debug_instance: None,
                debug_messenger: None,
            };
        }
    }

    pub fn setup_debug_messenger(
        entry: &Entry,
        instance: &Instance,
    ) -> (debug_utils::Instance, vk::DebugUtilsMessengerEXT) {
        let debug_info = vk::DebugUtilsMessengerCreateInfoEXT::default()
            .message_severity(
                vk::DebugUtilsMessageSeverityFlagsEXT::ERROR
                    | vk::DebugUtilsMessageSeverityFlagsEXT::WARNING,
            )
            .message_type(
                vk::DebugUtilsMessageTypeFlagsEXT::GENERAL
                    | vk::DebugUtilsMessageTypeFlagsEXT::VALIDATION
                    | vk::DebugUtilsMessageTypeFlagsEXT::PERFORMANCE,
            )
            .pfn_user_callback(Some(super::debug::vulkan_debug_callback));
        let debug_instance = debug_utils::Instance::new(&entry, &instance);
        let debug_messenger = unsafe {
            debug_instance
                .create_debug_utils_messenger(&debug_info, None)
                .expect("failed creating debug messenger")
        };
        (debug_instance, debug_messenger)
    }

    pub fn destroy(&self) {
        unsafe {
            if let (Some(debug_instance), Some(debug_messenger)) =
                (&self.debug_instance, self.debug_messenger)
            {
                debug_instance.destroy_debug_utils_messenger(debug_messenger, None);
            }
            self.instance.destroy_instance(None);
        }
    }
}
