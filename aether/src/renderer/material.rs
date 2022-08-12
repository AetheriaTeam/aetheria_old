use std::sync::Arc;

use vulkano::{
    command_buffer::{AutoCommandBufferBuilder, PrimaryAutoCommandBuffer},
    pipeline::GraphicsPipeline,
};

#[derive(Clone, Debug)]
pub struct Material {
    pipeline: Arc<GraphicsPipeline>,
}

impl Material {
    #[must_use]
    pub fn new(pipeline: Arc<GraphicsPipeline>) -> Arc<Self> {
        Arc::new(Self { pipeline })
    }
}

pub trait BindMaterial {
    fn bind_material(&mut self, material: Arc<Material>) -> &mut Self;
}

impl BindMaterial for AutoCommandBufferBuilder<PrimaryAutoCommandBuffer> {
    fn bind_material(&mut self, material: Arc<Material>) -> &mut Self {
        self.bind_pipeline_graphics(material.pipeline.clone())
    }
}
