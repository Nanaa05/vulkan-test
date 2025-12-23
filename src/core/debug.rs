use anyhow::Result;
use ash::{Entry, Instance, vk};

pub struct DebugMessenger {
    pub loader: ash::ext::debug_utils::Instance,
    pub messenger: vk::DebugUtilsMessengerEXT,
}

impl DebugMessenger {
    pub fn new(entry: &Entry, instance: &Instance) -> Result<Option<Self>> {
        #[cfg(not(debug_assertions))]
        {
            let _ = entry;
            let _ = instance;
            return Ok(None);
        }

        #[cfg(debug_assertions)]
        {
            unsafe extern "system" fn callback(
                severity: vk::DebugUtilsMessageSeverityFlagsEXT,
                _types: vk::DebugUtilsMessageTypeFlagsEXT,
                data: *const vk::DebugUtilsMessengerCallbackDataEXT,
                _user: *mut std::ffi::c_void,
            ) -> vk::Bool32 {
                let message = if data.is_null() {
                    "<null>".to_string()
                } else {
                    unsafe {
                        std::ffi::CStr::from_ptr((*data).p_message)
                            .to_string_lossy()
                            .to_string()
                    }
                };

                if severity.contains(vk::DebugUtilsMessageSeverityFlagsEXT::ERROR) {
                    eprintln!("[VK][ERROR] {}", message);
                } else if severity.contains(vk::DebugUtilsMessageSeverityFlagsEXT::WARNING) {
                    eprintln!("[VK][WARN ] {}", message);
                } else {
                    eprintln!("[VK][INFO ] {}", message);
                }
                vk::FALSE
            }

            let loader = ash::ext::debug_utils::Instance::new(entry, instance);

            let create_info = vk::DebugUtilsMessengerCreateInfoEXT::default()
                .message_severity(
                    vk::DebugUtilsMessageSeverityFlagsEXT::WARNING
                        | vk::DebugUtilsMessageSeverityFlagsEXT::ERROR
                        | vk::DebugUtilsMessageSeverityFlagsEXT::INFO,
                )
                .message_type(
                    vk::DebugUtilsMessageTypeFlagsEXT::GENERAL
                        | vk::DebugUtilsMessageTypeFlagsEXT::VALIDATION
                        | vk::DebugUtilsMessageTypeFlagsEXT::PERFORMANCE,
                )
                .pfn_user_callback(Some(callback));

            let messenger = unsafe { loader.create_debug_utils_messenger(&create_info, None)? };
            Ok(Some(Self { loader, messenger }))
        }
    }
}

impl Drop for DebugMessenger {
    fn drop(&mut self) {
        unsafe {
            self.loader
                .destroy_debug_utils_messenger(self.messenger, None);
        }
    }
}
