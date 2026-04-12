use ash::vk;
use thiserror::Error;

pub type Result<T> = std::result::Result<T, EngineError>;

#[derive(Debug, Error)]
pub enum EngineError {
    #[error("vulkan failure while {operation}: {result:?}")]
    Vulkan {
        operation: &'static str,
        result: vk::Result,
    },
    #[error("io failure while {operation}: {source}")]
    Io {
        operation: &'static str,
        #[source]
        source: std::io::Error,
    },
    #[error("image failure while loading `{path}`: {source}")]
    Image {
        path: String,
        #[source]
        source: image::ImageError,
    },
    #[error("obj import failure while loading `{path}`: {source}")]
    Obj {
        path: String,
        #[source]
        source: tobj::LoadError,
    },
    #[error("egui renderer failure: {0}")]
    EguiRenderer(#[from] egui_ash_renderer::RendererError),
    #[error("no compatible vulkan physical device found")]
    NoCompatibleDevice,
    #[error("window extent is zero")]
    ZeroSizedWindow,
    #[error("swapchain is out of date")]
    SwapchainOutOfDate,
    #[error("{0}")]
    Message(String),
}

impl EngineError {
    pub fn vk(operation: &'static str, result: vk::Result) -> Self {
        Self::Vulkan { operation, result }
    }
}
