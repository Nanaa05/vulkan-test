use anyhow::{Context, Result};
use ash::{Instance, vk};

use super::{queues::QueueFamilyIndices, surface::Surface};

pub struct Device {
    pub instance: ash::Instance,
    pub physical: vk::PhysicalDevice,
    pub device: ash::Device,
    pub queues: QueueFamilyIndices,
    pub graphics_queue: vk::Queue,
    pub present_queue: vk::Queue,
    pub memory_properties: vk::PhysicalDeviceMemoryProperties,
    pub upload_pool: vk::CommandPool,
    pub upload_fence: vk::Fence,
}

impl Device {
    pub fn new(instance: &Instance, surface: &Surface) -> Result<Self> {
        let physical_devices = unsafe { instance.enumerate_physical_devices() }
            .context("No physical devices found")?;

        let mut picked: Option<(vk::PhysicalDevice, QueueFamilyIndices)> = None;

        for pd in physical_devices {
            if let Some(q) = Self::find_queue_families(instance, surface, pd)? {
                if Self::supports_swapchain(instance, pd)? {
                    picked = Some((pd, q));
                    break;
                }
            }
        }

        let (physical, queues) =
            picked.context("Failed to find suitable GPU (graphics+present+swapchain)")?;

        let priorities = [1.0_f32];

        let mut unique_families = vec![queues.graphics_family];
        if !queues.same_family() {
            unique_families.push(queues.present_family);
        }

        let queue_infos: Vec<vk::DeviceQueueCreateInfo> = unique_families
            .iter()
            .map(|&fam| {
                vk::DeviceQueueCreateInfo::default()
                    .queue_family_index(fam)
                    .queue_priorities(&priorities)
            })
            .collect();

        // ---- device extensions ----
        let available_exts = unsafe { instance.enumerate_device_extension_properties(physical)? };
        let has_portability_subset = available_exts.iter().any(|e| unsafe {
            std::ffi::CStr::from_ptr(e.extension_name.as_ptr()).to_bytes()
                == b"VK_KHR_portability_subset"
        });

        let mut device_exts: Vec<*const i8> = vec![ash::khr::swapchain::NAME.as_ptr()];

        // Keep CString alive until create_device()
        let portability_subset_name = std::ffi::CString::new("VK_KHR_portability_subset")?;

        #[cfg(target_os = "macos")]
        {
            if has_portability_subset {
                device_exts.push(portability_subset_name.as_ptr());
            }
        }

        let create_info = vk::DeviceCreateInfo::default()
            .queue_create_infos(&queue_infos)
            .enabled_extension_names(&device_exts);

        let device = unsafe { instance.create_device(physical, &create_info, None) }
            .context("Failed to create logical device")?;

        let graphics_queue = unsafe { device.get_device_queue(queues.graphics_family, 0) };
        let present_queue = unsafe { device.get_device_queue(queues.present_family, 0) };

        let memory_properties = unsafe { instance.get_physical_device_memory_properties(physical) };

        let upload_pool_info = vk::CommandPoolCreateInfo::default()
            .queue_family_index(queues.graphics_family)
            .flags(vk::CommandPoolCreateFlags::TRANSIENT);

        let upload_pool = unsafe { device.create_command_pool(&upload_pool_info, None)? };

        let fence_info = vk::FenceCreateInfo::default();
        let upload_fence = unsafe { device.create_fence(&fence_info, None)? };

        Ok(Self {
            instance: instance.clone(), // ✅ FIXED
            physical,
            device,
            queues,
            graphics_queue,
            present_queue,
            memory_properties,
            upload_pool,
            upload_fence,
        })
    }

    // ---------- public capability API ----------

    pub fn pick_depth_format(&self) -> Result<vk::Format> {
        self.find_supported_format(
            &[
                vk::Format::D32_SFLOAT,
                vk::Format::D32_SFLOAT_S8_UINT,
                vk::Format::D24_UNORM_S8_UINT,
            ],
            vk::ImageTiling::OPTIMAL,
            vk::FormatFeatureFlags::DEPTH_STENCIL_ATTACHMENT,
        )
    }

    // ---------- private helpers ----------

    fn find_supported_format(
        &self,
        candidates: &[vk::Format],
        tiling: vk::ImageTiling,
        features: vk::FormatFeatureFlags,
    ) -> Result<vk::Format> {
        for &format in candidates {
            let props = unsafe {
                self.instance
                    .get_physical_device_format_properties(self.physical, format)
            };

            let supported = match tiling {
                vk::ImageTiling::LINEAR => props.linear_tiling_features.contains(features),
                vk::ImageTiling::OPTIMAL => props.optimal_tiling_features.contains(features),
                _ => false,
            };

            if supported {
                return Ok(format);
            }
        }

        anyhow::bail!("No supported format found")
    }

    fn supports_swapchain(instance: &Instance, pd: vk::PhysicalDevice) -> Result<bool> {
        let exts = unsafe { instance.enumerate_device_extension_properties(pd) }?;
        Ok(exts.iter().any(|e| unsafe {
            std::ffi::CStr::from_ptr(e.extension_name.as_ptr()) == ash::khr::swapchain::NAME
        }))
    }

    fn find_queue_families(
        instance: &Instance,
        surface: &Surface,
        pd: vk::PhysicalDevice,
    ) -> Result<Option<QueueFamilyIndices>> {
        let props = unsafe { instance.get_physical_device_queue_family_properties(pd) };

        let mut graphics = None;
        let mut present = None;

        for (i, p) in props.iter().enumerate() {
            let idx = i as u32;

            if p.queue_flags.contains(vk::QueueFlags::GRAPHICS) && graphics.is_none() {
                graphics = Some(idx);
            }

            let present_support = unsafe {
                surface
                    .loader
                    .get_physical_device_surface_support(pd, idx, surface.surface)?
            };
            if present_support && present.is_none() {
                present = Some(idx);
            }
        }

        match (graphics, present) {
            (Some(g), Some(p)) => Ok(Some(QueueFamilyIndices {
                graphics_family: g,
                present_family: p,
            })),
            _ => Ok(None),
        }
    }
}

impl Drop for Device {
    fn drop(&mut self) {
        unsafe {
            self.device.destroy_fence(self.upload_fence, None);
            self.device.destroy_command_pool(self.upload_pool, None);
            self.device.destroy_device(None);
        }
    }
}

impl Device {
    pub fn begin_one_time_commands(&self) -> anyhow::Result<vk::CommandBuffer> {
        let alloc = vk::CommandBufferAllocateInfo::default()
            .command_pool(self.upload_pool)
            .level(vk::CommandBufferLevel::PRIMARY)
            .command_buffer_count(1);

        let cmd = unsafe { self.device.allocate_command_buffers(&alloc)?[0] };

        let begin = vk::CommandBufferBeginInfo::default()
            .flags(vk::CommandBufferUsageFlags::ONE_TIME_SUBMIT);

        unsafe {
            self.device.begin_command_buffer(cmd, &begin)?;
        }

        Ok(cmd)
    }

    pub fn end_one_time_commands(&self, cmd: vk::CommandBuffer) -> anyhow::Result<()> {
        unsafe {
            self.device.end_command_buffer(cmd)?;

            // reset fence, submit, wait
            self.device.reset_fences(&[self.upload_fence])?;

            let submit = vk::SubmitInfo::default().command_buffers(std::slice::from_ref(&cmd));
            self.device
                .queue_submit(self.graphics_queue, &[submit], self.upload_fence)?;

            self.device
                .wait_for_fences(&[self.upload_fence], true, u64::MAX)?;

            // free the command buffer back to upload_pool
            self.device.free_command_buffers(self.upload_pool, &[cmd]);
        }
        Ok(())
    }
}
