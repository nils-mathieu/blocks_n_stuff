//! This module defines everything that's needed to draw stuff on the screen.

use std::sync::Arc;

mod helpers;
mod shaders;

mod gpu;
pub use gpu::Gpu;

mod surface;
pub use surface::Surface;

pub mod render_data;

use self::helpers::UniformBuffer;
use self::render_data::{InstantUniforms, RenderData};

/// The renderer is responsible for using the GPU to render things on a render target.
pub struct Renderer {
    /// The GPU that's used to perform the work.
    gpu: Arc<Gpu>,

    /// See [`shaders::quad::create`].
    quad_pipeline: wgpu::RenderPipeline,

    /// The uniforms that are shared across a single frame.
    instant_uniforms: UniformBuffer,
}

impl Renderer {
    /// Creates a new [`Renderer`] instance.
    pub fn new(gpu: Arc<Gpu>, output_format: wgpu::TextureFormat) -> Self {
        let instant_uniforms =
            UniformBuffer::new_for::<InstantUniforms>(&gpu.device, wgpu::ShaderStages::VERTEX);

        Self {
            quad_pipeline: shaders::quad::create(
                &gpu.device,
                instant_uniforms.layout(),
                output_format,
            ),
            instant_uniforms,
            gpu,
        }
    }

    /// Renders a frame to the provided target.
    pub fn render(&self, target: &wgpu::TextureView, render_data: &RenderData) {
        // Write the render data to the uniform buffer.
        self.instant_uniforms
            .write(&self.gpu.queue, &render_data.instant_uniforms);

        // Start recording the commands.
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

            rp.set_bind_group(0, self.instant_uniforms.bind_group(), &[]);
            rp.set_pipeline(&self.quad_pipeline);
            rp.draw(0..4, 0..1);
        }

        self.gpu.queue.submit(Some(encoder.finish()));
    }

    /// A convenience function that renders a frame to the provided surface.
    pub fn render_to_surface(&self, surface: &Surface, render_data: &RenderData) {
        let Some(texture) = surface.acquire_next_image() else {
            return;
        };

        let view = texture
            .texture
            .create_view(&wgpu::TextureViewDescriptor::default());
        self.render(&view, render_data);
        texture.present();
    }
}
