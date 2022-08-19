use std::sync::Arc;

use eyre::Context;
use serde::{Deserialize, Serialize};
use vulkano::buffer::{BufferUsage, CpuAccessibleBuffer};

use crate::{
    renderer::material::Material,
    vulkan::{context, vertex::Vertex},
};

#[derive(Clone)]
pub struct Mesh {
    pub vertex_buffer: Arc<CpuAccessibleBuffer<[Vertex]>>,
    pub index_buffer: Arc<CpuAccessibleBuffer<[u32]>>,
    pub material: Arc<Material>,
    pub num_indices: u32,
}

impl Mesh {
    #[doc = "# Errors"]
    #[doc = "Errors if vertex buffer creation failes"]
    pub fn from_data(
        ctx: &context::Context,
        data: &MeshData,
        material: Arc<Material>,
    ) -> eyre::Result<Self> {
        #[allow(clippy::expect_used)]
        Ok(Self {
            vertex_buffer: CpuAccessibleBuffer::from_iter(
                ctx.device.clone(),
                BufferUsage::vertex_buffer(),
                false,
                data.vertices.clone(),
            )
            .wrap_err("Failed to create vertex buffer for mesh")?,
            index_buffer: CpuAccessibleBuffer::from_iter(
                ctx.device.clone(),
                BufferUsage::index_buffer(),
                false,
                data.indices.clone(),
            )
            .wrap_err("Failed to create index buffer for mesh")?,
            material,
            num_indices: data.indices.len().try_into().expect("Failed to convert length of indices into a u32"),
        })
    }
}

#[allow(clippy::module_name_repetitions)] // Would be really annoying to have a struct called Data
#[derive(Deserialize, Serialize)]
pub struct MeshData {
    pub vertices: Vec<Vertex>,
    pub indices: Vec<u32>,
}
