use super::render_data::{ChunkUniforms, FrameUniforms, RenderData};
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

    /// The resources that were used to create this [`RenderGraph`].
    resources: RenderResources,

    /// The buffer responsible for storing the frame uniforms.
    ///
    /// This buffer is supposed to store an instance of [`FrameUniforms`].
    frame_uniforms: wgpu::Buffer,
    /// A bind group that references `frame_uniforms`.
    frame_uniforms_bind_group: wgpu::BindGroup,

    /// The buffer responsible for storing the chunk uniforms.
    chunk_uniforms: wgpu::Buffer,
    /// A bind group that references `chunk_uniforms`.
    chunk_uniforms_bind_group: wgpu::BindGroup,
}

impl RenderGraph {
    /// Creates a new [`RenderGraph`] instance.
    pub fn new(gpu: &Gpu, c: &wgpu::SurfaceConfiguration) -> Self {
        let resources = RenderResources::new(gpu);

        let depth_buffer_view = create_depth_buffer(gpu, c.width, c.height);
        let quad_pipeline = shaders::quad::create(gpu, &resources, c.format);
        let (frame_uniforms, frame_uniforms_bind_group) = create_frame_uniforms(gpu, &resources);
        let (chunk_uniforms, chunk_uniforms_bind_group) =
            create_chunks_uniforms(gpu, &resources, 64);

        Self {
            quad_pipeline,
            depth_buffer_view,
            frame_uniforms,
            frame_uniforms_bind_group,
            chunk_uniforms,
            chunk_uniforms_bind_group,
            resources,
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
        // Write the frame uniforms.
        gpu.queue.write_buffer(
            &self.frame_uniforms,
            0,
            bytemuck::bytes_of(&render_data.frame_uniforms),
        );

        // Write the chunk uniforms.
        if std::mem::size_of_val(render_data.chunk_uniforms) as wgpu::BufferAddress
            > self.chunk_uniforms.size()
        {
            (self.chunk_uniforms, self.chunk_uniforms_bind_group) = create_chunks_uniforms(
                gpu,
                &self.resources,
                render_data.chunk_uniforms.len() as wgpu::BufferAddress,
            );
        }

        gpu.queue.write_buffer(
            &self.chunk_uniforms,
            0,
            bytemuck::cast_slice(render_data.chunk_uniforms),
        );

        // Start recording the commands.
        let mut encoder = gpu.device.create_command_encoder(&Default::default());

        {
            let mut rp = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
                label: Some("Main Render Pass"),
                color_attachments: &[
                    // The main output attachment.
                    Some(wgpu::RenderPassColorAttachment {
                        ops: wgpu::Operations {
                            load: wgpu::LoadOp::Clear(wgpu::Color {
                                a: render_data.clear_color.w as f64,
                                r: render_data.clear_color.x as f64,
                                g: render_data.clear_color.y as f64,
                                b: render_data.clear_color.z as f64,
                            }),
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

            rp.set_bind_group(0, &self.frame_uniforms_bind_group, &[]);
            rp.set_pipeline(&self.quad_pipeline);
            for (chunk_index, quad) in render_data.quads.iter().enumerate() {
                rp.set_bind_group(
                    1,
                    &self.chunk_uniforms_bind_group,
                    &[chunk_index as u32 * std::mem::size_of::<ChunkUniforms>() as u32],
                );
                rp.set_vertex_buffer(
                    0,
                    quad.buffer.slice(
                        ..quad.len as wgpu::BufferAddress
                            * std::mem::size_of::<ChunkUniforms>() as wgpu::BufferAddress,
                    ),
                );
                rp.draw(0..4, 0..quad.len);
            }
        }

        gpu.queue.submit(Some(encoder.finish()));
    }
}

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

fn create_frame_uniforms(
    gpu: &Gpu,
    resources: &RenderResources,
) -> (wgpu::Buffer, wgpu::BindGroup) {
    let buffer = gpu.device.create_buffer(&wgpu::BufferDescriptor {
        label: Some("Frame Uniforms Buffer"),
        size: std::mem::size_of::<FrameUniforms>() as wgpu::BufferAddress,
        usage: wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST,
        mapped_at_creation: false,
    });

    let bind_group = gpu.device.create_bind_group(&wgpu::BindGroupDescriptor {
        label: Some("Frame Uniforms Bind Group"),
        layout: &resources.frame_layout,
        entries: &[wgpu::BindGroupEntry {
            binding: 0,
            resource: buffer.as_entire_binding(),
        }],
    });

    (buffer, bind_group)
}

fn create_chunks_uniforms(
    gpu: &Gpu,
    resources: &RenderResources,
    capacity: wgpu::BufferAddress,
) -> (wgpu::Buffer, wgpu::BindGroup) {
    let buffer = gpu.device.create_buffer(&wgpu::BufferDescriptor {
        label: Some("Chunks Uniforms Buffer"),
        size: std::mem::size_of::<ChunkUniforms>() as wgpu::BufferAddress * capacity,
        usage: wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST,
        mapped_at_creation: false,
    });

    let bind_group = gpu.device.create_bind_group(&wgpu::BindGroupDescriptor {
        label: Some("Chunks Uniforms Bind Group"),
        layout: &resources.chunk_layout,
        entries: &[wgpu::BindGroupEntry {
            binding: 0,
            resource: wgpu::BindingResource::Buffer(wgpu::BufferBinding {
                buffer: &buffer,
                offset: 0,
                size: wgpu::BufferSize::new(std::mem::size_of::<ChunkUniforms>() as u64),
            }),
        }],
    });

    (buffer, bind_group)
}
