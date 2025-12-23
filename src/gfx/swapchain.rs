use crate::core::swapchain::Swapchain;
use crate::gfx::context::VkContext;
use crate::utils::config::Config;
use anyhow::Result;
use ash::vk;
pub struct SwapchainManager {
    pub swapchain: Swapchain,
}

impl SwapchainManager {
    pub fn new(context: &VkContext, cfg: &Config, width: u32, height: u32) -> Result<Self> {
        let swapchain = Swapchain::new(
            &context.instance,
            &context.device,
            &context.surface,
            width,
            height,
        )?;

        Ok(Self { swapchain })
    }
}

impl SwapchainManager {
    pub fn recreate(&mut self, context: &VkContext, width: u32, height: u32) -> Result<bool> {
        if width == 0 || height == 0 {
            return Ok(false);
        }

        unsafe {
            context.device.device.device_wait_idle()?;
        }

        // ✅ FIX HERE
        self.swapchain.destroy(&context.device.device);
        self.swapchain = Swapchain::new(
            &context.instance,
            &context.device,
            &context.surface,
            width,
            height,
        )?;

        Ok(true)
    }
}

impl SwapchainManager {
    pub fn extent(&self) -> vk::Extent2D {
        self.swapchain.extent
    }
}

impl SwapchainManager {
    pub fn destroy(&mut self, device: &ash::Device) {
        self.swapchain.destroy(device);
    }
}
