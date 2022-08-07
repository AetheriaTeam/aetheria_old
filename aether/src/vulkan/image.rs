use crate::prelude::*;
use crate::vulkan::device::Device;
use ash::{prelude::*, vk};

#[derive(Debug, Default)]
pub struct Config {
    pub size: math::Size<u32>,
    pub existing_handle: Option<vk::Image>,
    pub usage: Option<vk::ImageUsageFlags>,
    pub format: vk::Format,
}

pub struct Image {
    pub handle: vk::Image,
    pub size: math::Size<u32>,
    pub format: vk::Format,
}

impl Image {
    #[doc = "# Panics"]
    #[doc = "Panics if the config is invalid, i.e no usage or external handle"]
    pub fn new(device: &Device, config: Config) -> VkResult<Self> {
        // Wrap around existing image handles so swapchain images can have the wrapping
        match config.existing_handle {
            Some(value) => {
                return Ok(Self {
                    handle: value,
                    size: config.size,
                    format: config.format,
                })
            }
            None => (),
        };

        let image_info = vk::ImageCreateInfo::builder()
            .usage(match config.usage {
                Some(usage) => usage,
                None => panic!("All images must have a usage")
            })
            .format(config.format)
            .extent(config.size.clone().into())
            .tiling(vk::ImageTiling::LINEAR)
            .samples(vk::SampleCountFlags::TYPE_1)
            .image_type(vk::ImageType::TYPE_2D)
            .mip_levels(1)
            .array_layers(1)
            .sharing_mode(vk::SharingMode::EXCLUSIVE);

        let handle = unsafe { device.create_image(&image_info, None)? };

        Ok(Self {
            handle,
            size: config.size,
            format: config.format,
        })
    }

    #[doc = "# Errors"]
    #[doc = "Errors if internal ash functions fail"]
    pub fn create_full_view(
        &self,
        device: &Device,
        aspect: vk::ImageAspectFlags,
    ) -> VkResult<vk::ImageView> {
        let subresource_range = vk::ImageSubresourceRange::builder()
            .aspect_mask(aspect)
            .level_count(1)
            .layer_count(1)
            .base_mip_level(0)
            .base_array_layer(0);

        let view_info = vk::ImageViewCreateInfo::builder()
            .format(self.format)
            .image(self.handle)
            .view_type(vk::ImageViewType::TYPE_2D)
            .subresource_range(*subresource_range);

        unsafe { device.create_image_view(&view_info, None) }
    }
}
