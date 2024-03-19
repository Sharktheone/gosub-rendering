#![allow(unused_variables)]

use std::sync::Arc;

use vello::{self, util::RenderSurface};
use winit::{event_loop::{ControlFlow, EventLoop}, keyboard::ModifiersState, window::Window};
use winit::event::{Event, WindowEvent};

struct RenderState<'a> {
    surface: RenderSurface<'a>,
    window: Arc<Window>

}


fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("Hello, world!");

    let event_loop = EventLoop::new()?;

    
    #[allow(unused_mut)]
    let mut renderer = vello::util::RenderContext::new().unwrap();
    let _proxy = event_loop.create_proxy();

    let mut modifiers = ModifiersState::default();

    let mut render_state = None::<RenderState>;


    event_loop.run(move |event, control_flow| {

        match event {
            Event::WindowEvent { ref  event, window_id } => {


                let Some(render_state) = &mut render_state else {
                    return;
                };
                if render_state.window.id() != window_id {
                    return;
                }

                match event {

                    WindowEvent::CloseRequested => {
                        event_loop.exit();
                    }

                    WindowEvent::ModifiersChanged(m) => {
                        modifiers = m.state()
                    }

                    WindowEvent::Resized(size) => {
                        renderer.resize_surface(&mut render_state.surface, size.width, size.height);
                        render_state.window.request_redraw();
                    }

                    WindowEvent::RedrawRequested => {
                        let width = render_state.surface.config.width; 
                        let height = render_state.surface.config.height;
                        let device = &renderer.devices[render_state.surface.dev_id];

                        todo!()
                    }
                    _ => (),
                }
            },
            Event::Suspended => {
                println!("Suspended");
                event_loop.set_control_flow(ControlFlow::Wait);
            },
            Event::Resumed => {
                println!("Resumed");
                event_loop.set_control_flow(ControlFlow::Poll);
            },
            _ => (),
        }
    });

    Ok(())
}
