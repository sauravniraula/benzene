use crate::core::{device::VDevice, swapchain::VSwapchain};

pub enum VBackendEvent<'a> {
    UpdateFramebuffers(&'a VDevice, &'a VSwapchain),
    None,
}
