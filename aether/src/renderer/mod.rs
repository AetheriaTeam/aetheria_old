pub mod material;
pub mod mesh;

use crate::{
    renderer::material::{BindMaterial, Material},
    types::mesh::Mesh,
    vulkan::{context, vertex::Vertex},
};
use eyre::Context;
use std::{cell::RefCell, fmt::Debug, rc::Rc, sync::Arc};
use vulkano::{
    buffer::{BufferUsage, CpuAccessibleBuffer},
    command_buffer::{
        AutoCommandBufferBuilder, CommandBufferUsage, PrimaryAutoCommandBuffer,
        RenderPassBeginInfo, SubpassContents,
    },
    pipeline::{
        graphics::{
            input_assembly::InputAssemblyState,
            vertex_input::BuffersDefinition,
            viewport::{Viewport, ViewportState},
        },
        GraphicsPipeline,
    },
    render_pass::{Framebuffer, RenderPass, Subpass},
};
use winit::event_loop;

pub type CommandBufferBuilder = AutoCommandBufferBuilder<PrimaryAutoCommandBuffer>;

pub trait Drawable {
    #[doc = "# Errors"]
    #[doc = "Errors if drawing the object fails, either due to vulkan or bad usage"]
    fn draw<'a>(&'a self, cmd: &'a mut CommandBufferBuilder) -> eyre::Result<()>;
}

pub struct Renderer {
    pub ctx: context::Context,
    draw_list: Vec<Box<dyn Drawable>>,
    framebuffer: Option<Arc<Framebuffer>>,
    pub render_pass: Arc<RenderPass>,
    pub render_pipeline: Arc<GraphicsPipeline>,
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
    pub fn new(event_loop: &winit::event_loop::EventLoop<()>) -> eyre::Result<Self> {
        mod render_vs {
            vulkano_shaders::shader! {
                ty: "vertex",
                path: "../arbiter/assets/shaders/render.vert"
            }
        }

        mod render_fs {
            vulkano_shaders::shader! {
                ty: "fragment",
                path: "../arbiter/assets/shaders/render.frag"
            }
        }

        let ctx = context::Context::new(event_loop)?;

        let render_pass = vulkano::single_pass_renderpass!(ctx.device.clone(),
            attachments: {
                color: {
                    load: Clear,
                    store: Store,
                    format: ctx.swapchain.image_format(),
                    samples: 1,
                }
            },
            pass: {
                color: [color],
                depth_stencil: {}
            }
        )?;

        let vs = match render_vs::load(ctx.device.clone()) {
            Ok(shader) => shader,
            Err(e) => panic!("Failed to load vertex shader due to {}", e),
        };
        let fs = match render_fs::load(ctx.device.clone()) {
            Ok(shader) => shader,
            Err(e) => panic!("Failed to load fragment shader due to {}", e),
        };

        let render_pipeline = match GraphicsPipeline::start()
            .vertex_input_state(BuffersDefinition::new().vertex::<Vertex>())
            .vertex_shader(
                vs.entry_point("main")
                    .expect("Failed to get entry point of vertex shader"),
                (),
            )
            .input_assembly_state(InputAssemblyState::new())
            .viewport_state(ViewportState::viewport_dynamic_scissor_irrelevant())
            .fragment_shader(
                fs.entry_point("main")
                    .expect("Failed to get entry point of fragment shader"),
                (),
            )
            .render_pass(
                Subpass::from(render_pass.clone(), 0).expect("Failed to create subpass info"),
            )
            .build(ctx.device.clone())
        {
            Ok(pipeline) => pipeline,
            Err(e) => panic!("Failed to create pipeline because {}", e),
        };

        Ok(Self {
            ctx,
            draw_list: Vec::new(),
            framebuffer: None,
            render_pass,
            render_pipeline,
        })
    }

    pub fn new_frame(&mut self, framebuffer: Arc<Framebuffer>) {
        self.draw_list.clear();
        self.framebuffer = Some(framebuffer);
    }

    pub fn add(&mut self, drawable: Box<dyn Drawable>) {
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

        let viewport = Viewport {
            origin: [0.0, 0.0],
            dimensions: self.ctx.surface.window().inner_size().into(),
            depth_range: 0.0..1.0,
        };

        cmd.begin_render_pass(begin_info, SubpassContents::Inline)
            .unwrap()
            .bind_pipeline_graphics(self.render_pipeline.clone())
            .set_viewport(0, [viewport]);

        for drawable in &self.draw_list {
            drawable.draw(cmd).unwrap();
        }

        cmd.end_render_pass().unwrap();

        command
    }
}
