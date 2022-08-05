use ash::{prelude::VkResult, vk};
use std::ffi::CString;

pub struct QueueFamilies {
    pub graphics: Option<u32>,
}

impl QueueFamilies {
    fn new() -> Self {
        QueueFamilies { graphics: None }
    }

    fn is_complete(&self) -> bool {
        return self.graphics.is_some();
    }
}

impl Default for QueueFamilies {
    fn default() -> Self {
        QueueFamilies::new()
    }
}

pub struct PhysicalDevice {
    pub handle: vk::PhysicalDevice,
    pub families: QueueFamilies
}

pub trait Instance {
    fn create(entry: &ash::Entry, extensions: Vec<*const i8>, layers: &[&str]) -> VkResult<ash::Instance>;
    fn pick_physical_device(&self) -> Option<PhysicalDevice>;
}

impl Instance for ash::Instance {
    fn create(entry: &ash::Entry, extensions: Vec<*const i8>, layers: &[&str]) -> VkResult<ash::Instance> {
        let app_info = vk::ApplicationInfo {
            api_version: vk::make_api_version(0, 1, 3, 0),
            ..Default::default()
        };

        let layers_cstr: Vec<CString> = layers
            .iter()
            .map(|layer| CString::new(*layer).unwrap())
            .collect();
        let layers_ptrs: Vec<*const i8> = layers_cstr.iter().map(|str| str.as_ptr()).collect();

        let instance_builder = vk::InstanceCreateInfo::builder()
            .application_info(&app_info)
            .enabled_extension_names(&extensions[..])
            .enabled_layer_names(&layers_ptrs[..]);

        let instance = unsafe {
            entry
                .create_instance(&instance_builder, None)?
        };

        println!("Vulkan instance created");

        Ok(instance)
    }

    fn pick_physical_device(&self) -> Option<PhysicalDevice> {
        let physicals = unsafe { self.enumerate_physical_devices().expect("Failed to get physical devices") };
        for physical in physicals.iter() {
            let family_properties = unsafe { self.get_physical_device_queue_family_properties(*physical) };
            let mut families = QueueFamilies::new();

            for (i, family) in family_properties.iter().enumerate() {
                if family.queue_flags.contains(vk::QueueFlags::GRAPHICS) {
                    families.graphics = Some(i as u32);
                }
            }

            if families.is_complete() {
                return Some(PhysicalDevice {
                    handle: *physical,
                    families
                });
            }
        }
        
        None
    } 
}
