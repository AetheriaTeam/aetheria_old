use std::fmt::Debug;

#[derive(Clone, Debug, Default)]
pub struct Size<T : Debug + Default>
{
    pub width: T,
    pub height: T
}

impl From<ash::vk::Extent2D> for Size<u32> {
    fn from(extent: ash::vk::Extent2D) -> Self {
       Self { width: extent.width, height: extent.height }
    }
}

impl From<Size<u32>> for ash::vk::Extent2D {
    fn from(size: Size<u32>) -> Self {
        Self { width: size.width, height: size.height }
    }
}

impl From<Size<u32>> for ash::vk::Extent3D {
    fn from(size: Size<u32>) -> Self {
        Self { width: size.width, height: size.height, depth: 1}
    }
}
