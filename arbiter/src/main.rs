#![warn(clippy::pedantic)]
#![warn(clippy::nursery)]
#![warn(clippy::unwrap_used)]
#![warn(clippy::expect_used)]

use std::path::Path;
use vulkano::{
    image::view::ImageView,
    render_pass::{Framebuffer, FramebufferCreateInfo},
    swapchain::{AcquireError, SwapchainCreateInfo},
    sync::{FlushError, GpuFuture},
};
use winit::{
    event::{DeviceEvent, Event, WindowEvent},
    event_loop::{ControlFlow, EventLoop},
};

use aether::{
    ecs::Component,
    fs::{aproject::AProject, gltf::Gltf},
    renderer::{material::Material, Renderer},
    types::mesh::{Mesh, MeshData},
};

// Temporary code, allowing too many lines
#[allow(clippy::too_many_lines)]
fn main() {
    let event_loop = EventLoop::new();

    let mut renderer = match Renderer::new(&event_loop) {
        Ok(value) => value,
        Err(e) => panic!("Failed to create renderer because {}", e),
    };

    let mut project =
        AProject::new(Path::new("./test.aproject"), String::from("Test Project")).unwrap();
    let entity_id = project.world.new_entity(None);
    entity_id.execute(&mut project.world, |entity| {
        entity.add_component(Component::Tag(String::from("Test entity")));
        entity.add_component(Component::Position { x: 42.0, y: 31.0 });
    });
    project.save(Path::new("./test.aproject")).unwrap();

    let gltf = Gltf::load(Path::new(
        "./assets/models/gltf-sample-models/2.0/AntiqueCamera/glTF-Binary/AntiqueCamera.glb",
    ))
    .unwrap();
    let mesh_datas: Vec<MeshData> = gltf.to_meshes();

    let material = Material::new();

    let meshes: Vec<Mesh> = mesh_datas
        .iter()
        .map(|data| Mesh::from_data(&renderer.ctx, data, material.clone()).unwrap())
        .collect();

    let mut recreate_swapchain = false;

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
                    let dimensions = renderer.ctx.surface.window().inner_size();
                    let (new_swapchain, new_images) =
                        match renderer.ctx.swapchain.recreate(SwapchainCreateInfo {
                            image_extent: dimensions.into(),
                            ..renderer.ctx.swapchain.create_info()
                        }) {
                            Ok(value) => value,
                            Err(e) => panic!("Failed to recreate swapchain due to {}", e),
                        };

                    renderer.ctx.swapchain = new_swapchain;
                    renderer.ctx.images = new_images;

                    let (image_idx, suboptimal, aquire_future) =
                        match vulkano::swapchain::acquire_next_image(
                            renderer.ctx.swapchain.clone(),
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

                    let view = match ImageView::new_default(renderer.ctx.images[image_idx].clone())
                    {
                        Ok(view) => view,
                        Err(e) => panic!("Failed to create swapchain image view because {}", e),
                    };
                    let framebuffer = match Framebuffer::new(
                        renderer.render_pass.clone(),
                        FramebufferCreateInfo {
                            attachments: vec![view],
                            ..Default::default()
                        },
                    ) {
                        Ok(framebuffer) => framebuffer,
                        Err(e) => panic!("Failed to create framebuffer due to {}", e),
                    };

                    renderer.new_frame(framebuffer);
                    meshes
                        .iter()
                        .for_each(|mesh| renderer.add(Box::new(mesh.clone())));
                    let command = renderer.end_frame();

                    let cmd = match command.build() {
                        Ok(cmd) => cmd,
                        Err(e) => panic!("Failed to build command buffer because {:?}", e),
                    };

                    #[allow(clippy::expect_used)]
                    let execution = vulkano::sync::now(renderer.ctx.device.clone())
                        .join(aquire_future)
                        .then_execute(renderer.ctx.graphics.clone(), cmd)
                        .expect("Executing draw command buffer failed")
                        .then_swapchain_present(
                            renderer.ctx.present.clone(),
                            renderer.ctx.swapchain.clone(),
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
