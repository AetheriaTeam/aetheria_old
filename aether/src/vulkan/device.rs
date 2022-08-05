use crate::vulkan::instance::PhysicalDevice;

use ash::{prelude::VkResult, vk};
use std::ffi::CString;
use std::ops::Deref;

pub struct DeviceExtensions {
    swapchain: ash::extensions::khr::Swapchain,
}

impl DeviceExtensions {
    pub fn get_names() -> Vec<*const i8> {
        vec![ash::extensions::khr::Swapchain::name().as_ptr()]
    }

    pub fn load(instance: &ash::Instance, device: &ash::Device) -> DeviceExtensions {
        DeviceExtensions {
            swapchain: ash::extensions::khr::Swapchain::new(instance, device),
        }
    }
}

pub struct Queues {
    graphics: vk::Queue,
    present: vk::Queue,
}

impl Queues {
    pub unsafe fn new(physical: &PhysicalDevice, device: &ash::Device) -> Queues {
        Queues {
            graphics: device.get_device_queue(physical.families.graphics.unwrap(), 0),
            present: device.get_device_queue(physical.families.present.unwrap(), 0),
        }
    }
}

pub struct Device {
    pub handle: ash::Device,
    pub physical: PhysicalDevice,
    pub extensions: DeviceExtensions,
    pub queues: Queues,
}

pub struct Swapchain {
    pub handle: vk::SwapchainKHR,
    pub images: Vec<vk::Image>,
    pub format: vk::SurfaceFormatKHR,
    pub extent: vk::Extent2D,
}

impl Device {
    pub fn new(
        instance: &ash::Instance,
        physical: PhysicalDevice,
        layers: &[&str],
    ) -> VkResult<Device> {
        let unique_families = physical.families.get_unique_families().unwrap();

        let queue_priorities: [f32; 1] = [1.0];
        let queue_infos: Vec<vk::DeviceQueueCreateInfo> = unique_families
            .iter()
            .map(|family| {
                vk::DeviceQueueCreateInfo::builder()
                    .queue_family_index(*family)
                    .queue_priorities(&queue_priorities)
                    .build()
            })
            .collect();

        let layers_cstr: Vec<CString> = layers
            .iter()
            .map(|layer| CString::new(*layer).unwrap())
            .collect();
        let layers_ptrs: Vec<*const i8> = layers_cstr.iter().map(|str| str.as_ptr()).collect();

        let extension_names = DeviceExtensions::get_names();

        let device_info = vk::DeviceCreateInfo::builder()
            .enabled_extension_names(&extension_names)
            .enabled_layer_names(&layers_ptrs)
            .queue_create_infos(&queue_infos);

        let handle = unsafe { instance.create_device(physical.handle, &device_info, None)? };
        let extensions = DeviceExtensions::load(instance, &handle);
        let queues = unsafe { Queues::new(&physical, &handle) };

        println!("Vulkan device created");

        Ok(Device {
            handle,
            physical,
            extensions,
            queues,
        })
    }

    fn choose_extent(&self, window: &winit::window::Window) -> vk::Extent2D {
        if self.physical.swapchain.capabilities.current_extent.width != u32::MAX
            && self.physical.swapchain.capabilities.current_extent.height != u32::MAX
        {
            self.physical.swapchain.capabilities.current_extent
        } else {
            let size = window.inner_size();
            vk::Extent2D {
                width: size
                    .width
                    .max(self.physical.swapchain.capabilities.min_image_extent.width)
                    .min(self.physical.swapchain.capabilities.max_image_extent.width),
                height: size
                    .height
                    .max(self.physical.swapchain.capabilities.min_image_extent.height)
                    .min(self.physical.swapchain.capabilities.max_image_extent.height),
            }
        }
    }

    pub fn create_swapchain(
        &self,
        surface: &vk::SurfaceKHR,
        window: &winit::window::Window,
    ) -> VkResult<Swapchain> {
        let optimal_formats: Vec<&vk::SurfaceFormatKHR> = self
            .physical
            .swapchain
            .formats
            .iter()
            .filter(|format| {
                format.format == vk::Format::B8G8R8A8_SRGB
                    && format.color_space == vk::ColorSpaceKHR::SRGB_NONLINEAR
            })
            .collect();
        let format;
        if optimal_formats.is_empty() {
            format = *self.physical.swapchain.formats.first().unwrap();
        } else {
            format = **optimal_formats.first().unwrap();
        }

        let optimal_modes: Vec<&vk::PresentModeKHR> = self
            .physical
            .swapchain
            .modes
            .iter()
            .filter(|mode| **mode == vk::PresentModeKHR::MAILBOX)
            .collect();
        let mode;
        if optimal_modes.is_empty() {
            mode = vk::PresentModeKHR::FIFO;
        } else {
            mode = **optimal_modes.first().unwrap();
        }
        let extent = self.choose_extent(window);

        let mut num_images = self.physical.swapchain.capabilities.min_image_count + 1;
        if self.physical.swapchain.capabilities.max_image_count != 0 {
            num_images = num_images.min(self.physical.swapchain.capabilities.max_image_count);
        }

        let queues = [
            self.physical.families.graphics.unwrap(),
            self.physical.families.present.unwrap(),
        ];

        let mut swapchain_info = vk::SwapchainCreateInfoKHR::builder()
            .surface(*surface)
            .min_image_count(num_images)
            .image_format(format.format)
            .image_color_space(format.color_space)
            .image_extent(extent)
            .image_array_layers(1)
            .image_usage(vk::ImageUsageFlags::COLOR_ATTACHMENT);

        if self.physical.families.graphics == self.physical.families.present {
            swapchain_info = swapchain_info.image_sharing_mode(vk::SharingMode::EXCLUSIVE);
        } else {
            swapchain_info = swapchain_info
                .image_sharing_mode(vk::SharingMode::CONCURRENT)
                .queue_family_indices(&queues);
        }

        swapchain_info = swapchain_info
            .pre_transform(self.physical.swapchain.capabilities.current_transform)
            .composite_alpha(vk::CompositeAlphaFlagsKHR::OPAQUE)
            .present_mode(mode)
            .clipped(true);

        let handle = unsafe {
            self.extensions
                .swapchain
                .create_swapchain(&swapchain_info, None)?
        };
        let images = unsafe { self.extensions.swapchain.get_swapchain_images(handle)? };

        Ok(Swapchain {
            handle,
            images,
            format,
            extent,
        })
    }
}

impl Deref for Device {
    type Target = ash::Device;

    fn deref(&self) -> &Self::Target {
        &self.handle
    }
}

impl Deref for Swapchain {
    type Target = vk::SwapchainKHR;

    fn deref(&self) -> &Self::Target {
        &self.handle
    }
}
