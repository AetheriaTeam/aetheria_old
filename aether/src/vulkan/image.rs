use crate::prelude::*;
use crate::vulkan::device::Device;
use ash::{prelude::*, vk};

#[derive(Debug, Default)]
pub struct ImageConfig {
    pub size: math::Size<u32>,
    pub existing_handle: Option<vk::Image>,
    pub usage: Option<vk::ImageUsageFlags>,
    pub format: vk::Format
}

pub struct Image {
    pub handle: vk::Image,
    pub size: math::Size<u32>,
    pub format: vk::Format
}

impl Image {
    pub fn new(device: &Device, config: ImageConfig) -> VkResult<Image> {
        // Wrap around existing image handles so swapchain images can have the wrapping
        if config.existing_handle.is_some() {
            return Ok(Image {
                handle: config.existing_handle.unwrap(),
                size: config.size,
                format: config.format
            })
        }

        let image_info = vk::ImageCreateInfo::builder()
            .usage(config.usage.expect("All images need to have a usage"))
            .format(config.format)
            .extent(config.size.to_extent())
            .tiling(vk::ImageTiling::LINEAR)
            .samples(vk::SampleCountFlags::TYPE_1)
            .image_type(vk::ImageType::TYPE_2D)
            .mip_levels(1)
            .array_layers(1)
            .sharing_mode(vk::SharingMode::EXCLUSIVE);

        let handle = unsafe { device.create_image(&image_info, None)? };

        Ok(Image {
            handle,
            size: config.size,
            format: config.format
        })
    }

    pub fn create_full_view(&self, device: &Device, aspect: vk::ImageAspectFlags) -> VkResult<vk::ImageView> {
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
