//! This module defines everything that's needed to draw stuff on the screen.

use std::sync::Arc;

mod helpers;
mod shaders;

mod gpu;
pub use gpu::Gpu;

mod surface;
pub use surface::Surface;

pub mod render_data;

use self::helpers::{UniformBuffer, UniformBufferLayout, VertexBuffer};
use self::render_data::{FrameUniforms, RenderData};

/// The renderer is responsible for using the GPU to render things on a render target.
pub struct Renderer {
    /// The GPU that's used to perform the work.
    gpu: Arc<Gpu>,

    /// See [`shaders::quad::create`].
    quad_pipeline: wgpu::RenderPipeline,

    /// The uniform buffer layout that's used to describe the uniform buffers that are supposed
    /// to change on every single frame.
    frame_uniform_layout: UniformBufferLayout<FrameUniforms>,
}

impl Renderer {
    /// Creates a new [`Renderer`] instance.
    pub fn new(gpu: Arc<Gpu>, output_format: wgpu::TextureFormat) -> Self {
        let frame_uniform_layout =
            UniformBufferLayout::new(gpu.clone(), wgpu::ShaderStages::VERTEX);

        Self {
            quad_pipeline: shaders::quad::create(&gpu.device, &frame_uniform_layout, output_format),

            frame_uniform_layout,

            gpu,
        }
    }

    /// Creates a new [`UniformBuffer`] that follows the layout of the frame uniform layout.
    pub fn create_frame_uniform_buffer(&self) -> UniformBuffer<FrameUniforms> {
        self.frame_uniform_layout.instanciate(&self.gpu.device)
    }

    /// Creates a new [`VertexBuffer`] that can store instances of `T`.
    pub fn create_vertex_buffer<T>(&self, capacity: wgpu::BufferAddress) -> VertexBuffer<T> {
        VertexBuffer::new(self.gpu.clone(), capacity)
    }

    /// Renders a frame to the provided target.
    pub fn render(&self, target: &wgpu::TextureView, render_data: &RenderData) {
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

            rp.set_bind_group(0, render_data.frame_uniforms.bind_group(), &[]);
            rp.set_vertex_buffer(0, render_data.quads.slice());
            rp.set_pipeline(&self.quad_pipeline);
            rp.draw(0..4, 0..render_data.quads.len() as u32);
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
