use anyhow::{Context, Result};
use ash::{Entry, Instance, khr, vk}; // Added khr
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
            .api_version(vk::make_api_version(0, 1, 2, 0));

        let mut extensions = ash_window::enumerate_required_extensions(display_handle)?.to_vec();

        // 1. ADD THIS: Mandatory extension for macOS/MoltenVK
        // These are the standard names for these extensions in ash
        extensions.push(khr::portability_enumeration::NAME.as_ptr());
        extensions.push(khr::get_physical_device_properties2::NAME.as_ptr());

        #[cfg(debug_assertions)]
        {
            extensions.push(ash::ext::debug_utils::NAME.as_ptr());
        }

        let mut enabled_layers = Vec::new();
        #[cfg(debug_assertions)]
        {
            enabled_layers.push(CString::new("VK_LAYER_KHRONOS_validation")?);
        }
        let layer_ptrs: Vec<*const i8> = enabled_layers
            .iter()
            .map(|l: &std::ffi::CString| l.as_ptr())
            .collect();

        // 3. CHANGE THIS: Add the Enumerate Portability flag
        let create_info = vk::InstanceCreateInfo::default()
            .application_info(&app_info)
            .enabled_extension_names(&extensions)
            .enabled_layer_names(&layer_ptrs)
            .flags(vk::InstanceCreateFlags::ENUMERATE_PORTABILITY_KHR); // <--- CRITICAL

        let instance = unsafe { entry.create_instance(&create_info, None) }
            .context("Failed to create VkInstance")?;

        Ok(Self {
            instance,
            enabled_layers,
        })
    }
}
