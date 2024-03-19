#![allow(unused_variables)]

use std::{num::NonZeroUsize, sync::Arc};

use vello::{self, util::RenderSurface, AaConfig, AaSupport, Renderer, RendererOptions, Scene};
use winit::{event_loop::{ControlFlow, EventLoop}, keyboard::ModifiersState, window::{Window, WindowBuilder}};
use winit::event::{Event, WindowEvent};
use winit::dpi::LogicalSize;

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
    let mut renderers = Vec::new();


    let mut render_state = None::<RenderState>;

    let mut cached_window = None::<Arc<Window>>;

    let mut scene = Scene::new();


    event_loop.run(move |event, event_loop| {
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

                let None = render_state else { return; };
                let window = cached_window.take().unwrap_or_else(||
                    create_window(&event_loop).expect("Failed to create window")
                );
                let size = window.inner_size();

                let surface = renderer.create_surface(&window, size.width, size.height, wgpu::PresentMode::AutoVsync);
                let surface = futures::executor::block_on(surface).expect("Failed to create surface");

                let render_state = {
                    let render_state = RenderState {
                        window,
                        surface,
                    };

                    renderers.resize_with(renderer.devices.len(), || None);
                    let id = render_state.surface.dev_id;
                    renderers[id].get_or_insert(|| {
                        let mut renderer = Renderer::new(
                            &renderer.devices[id].device,
                            RendererOptions {
                                surface_format: Some(render_state.surface.format),
                                use_cpu: false,
                                antialiasing_support: AaSupport::all(),
                                num_init_threads: NonZeroUsize::new(4)
                            }
                            );
                    })
                };




                println!("Resumed");
                event_loop.set_control_flow(ControlFlow::Poll);
            },
            _ => (),
        }
    })?;

    Ok(())
}

fn create_window(event_loop: &winit::event_loop::EventLoopWindowTarget<()>) -> Result<Arc<Window>, Box<dyn std::error::Error>> {
    Ok(Arc::new(
            WindowBuilder::new()
            .with_inner_size(LogicalSize::new(800, 600))
            .with_title("Render Demo")
            .build(&event_loop)?
            ))

}

