use std::{fmt::Debug, rc::Rc, sync::Arc};

use eyre::Context;
use vulkano::buffer::{BufferUsage, CpuAccessibleBuffer};

use crate::vulkan::vertex::Vertex;

use super::{
    material::{BindMaterial, Material},
    renderer::{CommandBufferBuilder, Drawable, Renderer},
};

#[derive(Clone, Debug)]
pub struct Mesh {
    vertex_buffer: Arc<CpuAccessibleBuffer<[Vertex]>>,
    index_buffer: Arc<CpuAccessibleBuffer<[u32]>>,
    material: Arc<Material>,
    num_vertices: u32,
}

impl Mesh {
    #[doc = "# Errors"]
    #[doc = "Errors if vertex buffer creation failes"]
    pub fn new(
        renderer: &Renderer,
        vertices: Vec<Vertex>,
        indices: Vec<u32>,
        material: Arc<Material>,
    ) -> eyre::Result<Rc<Self>> {
        let num_vertices = indices.len() as u32;
        Ok(Rc::new(Self {
            vertex_buffer: CpuAccessibleBuffer::from_iter(
                renderer.ctx.device.clone(),
                BufferUsage::vertex_buffer(),
                false,
                vertices,
            )
            .wrap_err("Failed to create vertex buffer for mesh")?,
            index_buffer: CpuAccessibleBuffer::from_iter(
                renderer.ctx.device.clone(),
                BufferUsage::index_buffer(),
                false,
                indices,
            )
            .wrap_err("Failed to create index buffer for mesh")?,
            material,
            num_vertices,
        }))
    }
}

impl Drawable for Mesh {
    fn draw<'a>(&'a self, cmd: &'a mut CommandBufferBuilder) -> eyre::Result<()> {
        cmd.bind_material(self.material.clone())
            .bind_index_buffer(self.index_buffer.clone())
            .bind_vertex_buffers(0, self.vertex_buffer.clone())
            .draw_indexed(self.num_vertices, 1, 0, 0, 0)
            .wrap_err("Drawing triangle failed")?;
        Ok(())
    }
}
