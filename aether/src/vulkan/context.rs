use crate::vulkan::{
    device::{Device, Swapchain},
    instance::Instance,
};

use eyre::Result;

pub struct Context {
    pub entry: ash::Entry,
    pub instance: Instance,
    pub surface: ash::vk::SurfaceKHR,
    pub device: Device,
    pub swapchain: Swapchain,
}

impl Context {
    #[doc = "# Panics"]
    #[doc = "Will panic if no suitable devices are found"]
    pub fn new(window: &winit::window::Window) -> Result<Self> {
        let layers = ["VK_LAYER_KHRONOS_validation"];

        let entry = unsafe { ash::Entry::load()? };
        let instance = Instance::new(&entry, &layers)?;
        let surface = instance.create_surface(window)?;
        let physical = match instance.pick_physical_device(&surface) {
            None => panic!("No suitable GPU found"),
            Some(physical) => physical,
        };
        let device = Device::new(&instance, physical, &layers)?;
        let swapchain = device.create_swapchain(&surface, window)?;

        Ok(Self {
            entry,
            instance,
            surface,
            device,
            swapchain,
        })
    }
}
