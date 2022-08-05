use crate::vulkan::{device::{Device}, instance::{PhysicalDevice, Instance}};
use ash::prelude::VkResult;

#[cfg(target_os = "windows")]
use ash::extensions::khr::Win32Surface;
#[cfg(target_os = "linux")]
use ash::extensions::khr::XlibSurface;

use ash::extensions::khr::Surface;

#[cfg(target_os = "windows")]
fn required_extensions() -> Vec<*const i8> {
    vec![
        Win32Surface::name().as_ptr(),
        Surface::name().as_ptr()
    ]
}

#[cfg(target_os = "linux")]
fn required_extensions() -> Vec<*const i8> {
    vec![
        XlibSurface::name().as_ptr(),
        Surface::name().as_ptr()
    ]
}

pub struct Context {
    entry: ash::Entry,
    instance: ash::Instance,
    physical: PhysicalDevice,
    device: ash::Device
}

impl Context {
    pub fn new() -> VkResult<Context> {
        let layers = ["VK_LAYER_KHRONOS_validation"];
        let extensions = required_extensions();

        let entry = unsafe { ash::Entry::load().unwrap() };
        let instance = ash::Instance::create(&entry, extensions, &layers)?;
        let physical = instance.pick_physical_device().expect("No suitable device found");
        let device = ash::Device::create(&instance, &physical, vec![], &layers)?;

        Ok(Context {
            entry,
            instance,
            physical,
            device
        })
    }
}
