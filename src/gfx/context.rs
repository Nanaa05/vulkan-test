// src/gfx/context.rs
use anyhow::Result;
use raw_window_handle::{RawDisplayHandle, RawWindowHandle};

use crate::core::{
    debug::DebugMessenger, device::Device, entry::create_entry, instance::VkInstance,
    surface::Surface,
};

pub struct VkContext {
    pub entry: ash::Entry,
    pub instance: ash::Instance,
    pub debug: Option<DebugMessenger>,
    pub surface: Surface,
    pub device: Device,
}

impl VkContext {
    pub fn new(display: RawDisplayHandle, window: RawWindowHandle) -> Result<Self> {
        // Entry
        let entry = create_entry()?;

        // Instance
        let instance_wrapper = VkInstance::new(&entry, display)?;
        let instance = instance_wrapper.instance.clone();

        // Debug
        let debug = DebugMessenger::new(&entry, &instance)?;

        // Surface
        let surface = Surface::new(&entry, &instance, display, window)?;

        // Device
        let device = Device::new(&instance, &surface)?;

        Ok(Self {
            entry,
            instance,
            debug,
            surface,
            device,
        })
    }
}
