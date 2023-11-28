//! This module defines everything that's needed to draw stuff on the screen.

mod shaders;

mod gpu;
use std::sync::Arc;

pub use gpu::Gpu;

mod surface;
pub use surface::Surface;

/// The renderer is responsible for using the GPU to render things on a render target.
pub struct Renderer {
    /// The GPU that's used to perform the work.
    gpu: Arc<Gpu>,

    /// See [`shaders::quad::create`].
    quad_pipeline: wgpu::RenderPipeline,
}

impl Renderer {
    /// Creates a new [`Renderer`] instance.
    pub fn new(gpu: Arc<Gpu>, output_format: wgpu::TextureFormat) -> Self {
        Self {
            quad_pipeline: shaders::quad::create(&gpu.device, output_format),

            gpu,
        }
    }

    /// Renders a frame to the provided target.
    pub fn render(&self, target: &wgpu::TextureView) {
        let mut encoder = self.gpu.device.create_command_encoder(&Default::default());

        {
            let mut rp = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
                color_attachments: &[
                    // The main output attachment.
                    Some(wgpu::RenderPassColorAttachment {
                        ops: wgpu::Operations {
                            load: wgpu::LoadOp::Clear(wgpu::Color::BLACK),
                            store: wgpu::StoreOp::Store,
                        },
                        resolve_target: None,
                        view: target,
                    }),
                ],
                ..Default::default()
            });

            rp.set_pipeline(&self.quad_pipeline);
            rp.draw(0..4, 0..1);
        }

        self.gpu.queue.submit(Some(encoder.finish()));
    }

    /// A convenience function that renders a frame to the provided surface.
    pub fn render_to_surface(&self, surface: &Surface) {
        let Some(texture) = surface.acquire_next_image() else {
            return;
        };

        let view = texture
            .texture
            .create_view(&wgpu::TextureViewDescriptor::default());
        self.render(&view);
        texture.present();
    }
}
