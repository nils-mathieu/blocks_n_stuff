use super::render_data::RenderData;
use super::shaders::{self, RenderResources};
use super::Gpu;

/// Represents the render graph of our application.
pub struct RenderGraph {
    /// See [`shaders::quad::create`].
    quad_pipeline: wgpu::RenderPipeline,

    /// A full view into the depth buffer.
    ///
    /// This must be re-created when the surface is resized.
    depth_buffer_view: wgpu::TextureView,
}

impl RenderGraph {
    /// Creates a new [`RenderGraph`] instance.
    pub fn new(gpu: &Gpu, resources: &RenderResources, c: &wgpu::SurfaceConfiguration) -> Self {
        let depth_buffer_view = create_depth_buffer(gpu, c.width, c.height);
        let quad_pipeline = shaders::quad::create(gpu, resources, c.format);

        Self {
            quad_pipeline,
            depth_buffer_view,
        }
    }

    /// Notifies the render graph that the size of the window on which it is drawing has changed.
    pub fn notify_resized(&mut self, gpu: &Gpu, width: u32, height: u32) {
        self.depth_buffer_view = create_depth_buffer(gpu, width, height);
    }

    /// Renders a frame to the provided target.
    ///
    /// The provided `output` texture must be of the same format as the one provided to
    /// [`RenderGraph::new`].
    pub fn render_on(&mut self, gpu: &Gpu, render_data: &RenderData, output: &wgpu::TextureView) {
        // Start recording the commands.
        let mut encoder = gpu.device.create_command_encoder(&Default::default());

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
                        view: output,
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

        gpu.queue.submit(Some(encoder.finish()));
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
