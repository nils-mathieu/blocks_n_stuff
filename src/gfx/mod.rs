//! This module defines everything that's needed to draw stuff on the screen.

use std::sync::Arc;

mod helpers;
mod shaders;

mod gpu;
pub use gpu::Gpu;

mod surface;
pub use surface::Surface;

pub mod render_data;

use self::helpers::{UniformBuffer, UniformBufferLayout};
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

    /// A full view into the depth buffer.
    depth_buffer_view: wgpu::TextureView,
}

impl Renderer {
    /// Creates a new [`Renderer`] instance.
    pub fn new(for_surface: &Surface) -> Self {
        let gpu = for_surface.gpu().clone();
        let (width, height) = for_surface.size();
        let format = for_surface.format();

        let frame_uniform_layout =
            UniformBufferLayout::new(gpu.clone(), wgpu::ShaderStages::VERTEX);

        Self {
            quad_pipeline: shaders::quad::create(&gpu.device, &frame_uniform_layout, format),

            frame_uniform_layout,

            depth_buffer_view: create_depth_buffer(&gpu, width, height),

            gpu,
        }
    }

    /// Returns the [`Gpu`] instance that was used to create this [`Renderer`].
    #[inline]
    pub fn gpu(&self) -> &Arc<Gpu> {
        &self.gpu
    }

    /// Creates a new [`UniformBuffer`] that follows the layout of the frame uniform layout.
    pub fn create_frame_uniform_buffer(&self) -> UniformBuffer<FrameUniforms> {
        self.frame_uniform_layout.instanciate(&self.gpu.device)
    }

    /// Notifies the renderer that the size of the window on which it is drawing has changed.
    pub fn notify_resized(&mut self, width: u32, height: u32) {
        self.depth_buffer_view = create_depth_buffer(&self.gpu, width, height);
    }

    /// Renders a frame to the provided target.
    pub fn render(&self, target: &wgpu::TextureView, render_data: &RenderData) {
        // Start recording the commands.
        let mut encoder = self.gpu.device.create_command_encoder(&Default::default());

        {
            let mut rp = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
                label: Some("Main Render Pass"),
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
                depth_stencil_attachment: Some(wgpu::RenderPassDepthStencilAttachment {
                    view: &self.depth_buffer_view,
                    depth_ops: Some(wgpu::Operations {
                        load: wgpu::LoadOp::Clear(1.0),
                        store: wgpu::StoreOp::Store,
                    }),
                    stencil_ops: None,
                }),
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

/// Creates a depth buffer that can be used to render 3D scenes.
fn create_depth_buffer(gpu: &Gpu, width: u32, height: u32) -> wgpu::TextureView {
    let depth_buffer = gpu.device.create_texture(&wgpu::TextureDescriptor {
        label: Some("Depth Buffer"),
        size: wgpu::Extent3d {
            width,
            height,
            depth_or_array_layers: 1,
        },
        mip_level_count: 1,
        sample_count: 1,
        dimension: wgpu::TextureDimension::D2,
        format: wgpu::TextureFormat::Depth32Float,
        usage: wgpu::TextureUsages::RENDER_ATTACHMENT,
        view_formats: &[],
    });

    depth_buffer.create_view(&wgpu::TextureViewDescriptor::default())
}
