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
    format::Format,
    image::{view::ImageView, ImageDimensions, StorageImage},
    pipeline::{
        graphics::{
            input_assembly::InputAssemblyState,
            vertex_input::BuffersDefinition,
            viewport::{Viewport, ViewportState},
        },
        GraphicsPipeline, Pipeline,
    },
    render_pass::{Framebuffer, FramebufferCreateInfo, RenderPass, Subpass},
    swapchain::{acquire_next_image, AcquireError, Swapchain, SwapchainCreateInfo},
    sync::{FlushError, GpuFuture},
};
use winit::{
    event::{DeviceEvent, Event, WindowEvent},
    event_loop::{ControlFlow, EventLoop},
    window::WindowBuilder,
};

use aether::vulkan::context::{Context, Vertex};

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

fn get_pipeline(
    vulkan_ctx: Context,
    viewport: Viewport,
    renderpass: Arc<RenderPass>,
) -> Arc<GraphicsPipeline> {
    let vs = vs::load(vulkan_ctx.device.clone()).expect("failed to create shader module");
    let fs = fs::load(vulkan_ctx.device.clone()).expect("failed to create shader module");

    GraphicsPipeline::start()
        .vertex_input_state(BuffersDefinition::new().vertex::<Vertex>())
        .vertex_shader(vs.entry_point("main").unwrap(), ())
        .input_assembly_state(InputAssemblyState::new())
        .viewport_state(ViewportState::viewport_fixed_scissor_irrelevant([viewport]))
        .fragment_shader(fs.entry_point("main").unwrap(), ())
        .render_pass(Subpass::from(renderpass.clone(), 0).unwrap())
        .build(vulkan_ctx.device.clone())
        .unwrap()
}

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
                window_resized = true
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
                    let (new_swapchain, new_images) = vulkan_ctx
                        .swapchain
                        .recreate(SwapchainCreateInfo {
                            image_extent: dimensions.into(),
                            ..vulkan_ctx.swapchain.create_info()
                        })
                        .unwrap();

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

                    let mut builder = AutoCommandBufferBuilder::primary(
                        vulkan_ctx.device.clone(),
                        vulkan_ctx.graphics.family(),
                        CommandBufferUsage::MultipleSubmit,
                    )
                    .unwrap();

                    let view = ImageView::new_default(vulkan_ctx.images[image_idx].clone()).unwrap();
                    let framebuffer = Framebuffer::new(
                        renderpass.clone(),
                        FramebufferCreateInfo {
                            attachments: vec![view],
                            ..Default::default()
                        },
                    ).unwrap();

                    let pass_begin_info = RenderPassBeginInfo {
                        clear_values: vec![Some([0.0, 0.0, 0.0, 1.0].into())],
                        ..RenderPassBeginInfo::framebuffer(framebuffer.clone())
                    };

                    builder
                        .begin_render_pass(pass_begin_info, SubpassContents::Inline)
                        .unwrap()
                        .bind_pipeline_graphics(pipeline.clone())
                        .bind_vertex_buffers(0, vertex_buffer.clone())
                        .draw(3, 1, 0, 0)
                        .unwrap()
                        .end_render_pass()
                        .unwrap();

                    let cmd = builder.build();

                    let execution = vulkano::sync::now(vulkan_ctx.device.clone())
                        .join(aquire_future)
                        .then_execute(vulkan_ctx.graphics.clone(), cmd.unwrap())
                        .unwrap()
                        .then_swapchain_present(
                            vulkan_ctx.present.clone(),
                            vulkan_ctx.swapchain.clone(),
                            image_idx,
                        )
                        .then_signal_fence_and_flush();

                    match execution {
                        Ok(future) => {
                            future.wait(None).unwrap(); // wait for the GPU to finish
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
