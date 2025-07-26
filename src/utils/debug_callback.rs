use ash::vk;
use std::ffi;
use std::os;

use crate::print_separator;

pub unsafe extern "system" fn vulkan_debug_callback(
    message_severity: vk::DebugUtilsMessageSeverityFlagsEXT,
    message_type: vk::DebugUtilsMessageTypeFlagsEXT,
    p_callback_data: *const vk::DebugUtilsMessengerCallbackDataEXT<'_>,
    _user_data: *mut os::raw::c_void,
) -> vk::Bool32 {
    print_separator!("Debug Callback Message", 80, '?');

    let callback_data = unsafe { *p_callback_data };
    println!(
        "{:?} {:?} -> {:?}",
        message_type,
        message_severity,
        unsafe { ffi::CStr::from_ptr(callback_data.p_message) },
    );

    vk::FALSE
}
