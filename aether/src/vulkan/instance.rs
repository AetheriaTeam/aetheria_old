use std::{ffi::CString, ops::Deref, collections::HashSet};

use ash::extensions::khr::Surface;
#[cfg(target_os = "windows")]
use ash::extensions::khr::Win32Surface;
#[cfg(target_os = "linux")]
use ash::extensions::khr::XlibSurface;
use ash::{prelude::VkResult, vk};

#[derive(Debug, Default)]
pub struct QueueFamilies {
    pub graphics: Option<u32>,
    pub present: Option<u32>,
}

impl QueueFamilies {
    fn new() -> Self {
        QueueFamilies {
            graphics: None,
            present: None,
        }
    }

    pub fn is_complete(&self) -> bool {
        self.graphics.is_some() && self.present.is_some()
    }

    pub fn to_vec(&self) -> Option<Vec<u32>> {
        if self.is_complete() {
            Some(vec![self.graphics.unwrap(), self.present.unwrap()])
        } else {
            None
        }
    }

    pub fn get_unique_families(&self) -> Option<HashSet<u32>> {
        let families = self.to_vec();
        match families {
            Some(families) => {
                let mut set: HashSet<u32> = HashSet::new();
                for family in families.iter() {
                    set.insert(*family);
                }
                Some(set)
            },
            None => None
        }
    }
}

#[derive(Debug)]
pub struct SwapchainSupportInfo {
    pub capabilities: vk::SurfaceCapabilitiesKHR,
    pub formats: Vec<vk::SurfaceFormatKHR>,
    pub modes: Vec<vk::PresentModeKHR>
}

impl SwapchainSupportInfo {
    pub fn new(instance: &Instance, physical: &vk::PhysicalDevice, surface: &vk::SurfaceKHR) -> VkResult<SwapchainSupportInfo> {
        unsafe {
            let capabilities = instance.extensions.surface.get_physical_device_surface_capabilities(*physical, *surface)?;
            let formats = instance.extensions.surface.get_physical_device_surface_formats(*physical, *surface)?;
            let modes = instance.extensions.surface.get_physical_device_surface_present_modes(*physical, *surface)?;

            Ok(SwapchainSupportInfo {
                capabilities,
                formats,
                modes
            })
        }
    }
}

#[derive(Debug)]
pub struct PhysicalDevice {
    pub handle: vk::PhysicalDevice,
    pub families: QueueFamilies,
    pub swapchain: SwapchainSupportInfo
}

pub struct InstanceExtensions {
    pub surface: ash::extensions::khr::Surface,
    pub win32_surface: Option<ash::extensions::khr::Win32Surface>,
    pub xlib_surface: Option<ash::extensions::khr::XlibSurface>,
}

impl InstanceExtensions {
    #[cfg(target_os = "windows")]
    fn get_names() -> Vec<*const i8> {
        vec![Surface::name().as_ptr(), Win32Surface::name().as_ptr()]
    }

    #[cfg(target_os = "linux")]
    fn get_names() -> Vec<*const i8> {
        vec![Surface::name().as_ptr(), XlibSurface::name().as_ptr()]
    }

    #[cfg(target_os = "windows")]
    fn load(entry: &ash::Entry, instance: &ash::Instance) -> InstanceExtensions {
        InstanceExtensions {
            surface: ash::extensions::khr::Surface::new(entry, instance),
            win32_surface: Some(ash::extensions::khr::Win32Surface::new(entry, instance)),
            xlib_surface: None,
        }
    }

    #[cfg(target_os = "linux")]
    fn load(entry: &ash::Entry, instance: &ash::Instance) -> InstanceExtensions {
        InstanceExtensions {
            surface: ash::extensions::khr::Surface::new(entry, instance),
            win32_surface: None,
            xlib_surface: Some(ash::extensions::khr::XlibSurface::new(entry, instance)),
        }
    }
}

pub struct Instance {
    pub handle: ash::Instance,
    pub extensions: InstanceExtensions,
}

impl Instance {
    pub fn new(entry: &ash::Entry, layers: &[&str]) -> VkResult<Instance> {
        let app_info = vk::ApplicationInfo {
            api_version: vk::make_api_version(0, 1, 3, 0),
            ..Default::default()
        };

        let layers_cstr: Vec<CString> = layers
            .iter()
            .map(|layer| CString::new(*layer).unwrap())
            .collect();
        let layers_ptrs: Vec<*const i8> = layers_cstr.iter().map(|str| str.as_ptr()).collect();

        let extension_names = InstanceExtensions::get_names();

        let instance_builder = vk::InstanceCreateInfo::builder()
            .application_info(&app_info)
            .enabled_extension_names(&extension_names)
            .enabled_layer_names(&layers_ptrs);

        let handle = unsafe { entry.create_instance(&instance_builder, None)? };
        let extensions = InstanceExtensions::load(entry, &handle);

        println!("Vulkan instance created");

        Ok(Instance { handle, extensions })
    }

    pub fn pick_physical_device(&self, surface: &vk::SurfaceKHR) -> Option<PhysicalDevice> {
        let physicals = unsafe {
            self.enumerate_physical_devices()
                .expect("Failed to get physical devices")
        };

        for physical in physicals.iter() {
            let family_properties =
                unsafe { self.get_physical_device_queue_family_properties(*physical) };
            let mut families = QueueFamilies::new();
            let swapchain_info = unsafe { SwapchainSupportInfo::new(self, physical, surface).unwrap() };

            for (i, family) in family_properties.iter().enumerate() {
                if family.queue_flags.contains(vk::QueueFlags::GRAPHICS) {
                    families.graphics = Some(i as u32);
                }

                if unsafe {
                    self.extensions
                        .surface
                        .get_physical_device_surface_support(*physical, i as u32, *surface)
                        .unwrap()
                } {
                    families.present = Some(i as u32);
                }
            }

            let swapchain_supported = !swapchain_info.formats.is_empty() && !swapchain_info.modes.is_empty();

            if families.is_complete() && swapchain_supported {
                return Some(PhysicalDevice {
                    handle: *physical,
                    families,
                    swapchain: swapchain_info
                });
            }
        }

        None
    }

    #[cfg(target_os = "windows")]
    pub fn create_surface(&self, window: &winit::window::Window) -> VkResult<vk::SurfaceKHR> {
        todo!();
    }

    #[cfg(target_os = "linux")]
    pub fn create_surface(&self, window: &winit::window::Window) -> VkResult<vk::SurfaceKHR> {
        use winit::platform::unix::WindowExtUnix;

        let display = window.xlib_display().unwrap();

        let surface_info = vk::XlibSurfaceCreateInfoKHR::builder()
            .dpy(display as *mut vk::Display)
            .window(window.xlib_window().unwrap());

        unsafe {
            self.extensions
                .xlib_surface
                .as_ref()
                .unwrap()
                .create_xlib_surface(&surface_info, None)
        }
    }
}

impl Deref for Instance {
    type Target = ash::Instance;
    fn deref(&self) -> &Self::Target {
        &self.handle
    }
}
