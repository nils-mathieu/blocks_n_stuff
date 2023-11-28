//! This module defines everything that's needed to draw stuff on the screen.

use std::sync::Arc;

use pollster::FutureExt;
use winit::window::Window;

use self::quad_pipeline::QuadPipeline;

mod quad_pipeline;

/// Keeps traks of the state required to draw stuff on the screen.
///
/// This includes the an open connection with the GPU, as well as shaders and other resources.
pub struct Renderer {
    /// An open connection with the selected GPU device.
    device: wgpu::Device,
    /// The queue that's used to submit commands to the GPU.
    queue: wgpu::Queue,
    /// The surface on which the renderer is drawing.
    surface: wgpu::Surface<'static>,
    /// The current configuration of the surface/swapchain.
    surface_config: wgpu::SurfaceConfiguration,

    /// The pipeline responsible for rendering axis-aligned quads in the world.
    quad_pipeline: QuadPipeline,
}

impl Renderer {
    /// Creates a new [`Renderer`] for the provided window.
    pub fn new(window: Arc<Window>) -> Self {
        let surface_size = window.inner_size();
        let instance = wgpu::Instance::new(Default::default());
        let surface = instance
            .create_surface(window)
            .expect("failed to create a surface from the created window");
        let adapter = instance
            .request_adapter(&wgpu::RequestAdapterOptions {
                power_preference: wgpu::PowerPreference::HighPerformance,
                compatible_surface: Some(&surface),
                force_fallback_adapter: false,
            })
            .block_on()
            .expect("failed to find a suitable GPU adapter");
        let (device, queue) = adapter
            .request_device(&Default::default(), None)
            .block_on()
            .expect("failed to establish a connection with the selected GPU device");
        let surface_config = surface
            .get_default_config(&adapter, surface_size.width, surface_size.height)
            .expect("the selected GPU adapter is not compatible with the provided surface");
        surface.configure(&device, &surface_config);

        let quad_pipeline = QuadPipeline::new(&device, surface_config.format);

        Self {
            surface_config,
            surface,
            device,
            queue,
            quad_pipeline,
        }
    }

    /// Notifies the [`Renderer`] that the size of the window on which it is drawing has changed.
    pub fn notify_resized(&mut self, width: u32, height: u32) {
        self.surface_config.width = width;
        self.surface_config.height = height;
        self.surface.configure(&self.device, &self.surface_config);
    }

    /// Acquires the next available image from the underlying swapchain.
    ///
    /// If the image is not ready yet, this function will block until it is.
    pub fn render_next_image(&mut self) {
        let texture = self
            .surface
            .get_current_texture()
            .expect("failed to acquire the next image from the swapchain");
        let output_view = texture.texture.create_view(&Default::default());

        let mut command_encoder = self.device.create_command_encoder(&Default::default());

        {
            let mut rp = command_encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
                color_attachments: &[Some(wgpu::RenderPassColorAttachment {
                    ops: wgpu::Operations {
                        load: wgpu::LoadOp::Clear(wgpu::Color::BLACK),
                        store: wgpu::StoreOp::Store,
                    },
                    resolve_target: None,
                    view: &output_view,
                })],
                ..Default::default()
            });

            self.quad_pipeline.render(&mut rp);
        }

        self.queue.submit(Some(command_encoder.finish()));
        texture.present();
    }
}
