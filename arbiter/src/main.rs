use winit::{
    event::{DeviceEvent, Event, WindowEvent},
    event_loop::EventLoop,
    window::WindowBuilder,
};

fn main() {
    let event_loop = EventLoop::new();
    let window = WindowBuilder::new().build(&event_loop).unwrap();

    let _vulkan_ctx = aether::vulkan::Context::new(&window);

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
