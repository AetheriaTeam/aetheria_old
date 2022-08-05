use crate::vulkan::instance;

use std::ffi::CString;
use ash::{vk, prelude::VkResult};

pub trait Device {
    fn create(instance: &ash::Instance, physical: &instance::PhysicalDevice, extensions: Vec<*const i8>, layers: &[&str]) -> VkResult<ash::Device>;
}

impl Device for ash::Device {
    fn create(instance: &ash::Instance, physical: &instance::PhysicalDevice, extensions: Vec<*const i8>, layers: &[&str]) -> VkResult<ash::Device> {
        let queue_priorities: [f32; 1] = [1.0];
        let queue_info = vk::DeviceQueueCreateInfo::builder()
            .queue_family_index(physical.families.graphics.unwrap())
            .queue_priorities(&queue_priorities);
        let queue_infos = [*queue_info];

        let layers_cstr: Vec<CString> = layers
            .iter()
            .map(|layer| CString::new(*layer).unwrap())
            .collect();
        let layers_ptrs: Vec<*const i8> = layers_cstr.iter().map(|str| str.as_ptr()).collect();

        let device_info = vk::DeviceCreateInfo::builder()
            .enabled_extension_names(&extensions)
            .enabled_layer_names(&layers_ptrs)
            .queue_create_infos(&queue_infos);
    
        let device = unsafe {
            instance.create_device(physical.handle, &device_info, None)
        };

        println!("Vulkan device created");

        device

    }
}