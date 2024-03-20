#![allow(unused_variables)]

use std::{num::NonZeroUsize, sync::Arc};

use vello::{
    self, AaConfig, AaSupport, Renderer, RendererOptions, RenderParams, Scene, util::RenderSurface,
};
use vello::kurbo::{Affine, Circle, Rect, Stroke};
use vello::peniko::Color;
use winit::{
    event_loop::{ControlFlow, EventLoop},
    window::{Window, WindowBuilder},
};
use winit::dpi::LogicalSize;
use winit::event::{Event, WindowEvent};

struct RenderState<'a> {
    surface: RenderSurface<'a>,
    window: Arc<Window>,
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("Hello, world!");

    let event_loop = EventLoop::new()?;

    #[allow(unused_mut)]
        let mut renderer_cx = vello::util::RenderContext::new().unwrap();

    let mut renderers: Vec<Option<Renderer>> = Vec::new();

    let mut render_state = None::<RenderState>;

    let mut cached_window = None::<Arc<Window>>;

    let mut scene = Scene::new();

    event_loop.run(move |event, event_loop| match event {
        Event::WindowEvent {
            ref event,
            window_id,
        } => {
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

                WindowEvent::Resized(size) => {
                    renderer_cx.resize_surface(&mut render_state.surface, size.width, size.height);

                    render_state.window.request_redraw();
                }

                WindowEvent::RedrawRequested => {
                    scene.reset();
                    add_shapes_to_scene(&mut scene);

                    let surface = &render_state.surface;

                    let width = surface.config.width;
                    let height = surface.config.height;

                    let device = &renderer_cx.devices[surface.dev_id];

                    let surface_texture = surface
                        .surface
                        .get_current_texture()
                        .expect("failed to get surface texture");

                    let params = RenderParams {
                        base_color: Color::BLACK,
                        width,
                        height,
                        antialiasing_method: AaConfig::Msaa16,
                    };

                    renderers[surface.dev_id]
                        .as_mut()
                        .expect("failed to get renderer")
                        .render_to_surface(
                            &device.device,
                            &device.queue,
                            &scene,
                            &surface_texture,
                            &params,
                        )
                        .expect("failed to render to surface");

                    surface_texture.present();

                    device.device.poll(wgpu::Maintain::Poll);
                }
                _ => (),
            }
        }
        Event::Suspended => {
            if let Some(render_state) = &render_state {
                cached_window = Some(render_state.window.clone());
            }
            event_loop.set_control_flow(ControlFlow::Wait);
        }
        Event::Resumed => {
            let None = render_state else {
                return;
            };
            let window = cached_window
                .take()
                .unwrap_or_else(|| create_window(event_loop).expect("Failed to create window"));
            let size = window.inner_size();

            let surface = renderer_cx.create_surface(
                window.clone(),
                size.width,
                size.height,
                wgpu::PresentMode::AutoVsync,
            );
            let surface = futures::executor::block_on(surface).expect("Failed to create surface");

            render_state = {
                let render_state = RenderState { window, surface };

                renderers.resize_with(renderer_cx.devices.len(), || None);
                let id = render_state.surface.dev_id;
                renderers[id].get_or_insert_with(|| {
                    Renderer::new(
                        &renderer_cx.devices[id].device,
                        RendererOptions {
                            surface_format: Some(render_state.surface.format),
                            use_cpu: false,
                            antialiasing_support: AaSupport::all(),
                            num_init_threads: NonZeroUsize::new(1),
                        },
                    )
                        .expect("Failed to create renderer")
                });

                Some(render_state)
            };

            event_loop.set_control_flow(ControlFlow::Poll);
        }
        Event::AboutToWait => {
            if let Some(render_state) = &render_state {
                render_state.window.request_redraw();
            }
        }
        _ => (),
    })?;

    Ok(())
}

fn create_window(
    event_loop: &winit::event_loop::EventLoopWindowTarget<()>,
) -> Result<Arc<Window>, Box<dyn std::error::Error>> {
    Ok(Arc::new(
        WindowBuilder::new()
            .with_inner_size(LogicalSize::new(800, 600))
            .with_title("Render Demo")
            .build(event_loop)?,
    ))
}

fn add_shapes_to_scene(scene: &mut Scene) {
    let stroke = Stroke::new(1.0);
    let rect = Rect::new(10.0, 10.0, 100.0, 100.0);

    let color = Color::rgb(0.5, 0.5, 1.0);
    scene.stroke(&stroke, Affine::IDENTITY, color, None, &rect);

    let stroke = Stroke::new(10.0);
    let circle = Circle::new((200.0, 200.0), 128.0);
    let color = Color::rgb(1.0, 0.5, 0.5);
    scene.stroke(&stroke, Affine::IDENTITY, color, None, &circle);
}
