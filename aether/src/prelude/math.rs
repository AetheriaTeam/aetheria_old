use std::fmt::Debug;

#[derive(Debug, Default)]
pub struct Size<T : Debug + Default>
{
    pub width: T,
    pub height: T
}

impl Size<u32> {
    pub fn to_extent(&self) -> ash::vk::Extent3D {
        ash::vk::Extent3D { width: self.width, height: self.height, depth: 1 }
    }
}
