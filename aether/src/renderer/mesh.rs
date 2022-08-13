use std::{fmt::Debug, rc::Rc, sync::Arc};

use eyre::Context;
use serde::Serialize;
use vulkano::buffer::{BufferUsage, CpuAccessibleBuffer};

use crate::fs::amesh::AMesh;
use crate::vulkan::vertex::Vertex;

use crate::renderer::{
    material::{BindMaterial, Material},
    CommandBufferBuilder, Drawable, Renderer,
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
        mesh: &AMesh,
        material: Arc<Material>,
    ) -> eyre::Result<Rc<Self>> {
        let num_vertices = mesh.indices.len() as u32;
        Ok(Rc::new(Self {
            vertex_buffer: CpuAccessibleBuffer::from_iter(
                renderer.ctx.device.clone(),
                BufferUsage::vertex_buffer(),
                false,
                mesh.vertices.clone(),
            )
            .wrap_err("Failed to create vertex buffer for mesh")?,
            index_buffer: CpuAccessibleBuffer::from_iter(
                renderer.ctx.device.clone(),
                BufferUsage::index_buffer(),
                false,
                mesh.indices.clone(),
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
