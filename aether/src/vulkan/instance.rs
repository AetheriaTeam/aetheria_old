use std::{collections::HashSet, error::Error, ffi::CString, fmt::Display, ops::Deref};

use ash::extensions::khr::Surface;
#[cfg(target_os = "windows")]
use ash::extensions::khr::Win32Surface;
#[cfg(target_os = "linux")]
use ash::extensions::khr::XlibSurface;
use ash::{prelude::VkResult, vk};

#[derive(Debug, Clone)]
pub struct QueueFamilies {
    pub graphics: u32,
    pub present: u32,
}

impl QueueFamilies {
    #[must_use]
    pub fn to_vec(&self) -> Vec<u32> {
        vec![self.graphics, self.present]
    }

    #[must_use]
    pub fn get_unique_families(&self) -> HashSet<u32> {
        let families = self.to_vec();
        let mut set: HashSet<u32> = HashSet::new();
        for family in &families {
            set.insert(*family);
        }
        set
    }
}

#[derive(Debug, Default, Clone)]
pub struct PartialQueueFamilies {
    pub graphics: Option<u32>,
    pub present: Option<u32>,
}

impl PartialQueueFamilies {
    const fn new() -> Self {
        Self {
            graphics: None,
            present: None,
        }
    }

    #[must_use]
    pub const fn is_complete(&self) -> bool {
        self.graphics.is_some() && self.present.is_some()
    }
}

#[derive(Debug, Clone)]
pub struct IncompleteQueueFamiliesError {
    partial_families: PartialQueueFamilies,
}
impl Error for IncompleteQueueFamiliesError {}

impl Display for IncompleteQueueFamiliesError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        if self.partial_families.graphics.is_none() {
            write!(f, "Missing graphcis queue")?;
        };
        if self.partial_families.present.is_none() {
            write!(f, "Missing present queue")?;
        };
        Ok(())
    }
}

impl TryFrom<PartialQueueFamilies> for QueueFamilies {
    type Error = IncompleteQueueFamiliesError;

    fn try_from(partial_families: PartialQueueFamilies) -> Result<Self, Self::Error> {
        let err = IncompleteQueueFamiliesError {
            partial_families: partial_families.clone(),
        };
        Ok(Self {
            graphics: partial_families.graphics.ok_or_else(|| err.clone())?,
            present: partial_families.present.ok_or(err)?,
        })
    }
}

#[derive(Debug)]
pub struct SwapchainSupportInfo {
    pub capabilities: vk::SurfaceCapabilitiesKHR,
    pub formats: Vec<vk::SurfaceFormatKHR>,
    pub modes: Vec<vk::PresentModeKHR>,
}

impl SwapchainSupportInfo {
    #[doc = "# Errors"]
    #[doc = "Errors if an internal ash function fails"]
    pub fn new(
        instance: &Instance,
        physical: &vk::PhysicalDevice,
        surface: &vk::SurfaceKHR,
    ) -> VkResult<Self> {
        unsafe {
            let capabilities = instance
                .extensions
                .surface
                .get_physical_device_surface_capabilities(*physical, *surface)?;
            let formats = instance
                .extensions
                .surface
                .get_physical_device_surface_formats(*physical, *surface)?;
            let modes = instance
                .extensions
                .surface
                .get_physical_device_surface_present_modes(*physical, *surface)?;

            Ok(Self {
                capabilities,
                formats,
                modes,
            })
        }
    }
}

#[derive(Debug)]
pub struct PhysicalDevice {
    pub handle: vk::PhysicalDevice,
    pub families: QueueFamilies,
    pub swapchain: SwapchainSupportInfo,
}

pub struct Extensions {
    pub surface: ash::extensions::khr::Surface,
    pub win32_surface: Option<ash::extensions::khr::Win32Surface>,
    pub xlib_surface: Option<ash::extensions::khr::XlibSurface>,
}

impl Extensions {
    #[cfg(target_os = "windows")]
    fn get_names() -> Vec<*const i8> {
        vec![Surface::name().as_ptr(), Win32Surface::name().as_ptr()]
    }

    #[cfg(target_os = "linux")]
    fn get_names() -> Vec<*const i8> {
        vec![Surface::name().as_ptr(), XlibSurface::name().as_ptr()]
    }

    #[cfg(target_os = "windows")]
    fn load(entry: &ash::Entry, instance: &ash::Instance) -> Extensions {
        Extensions {
            surface: ash::extensions::khr::Surface::new(entry, instance),
            win32_surface: Some(ash::extensions::khr::Win32Surface::new(entry, instance)),
            xlib_surface: None,
        }
    }

    #[cfg(target_os = "linux")]
    fn load(entry: &ash::Entry, instance: &ash::Instance) -> Self {
        Self {
            surface: ash::extensions::khr::Surface::new(entry, instance),
            win32_surface: None,
            xlib_surface: Some(ash::extensions::khr::XlibSurface::new(entry, instance)),
        }
    }
}

pub struct Instance {
    pub handle: ash::Instance,
    pub extensions: Extensions,
}

impl Instance {
    #[doc = "# Panics"]
    #[doc = "Panics if a layer name couldn't be converted to a CString"]
    pub fn new(entry: &ash::Entry, layers: &[&str]) -> VkResult<Self> {
        let app_info = vk::ApplicationInfo {
            api_version: vk::make_api_version(0, 1, 3, 0),
            ..Default::default()
        };

        let layers_cstr: Vec<CString> = layers
            .iter()
            .map(|layer| match CString::new(*layer) {
                Ok(cstr) => cstr,
                Err(e) => panic!("Couldn't convert {} to a CString because {}", layer, e),
            })
            .collect();
        let layers_ptrs: Vec<*const i8> = layers_cstr.iter().map(|str| str.as_ptr()).collect();

        let extension_names = Extensions::get_names();

        let instance_builder = vk::InstanceCreateInfo::builder()
            .application_info(&app_info)
            .enabled_extension_names(&extension_names)
            .enabled_layer_names(&layers_ptrs);

        let handle = unsafe { entry.create_instance(&instance_builder, None)? };
        let extensions = Extensions::load(entry, &handle);

        println!("Vulkan instance created");

        Ok(Self { handle, extensions })
    }

    #[must_use]
    #[doc = "# Panics"]
    #[doc = "Panics if internal ash functions fail or if the number of physical devices is greater that [`u32::MAX`] (good luck installing 4,294,967,295 GPUs into your machine)"]
    pub fn pick_physical_device(&self, surface: &vk::SurfaceKHR) -> Option<PhysicalDevice> {
        let physicals = unsafe {
            match self.enumerate_physical_devices() {
                Ok(value) => value,
                Err(e) => panic!("Failed to get physical devices because {}", e),
            }
        };

        for physical in &physicals {
            let family_properties =
                unsafe { self.get_physical_device_queue_family_properties(*physical) };
            let mut partial_families = PartialQueueFamilies::new();
            let swapchain_info = match SwapchainSupportInfo::new(self, physical, surface) {
                Ok(value) => value,
                Err(e) => panic!("Failed to get swapchain support info because {}", e),
            };

            for (i_usize, family) in family_properties.iter().enumerate() {
                let i: u32 = match i_usize.try_into() {
                    Ok(value) => value,
                    Err(e) => panic!("Unable to cast i into u32 because {}", e)
                };

                if family.queue_flags.contains(vk::QueueFlags::GRAPHICS) {
                    partial_families.graphics = Some(i);
                }

                if match unsafe {
                    self.extensions
                        .surface
                        .get_physical_device_surface_support(*physical, i, *surface)
                } {
                    Ok(value) => value,
                    Err(e) => panic!("Failed to get queue family surface support because {}", e)
                } {
                    partial_families.present = Some(i);
                }
            }

            let swapchain_supported =
                !swapchain_info.formats.is_empty() && !swapchain_info.modes.is_empty();

            if partial_families.is_complete() && swapchain_supported {
                return Some(PhysicalDevice {
                    handle: *physical,
                    families: partial_families.try_into().ok()?,
                    swapchain: swapchain_info,
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
    #[doc = "# Panics"]
    #[doc = "Panics if [`winit::window::Window::xlib_window()`] or [`winit::window::Window::xlib_display()`] return [`None`]"]
    pub fn create_surface(&self, window: &winit::window::Window) -> VkResult<vk::SurfaceKHR> {
        use winit::platform::unix::WindowExtUnix;

        let display = match window.xlib_display() {
            Some(display) => display,
            None => panic!("Window doesn't have an XLib display")
        };

        let surface_info = vk::XlibSurfaceCreateInfoKHR::builder()
            .dpy(display.cast())
            .window(match window.xlib_window() {
                Some(value) => value,
                None => panic!("No XLib window")
            });
        
        match self.extensions.xlib_surface.as_ref() {
            None => panic!("XLib extension not loaded on linux machine, this should be impossible"),
            Some(ext) => unsafe { ext.create_xlib_surface(&surface_info, None) }
        }
    }
}

impl Deref for Instance {
    type Target = ash::Instance;
    fn deref(&self) -> &Self::Target {
        &self.handle
    }
}
