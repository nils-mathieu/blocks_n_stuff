mod instance;
pub use instance::*;

use std::mem::size_of;

use bns_render_preprocessor::preprocess;

use crate::{Gpu, Vertices};

/// The source of the shader, after preprocessing.
const SOURCE: &str = preprocess!("crates/render/src/shaders/quad/quad.wgsl");

/// Returns the alignment of [`ChunkUniforms`] for the provided [`Gpu`].
fn get_chunk_alignment(gpu: &Gpu) -> usize {
    wgpu::util::align_to(
        size_of::<ChunkUniforms>(),
        gpu.limits.min_uniform_buffer_offset_alignment as usize,
    )
}

/// A instance buffer that's ready to be rendered by the [`QuadPipeline`].
struct QuadBuffer<'res> {
    /// The chunk uniforms that is associated with the quad instances in the buffer.
    ///
    /// This is the offset within the chunk uniforms buffer to use when setting the bind group.
    chunk_idx: u32,
    /// The quad instances of the chunk.
    ///
    /// This is expected to be a collection of `len` instances of [`QuadInstance`].
    vertices: wgpu::BufferSlice<'res>,
    /// The number of [`QuadInstance`] instances in the buffer slice.
    len: u32,
}

/// A collection type used to properly lay out [`QuadInstance`]s and [`ChunkUniforms`] in a buffer.
pub struct Quads<'res> {
    /// The alignment of the [`ChunkUniforms`] instances in the buffer.
    ///
    /// When a new instance is added to the buffer, it must be aligned to this value.
    chunk_align: usize,
    /// The chunk uniforms that are used by the quads in the buffer.
    ///
    /// # Remarks
    ///
    /// This buffer is supposed to contain a bunch of [`ChunkUniforms`] instances. However, because
    /// it will be indexed using a dynamic offset, it's alignment depends on the minimum alignment
    /// available on the GPU. This means that the buffer may contain padding between the
    /// [`ChunkUniforms`] instances (and it will, because the minimum alignment varies between
    /// 64 and 256 bytes).
    chunks: Vec<u8>,
    /// The quad instances that are used by the quads in the buffer.
    buffers: Vec<QuadBuffer<'res>>,
}

impl<'res> Quads<'res> {
    /// Creates a new [`Quads`] instance.
    pub fn new(gpu: &Gpu) -> Self {
        Self {
            chunk_align: get_chunk_alignment(gpu),
            chunks: Vec::new(),
            buffers: Vec::new(),
        }
    }

    /// Resets the [`Quads`] instance with a potentially longer lifetime, allowing it to be used
    /// again without having to reallocate the buffers.
    pub fn reset<'res2>(mut self) -> Quads<'res2> {
        self.buffers.clear();

        // SAFETY:
        //  1. The buffer is empty, meaning that no references are actually being transmuted into
        //     a potentially longer lifetime.
        //  2. Two types that only differ in lifetime always have the same memory layout.
        let buffers = unsafe { std::mem::transmute(self.buffers) };

        Quads {
            chunk_align: self.chunk_align,
            chunks: self.chunks,
            buffers,
        }
    }

    /// Registers a [`ChunkUniforms`] instance to be used.
    fn add_chunk_uniforms(&mut self, chunk: &ChunkUniforms) -> u32 {
        let index = self.chunks.len() / self.chunk_align;

        self.chunks.extend_from_slice(bytemuck::bytes_of(chunk));
        self.chunks.resize(self.chunk_align * (index + 1), 0);

        index as u32
    }

    /// Registers a new chunk to be rendered.
    pub fn add_quad_buffer(
        &mut self,
        chunk: &ChunkUniforms,
        buffer: &'res dyn Vertices<Vertex = QuadInstance>,
    ) {
        let len = buffer.len();

        if len == 0 {
            return;
        }

        let chunk_idx = self.add_chunk_uniforms(chunk);
        self.buffers.push(QuadBuffer {
            chunk_idx,
            vertices: buffer.slice(),
            len,
        });
    }
}

/// Contains the data that's ready to be rendered by the [`QuadPipeline`].
pub struct Prepared<'res> {
    /// The quad instances that are ready to be rendered.
    buffers: &'res [QuadBuffer<'res>],
}

/// The rendering pipeline responsible for rendering axis-aligned quad [`Instance`]s.
pub struct QuadPipeline {
    /// The alignment of the [`ChunkUniforms`] instances in the buffer.
    chunk_align: usize,

    /// This buffer contains a bunch of [`ChunkUniforms`] instances.
    ///
    /// It is bound to bind group 1 using `chunk_uniforms_bind_group`.
    ///
    /// In order to select the correct [`ChunkUniforms`] instance within the buffer, a dynamic
    /// offset is used when setting the bind group.
    chunk_uniforms_buffer: wgpu::Buffer,
    /// The bind group layout that was used to create `chunk_uniforms_bind_group`.
    chunk_uniforms_layout: wgpu::BindGroupLayout,
    /// The bind group that's used to bind `chunk_uniforms_buffer`.
    chunk_uniforms_bind_group: wgpu::BindGroup,

    /// The render pipeline.
    pipeline: wgpu::RenderPipeline,
}

impl QuadPipeline {
    /// Creates a new [`QuadPipeline`] instance.
    pub fn new(
        gpu: &Gpu,
        frame_uniforms_layout: &wgpu::BindGroupLayout,
        texture_atlas_layout: &wgpu::BindGroupLayout,
        output_format: wgpu::TextureFormat,
    ) -> Self {
        let chunk_align = get_chunk_alignment(gpu);
        let chunk_uniforms_layout = create_chunk_uniforms_bind_group_layout(gpu, chunk_align);
        let (chunk_uniforms_buffer, chunk_uniforms_bind_group) = create_chunk_uniforms_buffer(
            gpu,
            &chunk_uniforms_layout,
            false,
            chunk_align as wgpu::BufferAddress * 64,
            chunk_align,
        );
        let pipeline = create_pipeline(
            gpu,
            frame_uniforms_layout,
            &chunk_uniforms_layout,
            texture_atlas_layout,
            output_format,
        );

        Self {
            chunk_uniforms_layout,
            chunk_uniforms_bind_group,
            chunk_uniforms_buffer,
            chunk_align,
            pipeline,
        }
    }

    /// Prepare the provided quad instances for rendering.
    pub fn prepare<'res>(&mut self, gpu: &Gpu, quads: &'res Quads<'res>) -> Prepared<'res> {
        // Write the chunk uniforms to the buffer.
        if self.chunk_uniforms_buffer.size() < quads.chunks.len() as u64 {
            (self.chunk_uniforms_buffer, self.chunk_uniforms_bind_group) =
                create_chunk_uniforms_buffer(
                    gpu,
                    &self.chunk_uniforms_layout,
                    true,
                    quads.chunks.len() as wgpu::BufferAddress,
                    self.chunk_align,
                );

            unsafe {
                // The buffer is mapped in memory.
                let mut mapped_range = self.chunk_uniforms_buffer.slice(..).get_mapped_range_mut();

                // SAFETY:
                //  We created a buffer that's large enough to store the chunk uniforms.
                std::ptr::copy_nonoverlapping(
                    quads.chunks.as_ptr(),
                    mapped_range.as_mut_ptr(),
                    quads.chunks.len(),
                );
            }

            self.chunk_uniforms_buffer.unmap();
        } else {
            // The buffer is large enough, but it might not be mapped in memory.
            gpu.queue
                .write_buffer(&self.chunk_uniforms_buffer, 0, &quads.chunks);
        }

        Prepared {
            buffers: &quads.buffers,
        }
    }

    /// Renders the provided quad instances to the provided [`RenderTarget`].
    ///
    /// # Remarks
    ///
    /// The provided render pass must have the following bind groups upon entering this function:
    ///
    /// 1. `frame_uniforms` (bind group 0)
    /// 2. `texture_atlas` (bind group 2)
    ///
    /// This function will clobber bind group 1.
    pub fn render<'res>(&'res mut self, rp: &mut wgpu::RenderPass<'res>, prepared: Prepared<'res>) {
        rp.set_pipeline(&self.pipeline);
        for buf in prepared.buffers {
            rp.set_bind_group(
                1,
                &self.chunk_uniforms_bind_group,
                &[buf.chunk_idx * self.chunk_align as u32],
            );
            rp.set_vertex_buffer(0, buf.vertices);
            rp.draw(0..4, 0..buf.len);
        }
    }
}

fn create_chunk_uniforms_bind_group_layout(gpu: &Gpu, align: usize) -> wgpu::BindGroupLayout {
    gpu.device
        .create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
            label: Some("Chunks Uniforms Bind Group Layout"),
            entries: &[wgpu::BindGroupLayoutEntry {
                binding: 0,
                count: None,
                ty: wgpu::BindingType::Buffer {
                    has_dynamic_offset: true,
                    min_binding_size: wgpu::BufferSize::new(align as wgpu::BufferAddress),
                    ty: wgpu::BufferBindingType::Uniform,
                },
                visibility: wgpu::ShaderStages::VERTEX,
            }],
        })
}

/// Creates a new bind group and buffer for the chunk uniforms.
fn create_chunk_uniforms_buffer(
    gpu: &Gpu,
    layout: &wgpu::BindGroupLayout,
    mapped_at_creation: bool,
    size: wgpu::BufferAddress,
    align: usize,
) -> (wgpu::Buffer, wgpu::BindGroup) {
    let buffer = gpu.device.create_buffer(&wgpu::BufferDescriptor {
        label: Some("Chunk Uniforms Buffer"),
        mapped_at_creation,
        size,
        usage: wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST,
    });

    let bind_group = gpu.device.create_bind_group(&wgpu::BindGroupDescriptor {
        label: Some("Chunk Uniforms Bind Group"),
        layout,
        entries: &[wgpu::BindGroupEntry {
            binding: 0,
            resource: wgpu::BindingResource::Buffer(wgpu::BufferBinding {
                buffer: &buffer,
                offset: 0,
                size: wgpu::BufferSize::new(align as wgpu::BufferAddress),
            }),
        }],
    });

    (buffer, bind_group)
}

/// Creates the [`wgpu::RenderPipeline`] responsible for rendering axis-aligned quad instances.
fn create_pipeline(
    gpu: &Gpu,
    frame_uniforms_layout: &wgpu::BindGroupLayout,
    chunk_uniforms_layout: &wgpu::BindGroupLayout,
    texture_atlas_layout: &wgpu::BindGroupLayout,
    output_format: wgpu::TextureFormat,
) -> wgpu::RenderPipeline {
    let shader_module = gpu
        .device
        .create_shader_module(wgpu::ShaderModuleDescriptor {
            label: Some("Quad Pipeline Shader Module"),
            source: wgpu::ShaderSource::Wgsl(SOURCE.into()),
        });

    let layout = gpu
        .device
        .create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
            label: Some("Quad Pipeline Layout"),
            bind_group_layouts: &[
                frame_uniforms_layout,
                chunk_uniforms_layout,
                texture_atlas_layout,
            ],
            push_constant_ranges: &[],
        });

    gpu.device
        .create_render_pipeline(&wgpu::RenderPipelineDescriptor {
            label: Some("Quad Pipeline"),
            layout: Some(&layout),
            vertex: wgpu::VertexState {
                buffers: &[wgpu::VertexBufferLayout {
                    array_stride: size_of::<QuadInstance>() as wgpu::BufferAddress,
                    attributes: &[wgpu::VertexAttribute {
                        format: wgpu::VertexFormat::Uint32,
                        offset: 0,
                        shader_location: 0,
                    }],
                    step_mode: wgpu::VertexStepMode::Instance,
                }],
                entry_point: "vs_main",
                module: &shader_module,
            },
            primitive: wgpu::PrimitiveState {
                conservative: false,
                cull_mode: Some(wgpu::Face::Back),
                polygon_mode: wgpu::PolygonMode::Fill,
                front_face: wgpu::FrontFace::Cw,
                topology: wgpu::PrimitiveTopology::TriangleStrip,
                strip_index_format: None,
                unclipped_depth: false,
            },
            fragment: Some(wgpu::FragmentState {
                entry_point: "fs_main",
                module: &shader_module,
                targets: &[Some(wgpu::ColorTargetState {
                    blend: Some(wgpu::BlendState::REPLACE),
                    format: output_format,
                    write_mask: wgpu::ColorWrites::ALL,
                })],
            }),
            depth_stencil: Some(wgpu::DepthStencilState {
                bias: wgpu::DepthBiasState::default(),
                depth_compare: wgpu::CompareFunction::LessEqual,
                depth_write_enabled: true,
                format: crate::DEPTH_FORMAT,
                stencil: wgpu::StencilState::default(),
            }),
            multisample: wgpu::MultisampleState {
                alpha_to_coverage_enabled: false,
                count: 1,
                mask: !0,
            },
            multiview: None,
        })
}
