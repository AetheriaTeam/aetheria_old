use crate::vulkan::{
    device::{Device, Swapchain},
    instance::Instance,
};

use ash::prelude::VkResult;

pub struct Context {
    pub entry: ash::Entry,
    pub instance: Instance,
    pub surface: ash::vk::SurfaceKHR,
    pub device: Device,
    pub swapchain: Swapchain,
}

impl Context {
    pub fn new(window: &winit::window::Window) -> VkResult<Context> {
        let layers = ["VK_LAYER_KHRONOS_validation"];

        let entry = unsafe { ash::Entry::load().unwrap() };
        let instance = Instance::new(&entry, &layers)?;
        let surface = instance.create_surface(window)?;
        let physical = instance
            .pick_physical_device(&surface)
            .expect("No suitable device found");
        let device = Device::new(&instance, physical, &layers)?;
        let swapchain = device.create_swapchain(&surface, window)?;

        Ok(Context {
            entry,
            instance,
            surface,
            device,
            swapchain,
        })
    }
}
