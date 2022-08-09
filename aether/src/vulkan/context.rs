use bytemuck::{Pod, Zeroable};
use eyre::Result;
use std::{collections::HashSet, sync::Arc};
use vulkano::{
    device::{
        physical::PhysicalDevice, Device, DeviceCreateInfo, DeviceExtensions, Queue,
        QueueCreateInfo,
    },
    format::Format,
    image::{ImageUsage, SwapchainImage},
    instance::{Instance, InstanceCreateInfo},
    swapchain::{ColorSpace, PresentMode, Surface, Swapchain, SwapchainCreateInfo}, pipeline::graphics,
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

#[repr(C)]
#[derive(Debug, Default, Clone, Pod, Zeroable, Copy)]
pub struct Vertex {
    pub position: [f32; 2],
}
vulkano::impl_vertex!(Vertex, position);

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
        let physical = match PhysicalDevice::enumerate(&instance)
            .filter(move |physical| {
                physical
                    .supported_extensions()
                    .is_superset_of(&device_extensions)
            })
            .next()
        {
            Some(physical) => physical,
            None => panic!("No devices supporting vulkan found"),
        };

        println!("Using device {}", physical.properties().device_name);

        let graphics_family = match physical
            .queue_families()
            .find(|family| family.supports_graphics())
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
                    physical
                        .queue_families()
                        .find(|family| family.id() == *queue_family)
                        .unwrap(),
                )
            })
            .collect();

        let (device, mut queues) = match Device::new(
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
            if id == graphics_family.id() { graphics = Some(queue.clone()); }
            if id == present_family.id() { present = Some(queue.clone()); }
        }

        let capabilities = physical
            .surface_capabilities(&surface, Default::default())
            .unwrap();
        let formats = physical
            .surface_formats(&surface, Default::default())
            .unwrap();
        let mut modes = physical.surface_present_modes(&surface).unwrap();

        let num_images =
            (capabilities.min_image_count + 1).min(capabilities.max_image_count.unwrap());

        let (format, colorspace) = if let Some(format) =
            formats.iter().find(|(format, colorspace)| {
                *format == Format::B8G8R8A8_SRGB && *colorspace == ColorSpace::SrgbNonLinear
            }) {
            format
        } else {
            formats.first().unwrap()
        };

        let mode = if let Some(mode) = modes.find(|mode| *mode == PresentMode::Mailbox) {
            mode
        } else {
            PresentMode::Fifo
        };

        let dimensions = surface.window().inner_size();
        let composite_alpha = capabilities
            .supported_composite_alpha
            .iter()
            .next()
            .unwrap();

        let (swapchain, images) = Swapchain::new(
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
        )
        .unwrap();

        Ok(Self {
            surface,
            device,
            graphics: graphics.unwrap(),
            present: present.unwrap(),
            swapchain,
            images,
        })
    }
}
