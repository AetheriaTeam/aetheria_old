use winit::{
    event::{
        Event,
        WindowEvent,
        DeviceEvent
    },
    window::WindowBuilder,
    event_loop::EventLoop
};

fn main() {
    let event_loop = EventLoop::new();
    let window = WindowBuilder::new().build(&event_loop).unwrap();

    let vulkan_ctx = aether::vulkan::Context::new();

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
                if key.state == winit::event::ElementState::Pressed {
                    if key.virtual_keycode == Some(winit::event::VirtualKeyCode::Escape) {
                        control_flow.set_exit();
                    }
                }
            },
            _ => ()
        };
    }); 
}
