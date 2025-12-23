use anyhow::Result;
use ash::{Entry, Instance, vk};

pub struct Surface {
    pub loader: ash::khr::surface::Instance,
    pub surface: vk::SurfaceKHR,
}

impl Surface {
    pub fn new(
        entry: &Entry,
        instance: &Instance,
        display_handle: raw_window_handle::RawDisplayHandle,
        window_handle: raw_window_handle::RawWindowHandle,
    ) -> Result<Self> {
        let loader = ash::khr::surface::Instance::new(entry, instance);

        // ash-window create_surface signature uses RawDisplayHandle + RawWindowHandle. :contentReference[oaicite:4]{index=4}
        let surface = unsafe {
            ash_window::create_surface(entry, instance, display_handle, window_handle, None)?
        };

        Ok(Self { loader, surface })
    }
}

impl Drop for Surface {
    fn drop(&mut self) {
        unsafe {
            self.loader.destroy_surface(self.surface, None);
        }
    }
}
