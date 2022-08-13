pub mod material;
pub mod mesh;

use crate::{
    renderer::material::{BindMaterial, Material},
    vulkan::{context, vertex::Vertex},
};
use eyre::Context;
use std::{rc::Rc, sync::Arc, fmt::Debug};
use vulkano::{
    buffer::{BufferUsage, CpuAccessibleBuffer},
    command_buffer::{AutoCommandBufferBuilder, CommandBufferUsage, PrimaryAutoCommandBuffer, RenderPassBeginInfo, SubpassContents},
    render_pass::Framebuffer,
};

pub type CommandBufferBuilder = AutoCommandBufferBuilder<PrimaryAutoCommandBuffer>;

pub trait Drawable : Debug {
    #[doc = "# Errors"]
    #[doc = "Errors if drawing the object fails, either due to vulkan or bad usage"]
    fn draw<'a>(&'a self, cmd: &'a mut CommandBufferBuilder) -> eyre::Result<()>;
}

pub struct Renderer {
    pub ctx: context::Context,
    draw_list: Vec<Rc<dyn Drawable>>,
    framebuffer: Option<Arc<Framebuffer>>,
}

#[derive(Clone, Debug)]
pub struct Triangle {
    buffer: Arc<CpuAccessibleBuffer<[Vertex; 3]>>,
    material: Arc<Material>,
}

impl Triangle {
    #[doc = "# Errors"]
    #[doc = "Errors if vertex buffer creation failes"]
    pub fn new(
        renderer: &Renderer,
        vertices: [Vertex; 3],
        material: Arc<Material>,
    ) -> eyre::Result<Rc<Self>> {
        Ok(Rc::new(Self {
            buffer: CpuAccessibleBuffer::from_data(
                renderer.ctx.device.clone(),
                BufferUsage::vertex_buffer(),
                false,
                vertices,
            )
            .wrap_err("Failed to create vertex buffer for triangle")?,
            material,
        }))
    }
}

impl Drawable for Triangle {
    fn draw<'a>(&'a self, cmd: &'a mut CommandBufferBuilder) -> eyre::Result<()> {
        cmd.bind_material(self.material.clone())
            .bind_vertex_buffers(0, self.buffer.clone())
            .draw(3, 1, 0, 0)
            .wrap_err("Drawing triangle failed")?;
        Ok(())
    }
}

impl Renderer {
    #[doc = "# Errors"]
    #[doc = "Errors if [`aether::vulkan::context::Context::new()`] fails"]
    pub fn new(
        event_loop: &winit::event_loop::EventLoop<()>,
    ) -> eyre::Result<Self> {
        Ok(Self {
            ctx: context::Context::new(event_loop)?,
            draw_list: Vec::new(),
            framebuffer: None
        })
    }

    pub fn new_frame(&mut self, framebuffer: Arc<Framebuffer>) {
        self.draw_list.clear();
        self.framebuffer = Some(framebuffer);
        
    }

    pub fn add(&mut self, drawable: Rc<dyn Drawable>) {
        self.draw_list.push(drawable);
    }

    pub fn end_frame(&mut self) -> CommandBufferBuilder {
        let mut command = AutoCommandBufferBuilder::primary(
            self.ctx.device.clone(),
            self.ctx.graphics.family(),
            CommandBufferUsage::OneTimeSubmit,
        )
        .unwrap();

        let mut cmd: &mut AutoCommandBufferBuilder<PrimaryAutoCommandBuffer> = &mut command;
        let mut begin_info = RenderPassBeginInfo::framebuffer(self.framebuffer.clone().unwrap());
        begin_info.clear_values = vec![Some([0.0, 0.0, 0.0, 1.0].into())];
        cmd.begin_render_pass(begin_info, SubpassContents::Inline).unwrap();

        for drawable in &self.draw_list {
            drawable.draw(cmd).unwrap();
        }

        cmd.end_render_pass().unwrap();

        command
    }
}
