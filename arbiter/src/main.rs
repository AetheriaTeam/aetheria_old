#![warn(clippy::pedantic)]
#![warn(clippy::nursery)]
#![warn(clippy::unwrap_used)]
#![warn(clippy::expect_used)]

use winit::{
    event::{DeviceEvent, Event, WindowEvent},
    event_loop::EventLoop,
    window::WindowBuilder,
};

use std::path::Path;

fn main() {
    let event_loop = EventLoop::new();
    let window = match WindowBuilder::new().build(&event_loop) {
        Ok(value) => value,
        Err(e) => panic!("Failed to create window because {}", e)
    };

    let vulkan_ctx = match aether::vulkan::Context::new(&window) {
        Ok(value) => value,
        Err(e) => panic!("Failed to create vulkan context because {}", e)
    };

    let vert_shader = match aether::vulkan::Shader::new(&vulkan_ctx, Path::new("assets/shaders/triangle.vert")) {
        Ok(value) => value,
        Err(e) => panic!("Failed to load vertex shader because {}", e)
    };

    let _frag_shader = aether::vulkan::Shader::new(&vulkan_ctx, Path::new("assets/shaders/triangle.frag"));
    let _vert_stage = vert_shader.get_stage();

    event_loop.run(move |event, _, control_flow| {
        control_flow.set_poll();

        match event {
            Event::WindowEvent {
                event: WindowEvent::CloseRequested,
                ..
            } => control_flow.set_exit(),
            Event::DeviceEvent {
                event: DeviceEvent::Key(key),
                ..
            } => {
                if key.state == winit::event::ElementState::Pressed && key.virtual_keycode == Some(winit::event::VirtualKeyCode::Escape) {
                    control_flow.set_exit();
                }
            }
            _ => (),
        };
    });
}
