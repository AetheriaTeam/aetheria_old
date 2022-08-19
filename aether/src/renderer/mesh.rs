use eyre::Context;

use crate::types::mesh::Mesh;
use crate::renderer::{
    material::Bind,
    CommandBufferBuilder, Drawable,
};

impl Drawable for Mesh {
    fn draw<'a>(&'a self, cmd: &'a mut CommandBufferBuilder) -> eyre::Result<()> {
        cmd.bind_material(self.material.clone())
            .bind_index_buffer(self.index_buffer.clone())
            .bind_vertex_buffers(0, self.vertex_buffer.clone())
            .draw_indexed(self.num_indices, 1, 0, 0, 0)
            .wrap_err("Drawing triangle failed")?;
        Ok(())
    }
}
