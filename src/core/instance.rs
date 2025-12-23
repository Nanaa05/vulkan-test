use anyhow::{Context, Result};
use ash::{Entry, Instance, vk};
use std::ffi::CString;

pub struct VkInstance {
    pub instance: Instance,
    pub enabled_layers: Vec<CString>,
}

impl VkInstance {
    pub fn new(entry: &Entry, display_handle: raw_window_handle::RawDisplayHandle) -> Result<Self> {
        let app_name = CString::new("vulkan-rust-playground")?;
        let engine_name = CString::new("no-engine")?;

        let app_info = vk::ApplicationInfo::default()
            .application_name(&app_name)
            .engine_name(&engine_name)
            .api_version(vk::make_api_version(0, 1, 3, 0));

        // Extensions needed for surface creation depend on platform. :contentReference[oaicite:3]{index=3}
        let mut extensions = ash_window::enumerate_required_extensions(display_handle)?.to_vec();

        // Enable debug utils in debug builds
        #[cfg(debug_assertions)]
        {
            extensions.push(ash::ext::debug_utils::NAME.as_ptr());
        }

        // Validation layer (debug only)
        let mut enabled_layers = Vec::new();
        #[cfg(debug_assertions)]
        {
            enabled_layers.push(CString::new("VK_LAYER_KHRONOS_validation")?);
        }
        let layer_ptrs: Vec<*const i8> = enabled_layers
            .iter()
            .map(|l: &std::ffi::CString| l.as_ptr())
            .collect();

        let create_info = vk::InstanceCreateInfo::default()
            .application_info(&app_info)
            .enabled_extension_names(&extensions)
            .enabled_layer_names(&layer_ptrs);

        let instance = unsafe { entry.create_instance(&create_info, None) }
            .context("Failed to create VkInstance")?;

        Ok(Self {
            instance,
            enabled_layers,
        })
    }
}
