#![warn(clippy::pedantic)]
#![warn(clippy::nursery)]
#![warn(clippy::unwrap_used)]
#![warn(clippy::expect_used)]

use std::sync::Arc;
use vulkano::{
    buffer::{BufferUsage, CpuAccessibleBuffer},
    command_buffer::{
        AutoCommandBufferBuilder, CommandBufferUsage, RenderPassBeginInfo, SubpassContents,
    },
    image::view::ImageView,
    pipeline::{
        graphics::{
            input_assembly::InputAssemblyState,
            vertex_input::BuffersDefinition,
            viewport::{Viewport, ViewportState},
        },
        GraphicsPipeline,
    },
    render_pass::{Framebuffer, FramebufferCreateInfo, RenderPass, Subpass},
    swapchain::{AcquireError, SwapchainCreateInfo},
    sync::{FlushError, GpuFuture},
};
use winit::{
    event::{DeviceEvent, Event, WindowEvent},
    event_loop::{ControlFlow, EventLoop},
};

use aether::vulkan::{context::Context, vertex::Vertex};

#[allow(clippy::needless_question_mark)]
mod vs {
    vulkano_shaders::shader! {
        ty: "vertex",
        src: "
#version 450

layout(location = 0) in vec2 position;

void main() {
    gl_Position = vec4(position, 0.0, 1.0);
}"
    }
}

#[allow(clippy::needless_question_mark)]
mod fs {
    vulkano_shaders::shader! {
        ty: "fragment",
        src: "
#version 450

layout(location = 0) out vec4 f_color;

void main() {
    f_color = vec4(1.0, 0.0, 0.0, 1.0);
}"
    }
}

// Temp code, allowing expect
#[allow(clippy::expect_used)]
fn get_pipeline(
    vulkan_ctx: Context,
    viewport: Viewport,
    renderpass: Arc<RenderPass>,
) -> Arc<GraphicsPipeline> {
    let vs = match vs::load(vulkan_ctx.device.clone()) {
        Ok(shader) => shader,
        Err(e) => panic!("Failed to load vertex shader due to {}", e)
    };
    let fs = match fs::load(vulkan_ctx.device.clone()) {
        Ok(shader) => shader,
        Err(e) => panic!("Failed to load fragment shader due to {}", e)
    };

    match GraphicsPipeline::start()
        .vertex_input_state(BuffersDefinition::new().vertex::<Vertex>())
        .vertex_shader(vs.entry_point("main").expect("Failed to get entry point of vertex shader"), ())
        .input_assembly_state(InputAssemblyState::new())
        .viewport_state(ViewportState::viewport_fixed_scissor_irrelevant([viewport]))
        .fragment_shader(fs.entry_point("main").expect("Failed to get entry point of fragment shader"), ())
        .render_pass(Subpass::from(renderpass, 0).expect("Failed to create subpass info"))
        .build(vulkan_ctx.device) {
            Ok(pipeline) => pipeline,
            Err(e) => panic!("Failed to create pipeline because {}", e)
        }
}

// Temporary code, allowing too many lines
#[allow(clippy::too_many_lines)]
fn main() {
    let event_loop = EventLoop::new();

    let mut vulkan_ctx = match Context::new(&event_loop) {
        Ok(value) => value,
        Err(e) => panic!("Failed to create vulkan context because {}", e),
    };

    let vertex1 = Vertex {
        position: [-0.5, -0.5],
    };
    let vertex2 = Vertex {
        position: [0.0, 0.5],
    };
    let vertex3 = Vertex {
        position: [0.5, -0.25],
    };

    let vertex_buffer = match CpuAccessibleBuffer::from_iter(
        vulkan_ctx.device.clone(),
        BufferUsage::vertex_buffer(),
        false,
        vec![vertex1, vertex2, vertex3],
    ) {
        Err(e) => panic!("Failed to create vertex buffer because {}", e),
        Ok(buffer) => buffer,
    };

    let renderpass = vulkano::single_pass_renderpass!(vulkan_ctx.device.clone(),
        attachments: {
            color: {
                load: Clear,
                store: Store,
                format: vulkan_ctx.swapchain.image_format(),
                samples: 1,
            }
        },
        pass: {
            color: [color],
            depth_stencil: {}
        }
    )
    .unwrap();

    let mut viewport = Viewport {
        origin: [0.0, 0.0],
        dimensions: vulkan_ctx.surface.window().inner_size().into(),
        depth_range: 0.0..1.0,
    };
    let mut pipeline = get_pipeline(vulkan_ctx.clone(), viewport.clone(), renderpass.clone());

    let mut recreate_swapchain = false;
    let mut window_resized = false;

    event_loop.run(move |event, _, control_flow| {
        *control_flow = ControlFlow::Poll;

        match event {
            Event::WindowEvent {
                event: WindowEvent::CloseRequested,
                ..
            } => *control_flow = ControlFlow::Exit,
            Event::WindowEvent {
                event: WindowEvent::Resized(_),
                ..
            } => {
                recreate_swapchain = true;
                window_resized = true;
            }
            Event::DeviceEvent {
                event: DeviceEvent::Key(key),
                ..
            } => {
                if key.state == winit::event::ElementState::Pressed
                    && key.virtual_keycode == Some(winit::event::VirtualKeyCode::Escape)
                {
                    *control_flow = ControlFlow::Exit;
                }
            }
            Event::MainEventsCleared => {
                if recreate_swapchain {
                    recreate_swapchain = false;
                    let dimensions = vulkan_ctx.surface.window().inner_size();
                    let (new_swapchain, new_images) = match vulkan_ctx
                        .swapchain
                        .recreate(SwapchainCreateInfo {
                            image_extent: dimensions.into(),
                            ..vulkan_ctx.swapchain.create_info()
                        }) {
                            Ok(value) => value,
                            Err(e) => panic!("Failed to recreate swapchain due to {}", e)
                        };

                    vulkan_ctx.swapchain = new_swapchain;
                    vulkan_ctx.images = new_images;

                    if window_resized {
                        window_resized = false;
                        viewport.dimensions = vulkan_ctx.surface.window().inner_size().into();
                        pipeline =
                            get_pipeline(vulkan_ctx.clone(), viewport.clone(), renderpass.clone());
                    }

                    let (image_idx, suboptimal, aquire_future) =
                        match vulkano::swapchain::acquire_next_image(
                            vulkan_ctx.swapchain.clone(),
                            None,
                        ) {
                            Ok(r) => r,
                            Err(AcquireError::OutOfDate) => {
                                recreate_swapchain = true;
                                return;
                            }
                            Err(e) => panic!("Failed to aquire swapchain image due to {}", e),
                        };

                    if suboptimal {
                        recreate_swapchain = true;
                    }

                    let mut builder = match AutoCommandBufferBuilder::primary(
                        vulkan_ctx.device.clone(),
                        vulkan_ctx.graphics.family(),
                        CommandBufferUsage::MultipleSubmit,
                    ) {
                        Ok(builder) => builder,
                        Err(e) => panic!("Failed to create command buffer builder because {}", e)
                    };

                    let view = match 
                        ImageView::new_default(vulkan_ctx.images[image_idx].clone()) {
                            Ok(view) => view,
                            Err(e) => panic!("Failed to create swapchain image view because {}", e)
                        };
                    let framebuffer = match Framebuffer::new(
                        renderpass.clone(),
                        FramebufferCreateInfo {
                            attachments: vec![view],
                            ..Default::default()
                        },
                    ) {
                        Ok(framebuffer) => framebuffer,
                        Err(e) => panic!("Failed to create framebuffer due to {}", e)
                    };

                    let pass_begin_info = RenderPassBeginInfo {
                        clear_values: vec![Some([0.0, 0.0, 0.0, 1.0].into())],
                        ..RenderPassBeginInfo::framebuffer(framebuffer)
                    };

                    #[allow(clippy::expect_used)]
                    builder
                        .begin_render_pass(pass_begin_info, SubpassContents::Inline)
                        .expect("Failed to begin render pass")
                        .bind_pipeline_graphics(pipeline.clone())
                        .bind_vertex_buffers(0, vertex_buffer.clone())
                        .draw(3, 1, 0, 0)
                        .expect("Failed to draw")
                        .end_render_pass()
                        .expect("Failed to end render pass");

                    let cmd = match builder.build() {
                        Ok(cmd) => cmd,
                        Err(e) => panic!("Failed to build command buffer because {}", e)
                    };

                    #[allow(clippy::expect_used)]
                    let execution = vulkano::sync::now(vulkan_ctx.device.clone())
                        .join(aquire_future)
                        .then_execute(vulkan_ctx.graphics.clone(), cmd)
                        .expect("Executing draw command buffer failed")
                        .then_swapchain_present(
                            vulkan_ctx.present.clone(),
                            vulkan_ctx.swapchain.clone(),
                            image_idx,
                        )
                        .then_signal_fence_and_flush();

                    match execution {
                        Ok(future) => {
                            if let Err(e) = future.wait(None) {
                                panic!("Error waiting for command buffer future because {}", e);
                            }
                        }
                        Err(FlushError::OutOfDate) => {
                            recreate_swapchain = true;
                        }
                        Err(e) => {
                            println!("Failed to flush future: {:?}", e);
                        }
                    }
                }
            }
            _ => (),
        };
    });
}
