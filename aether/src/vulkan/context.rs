use eyre::Result;
use std::{collections::HashSet, sync::Arc};
use vulkano::{
    device::{
        physical::{PhysicalDevice, QueueFamily},
        Device, DeviceCreateInfo, DeviceExtensions, Queue, QueueCreateInfo,
    },
    format::Format,
    image::{ImageUsage, SwapchainImage},
    instance::{Instance, InstanceCreateInfo},
    swapchain::{ColorSpace, PresentMode, Surface, SurfaceInfo, Swapchain, SwapchainCreateInfo},
};
use vulkano_win::VkSurfaceBuild;
use winit::{event_loop::EventLoop, window::WindowBuilder};

#[derive(Clone, Debug)]
pub struct Context {
    pub surface: Arc<Surface<winit::window::Window>>,
    pub device: Arc<Device>,
    pub graphics: Arc<Queue>,
    pub present: Arc<Queue>,
    pub swapchain: Arc<Swapchain<winit::window::Window>>,
    pub images: Vec<Arc<SwapchainImage<winit::window::Window>>>,
}

impl Context {
    #[doc = "# Panics"]
    #[doc = "# Panics if the device is unsuitable for aether, or if an internal vulkan error occurs"]
    pub fn new(event_loop: &EventLoop<()>) -> Result<Self> {
        let required_extensions = vulkano_win::required_extensions();
        let instance = match Instance::new(InstanceCreateInfo {
            enabled_extensions: required_extensions,
            ..Default::default()
        }) {
            Ok(instance) => instance,
            Err(e) => panic!("Failed to create vulkan instance because {}", e),
        };

        let surface = match WindowBuilder::new().build_vk_surface(event_loop, instance.clone()) {
            Ok(surface) => surface,
            Err(e) => panic!("Failed to create surface because {}", e),
        };

        let device_extensions = DeviceExtensions {
            khr_swapchain: true,
            ..DeviceExtensions::none()
        };
        let physical = match PhysicalDevice::enumerate(&instance).find(|physical| {
            physical
                .supported_extensions()
                .is_superset_of(&device_extensions)
        }) {
            Some(physical) => physical,
            None => panic!("No devices supporting vulkan found"),
        };

        println!("Using device {}", physical.properties().device_name);

        let graphics_family = match physical
            .queue_families()
            .find(QueueFamily::supports_graphics)
        {
            None => panic!("No graphics queues"),
            Some(family) => family,
        };

        let present_family =
            match physical
                .queue_families()
                .find(|family| match family.supports_surface(&surface) {
                    Ok(value) => value,
                    Err(e) => panic!("Checking surface support failed because {}", e),
                }) {
                None => panic!("No present queues"),
                Some(family) => family,
            };

        let mut unique_queue_families: HashSet<u32> = HashSet::new();
        unique_queue_families.insert(graphics_family.id());
        unique_queue_families.insert(present_family.id());

        let queue_create_infos = unique_queue_families
            .iter()
            .map(|queue_family| {
                QueueCreateInfo::family(
                    match physical
                        .queue_families()
                        .find(|family| family.id() == *queue_family)
                    {
                        None => {
                            panic!("Failed to match family id to family, this should be impossible")
                        }
                        Some(family) => family,
                    },
                )
            })
            .collect();

        let (device, queues) = match Device::new(
            physical,
            DeviceCreateInfo {
                queue_create_infos,
                enabled_extensions: device_extensions,
                ..Default::default()
            },
        ) {
            Ok(device) => device,
            Err(e) => panic!("Device creation failed due to {}", e),
        };

        let mut graphics = None;
        let mut present = None;

        for queue in queues {
            let id = queue.family().id();
            if id == graphics_family.id() {
                graphics = Some(queue.clone());
            }
            if id == present_family.id() {
                present = Some(queue.clone());
            }
        }

        let (swapchain, images) = Self::create_swapchain(&device, &surface);

        Ok(Self {
            surface,
            device,
            graphics: match graphics {
                None => panic!("No graphics queue found"),
                Some(queue) => queue,
            },
            present: match present {
                None => panic!("No present queue found"),
                Some(queue) => queue,
            },
            swapchain,
            images,
        })
    }

    fn create_swapchain(
        device: &Arc<Device>,
        surface: &Arc<Surface<winit::window::Window>>,
    ) -> (
        Arc<Swapchain<winit::window::Window>>,
        Vec<Arc<SwapchainImage<winit::window::Window>>>,
    ) {
        let capabilities = match device
            .physical_device()
            .surface_capabilities(surface, SurfaceInfo::default())
        {
            Ok(capabilites) => capabilites,
            Err(e) => panic!("Failed to get physical device capabilities because {}", e),
        };
        let formats = match device
            .physical_device()
            .surface_formats(surface, SurfaceInfo::default())
        {
            Ok(formats) => formats,
            Err(e) => panic!("Failed to get surface formats because {}", e),
        };
        let mut modes = match device.physical_device().surface_present_modes(surface) {
            Ok(modes) => modes,
            Err(e) => panic!("Failed to get surface present modes because {}", e),
        };

        let mut num_images = capabilities.min_image_count + 1;
        if let Some(max_images) = capabilities.max_image_count {
            num_images = num_images.min(max_images);
        }

        let (format, colorspace) = formats
            .iter()
            .find(|(format, colorspace)| {
                *format == Format::B8G8R8A8_SRGB && *colorspace == ColorSpace::SrgbNonLinear
            })
            .map_or_else(
                || match formats.first() {
                    Some(format) => format,
                    None => panic!("No surface formats"),
                },
                |format| format,
            );

        let mode = modes
            .find(|mode| *mode == PresentMode::Mailbox)
            .map_or(PresentMode::Fifo, |mode| mode);

        let dimensions = surface.window().inner_size();
        let composite_alpha = match capabilities.supported_composite_alpha.iter().next() {
            Some(composite_alpha) => composite_alpha,
            None => panic!("No supported composite alphas"),
        };

        match Swapchain::new(
            device.clone(),
            surface.clone(),
            SwapchainCreateInfo {
                min_image_count: num_images,
                image_format: Some(*format),
                image_color_space: *colorspace,
                present_mode: mode,
                image_extent: dimensions.into(),
                composite_alpha,
                image_usage: ImageUsage::color_attachment(),
                ..Default::default()
            },
        ) {
            Ok(value) => value,
            Err(e) => panic!("Failed to create swapchain because {}", e),
        }
    }
}
