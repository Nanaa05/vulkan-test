use anyhow::{Context, Result};
use ash::{Instance, vk};

use super::{device::Device, surface::Surface};

pub struct Swapchain {
    pub loader: ash::khr::swapchain::Device,
    pub swapchain: vk::SwapchainKHR,
    pub format: vk::Format,
    pub extent: vk::Extent2D,
    pub images: Vec<vk::Image>,
    pub image_views: Vec<vk::ImageView>,
}

impl Swapchain {
    pub fn new(
        instance: &Instance,
        dev: &Device,
        surface: &Surface,
        fb_width: u32,
        fb_height: u32,
    ) -> Result<Self> {
        // ✅ Correct: pass ash::Instance wrapper
        let loader = ash::khr::swapchain::Device::new(instance, &dev.device);

        let caps = unsafe {
            surface
                .loader
                .get_physical_device_surface_capabilities(dev.physical, surface.surface)?
        };
        let formats = unsafe {
            surface
                .loader
                .get_physical_device_surface_formats(dev.physical, surface.surface)?
        };
        let present_modes = unsafe {
            surface
                .loader
                .get_physical_device_surface_present_modes(dev.physical, surface.surface)?
        };

        let surface_format = formats
            .iter()
            .cloned()
            .find(|f| {
                f.format == vk::Format::B8G8R8A8_SRGB
                    && f.color_space == vk::ColorSpaceKHR::SRGB_NONLINEAR
            })
            .unwrap_or_else(|| formats[0]);

        let present_mode = present_modes
            .iter()
            .cloned()
            .find(|m| *m == vk::PresentModeKHR::MAILBOX)
            .unwrap_or(vk::PresentModeKHR::FIFO);

        let extent = if caps.current_extent.width != u32::MAX {
            caps.current_extent
        } else {
            vk::Extent2D {
                width: fb_width.clamp(caps.min_image_extent.width, caps.max_image_extent.width),
                height: fb_height.clamp(caps.min_image_extent.height, caps.max_image_extent.height),
            }
        };

        let mut image_count = caps.min_image_count + 1;
        if caps.max_image_count > 0 {
            image_count = image_count.min(caps.max_image_count);
        }

        let (sharing_mode, queue_family_indices) = if dev.queues.same_family() {
            (vk::SharingMode::EXCLUSIVE, vec![])
        } else {
            (
                vk::SharingMode::CONCURRENT,
                vec![dev.queues.graphics_family, dev.queues.present_family],
            )
        };

        let create_info = vk::SwapchainCreateInfoKHR::default()
            .surface(surface.surface)
            .min_image_count(image_count)
            .image_format(surface_format.format)
            .image_color_space(surface_format.color_space)
            .image_extent(extent)
            .image_array_layers(1)
            .image_usage(vk::ImageUsageFlags::COLOR_ATTACHMENT)
            .image_sharing_mode(sharing_mode)
            .queue_family_indices(&queue_family_indices)
            .pre_transform(caps.current_transform)
            .composite_alpha(vk::CompositeAlphaFlagsKHR::OPAQUE)
            .present_mode(present_mode)
            .clipped(true);

        let swapchain = unsafe { loader.create_swapchain(&create_info, None) }
            .context("Failed to create swapchain")?;

        let images = unsafe { loader.get_swapchain_images(swapchain)? };

        let image_views = images
            .iter()
            .map(|&img| {
                let view_info = vk::ImageViewCreateInfo::default()
                    .image(img)
                    .view_type(vk::ImageViewType::TYPE_2D)
                    .format(surface_format.format)
                    .subresource_range(
                        vk::ImageSubresourceRange::default()
                            .aspect_mask(vk::ImageAspectFlags::COLOR)
                            .level_count(1)
                            .layer_count(1),
                    );
                unsafe { dev.device.create_image_view(&view_info, None) }
            })
            .collect::<std::result::Result<Vec<_>, _>>()?;

        Ok(Self {
            loader,
            swapchain,
            format: surface_format.format,
            extent,
            images,
            image_views,
        })
    }

    pub fn destroy(&mut self, dev: &ash::Device) {
        unsafe {
            for &v in &self.image_views {
                dev.destroy_image_view(v, None);
            }
            self.image_views.clear();
            self.loader.destroy_swapchain(self.swapchain, None);
        }
    }
}
