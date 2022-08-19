use std::sync::Arc;

use vulkano::command_buffer::{AutoCommandBufferBuilder, PrimaryAutoCommandBuffer};

#[derive(Clone, Debug)]
pub struct Material {}

impl Material {
    #[must_use]
    pub fn new() -> Arc<Self> {
        Arc::new(Self {})
    }
}

pub trait Bind {
    fn bind_material(&mut self, material: Arc<Material>) -> &mut Self;
}

impl Bind for AutoCommandBufferBuilder<PrimaryAutoCommandBuffer> {
    fn bind_material(&mut self, _material: Arc<Material>) -> &mut Self {
        self
    }
}
