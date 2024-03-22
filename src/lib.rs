pub mod text;
pub mod image;

use std::num::NonZeroUsize;
use std::sync::Arc;
use vello::{AaConfig, AaSupport, Renderer, RendererOptions, RenderParams, Scene};
use vello::peniko::Color;
use vello::util::{RenderContext, RenderSurface};
use winit::dpi::LogicalSize;
use winit::event::{Event, WindowEvent};
use winit::event_loop::EventLoop;
use winit::window::{Window, WindowBuilder};

use rust_fontconfig::{FcFontCache};
use once_cell::sync::Lazy;

static FONT_CACHE: Lazy<FcFontCache> = Lazy::new(FcFontCache::build);

pub enum RenderState<'a> {
    Active {
        surface: RenderSurface<'a>,
        window: Arc<Window>,
    },
    Suspended(Arc<Window>),
}

pub struct WindowState<'a, FN: FnMut(&mut Scene, (usize, usize))> {
    event_loop: EventLoop<()>,
    render_state: RenderState<'a>,
    render_scene: &'a mut FN,
    cx: RenderContext,
    renderers: Vec<Option<Renderer>>,
    scene: Scene,
}


impl<'a, FN: FnMut(&mut Scene, (usize, usize))> WindowState<'a, FN> {
    pub fn new(render_scene: &'a mut FN) -> anyhow::Result<Self> {
        let event_loop = EventLoop::new()?;
        let render_state = RenderState::Suspended(create_window(&event_loop)?);
        let cx = RenderContext::new().map_err(|e| anyhow::anyhow!(e.to_string()))?;
        

        Ok(Self {
            event_loop,
            render_state,
            render_scene,
            cx,
            scene: Scene::new(),
            renderers: Vec::new(),
        })
    }


    pub fn start(mut self) -> anyhow::Result<()> {
        self.event_loop.run(move |event, event_loop| {
            match event {
                Event::Resumed => {
                    let RenderState::Suspended(window) = &self.render_state else {
                        return;
                    };
                    
                    let size = window.inner_size();
                    
                    let surface_future = self.cx.create_surface(
                        window.clone(),
                        size.width, 
                        size.height,
                        wgpu::PresentMode::AutoVsync,
                    );
                    
                    let surface = futures::executor::block_on(surface_future).expect("Error creating surface");
                    

                    
                    self.renderers.resize_with(self.cx.devices.len(), || None);
                    let id = surface.dev_id;
                    self.renderers[id].get_or_insert_with(|| {
                        Renderer::new(
                            &self.cx.devices[id].device,
                            RendererOptions {
                                surface_format: Some(surface.format),
                                use_cpu: false,
                                antialiasing_support: AaSupport::all(),
                                num_init_threads: NonZeroUsize::new(4),
                            },
                        )
                        .expect("Failed to create renderer")
                    });


                    self.render_state = RenderState::Active {
                        surface,
                        window: window.clone(),
                    };
                    
                    event_loop.set_control_flow(winit::event_loop::ControlFlow::Poll);
                    
                }
                
                Event::Suspended => {
                    if let RenderState::Active {window, .. } = &self.render_state {
                        self.render_state = RenderState::Suspended(window.clone());
                    }
                    event_loop.set_control_flow(winit::event_loop::ControlFlow::Wait);
                }
                
                Event::AboutToWait => {
                    if let RenderState::Active {window, .. } = &self.render_state {
                        window.request_redraw();
                    }
                }
                
                Event::WindowEvent { ref event, window_id } => {
                    let  RenderState::Active { window, surface} = &mut self.render_state else {
                        return;
                    };
                    
                    if window.id() != window_id {
                        return;
                    }
                    
                    match event {
                        WindowEvent::CloseRequested => {
                            event_loop.exit();
                        }
                        WindowEvent::Resized(size) => {
                            self.cx.resize_surface(surface, size.width, size.height);
                            window.request_redraw();
                        }
                        
                        WindowEvent::RedrawRequested => {
                            self.scene.reset();
                            let size = window.inner_size();

                            (self.render_scene)(&mut self.scene, (size.width as usize, size.height as usize));
                            
                            let width = surface.config.width;
                            let height = surface.config.height;
                            
                            let surface_texture = surface
                                .surface
                                .get_current_texture()
                                .expect("Failed to get surface texture");
                            
                            let device = &self.cx.devices[surface.dev_id];
                            
                            self.renderers[surface.dev_id]
                                .as_mut()
                                .expect("Failed to get renderer")
                                .render_to_surface(
                                    &device.device,
                                    &device.queue,
                                    &self.scene,
                                    &surface_texture,
                                    &RenderParams {
                                        base_color: Color::BLACK,
                                        width,
                                        height,
                                        antialiasing_method: AaConfig::Msaa16,
                                    },
                                )
                                .expect("Failed to render to surface");
                            
                            surface_texture.present();
                            
                            device.device.poll(wgpu::Maintain::Poll);
                        }
                        

                        _ => {}
                    }
                }
                _ => {}
            }
        })?;
        
        
        Ok(())
    }
}



fn create_window(
    event_loop: &winit::event_loop::EventLoopWindowTarget<()>,
) -> anyhow::Result<Arc<Window>> {
    Ok(Arc::new(
        WindowBuilder::new()
            .with_inner_size(LogicalSize::new(1920, 1080))
            .with_title("Render Demo")
            .build(event_loop)?,
    ))
}
