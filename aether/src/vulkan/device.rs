use crate::prelude::*;
use crate::vulkan::image::{Image, self};
use crate::vulkan::instance::PhysicalDevice;

use ash::{prelude::VkResult, vk};
use std::ffi::CString;
use std::ops::Deref;

pub struct Extensions {
    swapchain: ash::extensions::khr::Swapchain,
}

impl Extensions {
    fn get_names() -> Vec<*const i8> {
        vec![ash::extensions::khr::Swapchain::name().as_ptr()]
    }

    fn load(instance: &ash::Instance, device: &ash::Device) -> Self {
        Self {
            swapchain: ash::extensions::khr::Swapchain::new(instance, device),
        }
    }
}

pub struct Queues {
    graphics: vk::Queue,
    present: vk::Queue,
}

impl Queues {
    fn new(physical: &PhysicalDevice, device: &ash::Device) -> Self {
        unsafe {
            Self {
                graphics: device.get_device_queue(physical.families.graphics, 0),
                present: device.get_device_queue(physical.families.present, 0),
            }
        }
    }
}

pub struct Device {
    pub handle: ash::Device,
    pub physical: PhysicalDevice,
    pub extensions: Extensions,
    pub queues: Queues,
}

pub struct Swapchain {
    pub handle: vk::SwapchainKHR,
    pub images: Vec<Image>,
    pub views: Vec<vk::ImageView>,
    pub format: vk::SurfaceFormatKHR,
    pub extent: vk::Extent2D,
}

impl Device {
    #[doc = "# Errors"]
    #[doc = "Errors if an internal ash function fails"]
    pub fn new(
        instance: &ash::Instance,
        physical: PhysicalDevice,
        layers: &[&str],
    ) -> VkResult<Self> {
        let unique_families = physical.families.get_unique_families();

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
            .map(|layer| match CString::new(*layer) {
                Err(e) => panic!("Failed to convert {} into a CString due to {}", layer, e),
                Ok(value) => value,
            })
            .collect();
        let layers_ptrs: Vec<*const i8> = layers_cstr.iter().map(|str| str.as_ptr()).collect();

        let extension_names = Extensions::get_names();

        let device_info = vk::DeviceCreateInfo::builder()
            .enabled_extension_names(&extension_names)
            .enabled_layer_names(&layers_ptrs)
            .queue_create_infos(&queue_infos);

        let handle = unsafe { instance.create_device(physical.handle, &device_info, None)? };
        let extensions = Extensions::load(instance, &handle);
        let queues = Queues::new(&physical, &handle);

        println!("Vulkan device created");

        Ok(Self {
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

    #[doc = "# Panics"]
    #[doc = "Panics if there are no swapchain formats or no present modes"]
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
        let format = match optimal_formats.first() {
            Some(value) => **value,
            None => {
                // No optimal formats, just choose the first one
                match self.physical.swapchain.formats.first() {
                    Some(value) => *value,
                    None => panic!("No swapchain formats"),
                }
            }
        };

        let optimal_modes: Vec<&vk::PresentModeKHR> = self
            .physical
            .swapchain
            .modes
            .iter()
            .filter(|mode| **mode == vk::PresentModeKHR::MAILBOX)
            .collect();
        let mode = match optimal_modes.first() {
            Some(value) => **value,
            None => vk::PresentModeKHR::FIFO,
        };

        let extent = self.choose_extent(window);

        let mut num_images = self.physical.swapchain.capabilities.min_image_count + 1;
        if self.physical.swapchain.capabilities.max_image_count != 0 {
            num_images = num_images.min(self.physical.swapchain.capabilities.max_image_count);
        }

        let queues = [
            self.physical.families.graphics,
            self.physical.families.present,
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
        let image_handles = unsafe { self.extensions.swapchain.get_swapchain_images(handle)? };
        let images: Vec<Image> = image_handles
            .iter()
            .map(move |handle| {
                let config = image::Config {
                    existing_handle: Some(*handle),
                    size: math::Size {
                        width: extent.width,
                        height: extent.height,
                    },
                    format: format.format,
                    ..Default::default()
                };

                match Image::new(self, config) {
                    Ok(image) => image,
                    Err(e) => panic!("Swapchain image wrapping failed because {}", e),
                }
            })
            .collect();
        let views = images
            .iter()
            .map(move |image| match image.create_full_view(self, vk::ImageAspectFlags::COLOR) {
                Ok(value) => value,
                Err(e) => panic!("Swapchain image view creation failed because {}", e)
            })
            .collect();

        Ok(Swapchain {
            handle,
            images,
            views,
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
