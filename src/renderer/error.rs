use thiserror::Error;

#[derive(Debug, Error)]
pub enum RenderError {
    #[error("swapchain out of date")]
    SwapchainOutOfDate,

    #[error(transparent)]
    Vulkan(#[from] ash::vk::Result),

    #[error(transparent)]
    Other(#[from] anyhow::Error),
}

