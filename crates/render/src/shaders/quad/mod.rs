use wgpu::util::DeviceExt;

use std::mem::size_of;

use crate::Gpu;

mod instance;
pub use instance::*;

mod quads;
pub use quads::*;

/// Returns the alignment of [`ChunkUniforms`] for the provided [`Gpu`].
fn get_chunk_alignment(gpu: &Gpu) -> usize {
    wgpu::util::align_to(
        size_of::<ChunkUniforms>(),
        gpu.limits.min_uniform_buffer_offset_alignment as usize,
    )
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

    /// The render pipeline responsible for rendering opaque geometry.
    opaque_pipeline: wgpu::RenderPipeline,
    /// The render pipeline responsible for rendering transparent geometry.
    transparent_pipeline: wgpu::RenderPipeline,

    /// The pipeline responsible for rendering the depth map from the perspective of the sun.
    shadow_pipeline: wgpu::RenderPipeline,
}

impl QuadPipeline {
    /// Creates a new [`QuadPipeline`] instance.
    pub fn new(gpu: &Gpu, output_format: wgpu::TextureFormat) -> Self {
        let chunk_align = get_chunk_alignment(gpu);
        let chunk_uniforms_layout = create_chunk_uniforms_bind_group_layout(gpu, chunk_align);
        let (chunk_uniforms_buffer, chunk_uniforms_bind_group) = create_chunk_uniforms_buffer(
            gpu,
            &chunk_uniforms_layout,
            chunk_align as wgpu::BufferAddress * 64,
            chunk_align,
        );
        let pipeline_layout = create_pipeline_layout(gpu, &chunk_uniforms_layout);
        let shader_module = create_shader_module(gpu);
        let opaque_pipeline = create_pipeline(
            gpu,
            &pipeline_layout,
            &shader_module,
            output_format,
            PipelineFlavor::Opaque,
        );
        let transparent_pipeline = create_pipeline(
            gpu,
            &pipeline_layout,
            &shader_module,
            output_format,
            PipelineFlavor::Transparent,
        );
        let shadow_pipeline = create_shadow_pipeline(gpu, &chunk_uniforms_layout);

        Self {
            chunk_uniforms_layout,
            chunk_uniforms_bind_group,
            chunk_uniforms_buffer,
            chunk_align,
            opaque_pipeline,
            transparent_pipeline,
            shadow_pipeline,
        }
    }

    /// Prepares the pipeline for rendering the provided [`Quads`].
    pub fn prepare(&mut self, gpu: &Gpu, quads: &Quads) {
        // Copy the chunk data into the GPU buffer, eventually resizing it if needed.
        if self.chunk_uniforms_buffer.size() < quads.chunks.len() as u64 {
            self.chunk_uniforms_buffer =
                gpu.device
                    .create_buffer_init(&wgpu::util::BufferInitDescriptor {
                        contents: quads.chunks.as_slice(),
                        label: Some("Chunk Uniforms Buffer"),
                        usage: wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST,
                    });

            self.chunk_uniforms_bind_group = create_chunk_uniforms_bind_group(
                gpu,
                &self.chunk_uniforms_layout,
                &self.chunk_uniforms_buffer,
                self.chunk_align,
            );
        } else {
            gpu.queue
                .write_buffer(&self.chunk_uniforms_buffer, 0, &quads.chunks);
        }
    }

    /// Renders to the shadowmap.
    pub fn render_shadows<'res>(&'res self, rp: &mut wgpu::RenderPass<'res>, quads: &Quads<'res>) {
        // Draw each instance buffer registered, binding it to the correct chunk uniforms
        // using dynamic offsets.
        rp.set_pipeline(&self.shadow_pipeline);
        for buf in &quads.opaque_buffers {
            rp.set_bind_group(
                1,
                &self.chunk_uniforms_bind_group,
                &[buf.chunk_idx * self.chunk_align as u32],
            );
            rp.set_vertex_buffer(0, buf.slice.buffer);
            rp.draw(0..4, 0..buf.slice.len);
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
    /// 3. `shadow_map` (bind group 3)
    ///
    /// This function will clobber bind group 1.
    #[profiling::function]
    pub fn render<'res>(&'res self, rp: &mut wgpu::RenderPass<'res>, quads: &Quads<'res>) {
        // Draw each instance buffer registered, binding it to the correct chunk uniforms
        // using dynamic offsets.
        rp.set_pipeline(&self.opaque_pipeline);
        for buf in &quads.opaque_buffers {
            rp.set_bind_group(
                1,
                &self.chunk_uniforms_bind_group,
                &[buf.chunk_idx * self.chunk_align as u32],
            );
            rp.set_vertex_buffer(0, buf.slice.buffer);
            rp.draw(0..4, 0..buf.slice.len);
        }

        rp.set_pipeline(&self.transparent_pipeline);
        for buf in &quads.transparent_buffers {
            rp.set_bind_group(
                1,
                &self.chunk_uniforms_bind_group,
                &[buf.chunk_idx * self.chunk_align as u32],
            );
            rp.set_vertex_buffer(0, buf.slice.buffer);
            rp.draw(0..4, 0..buf.slice.len);
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
    size: wgpu::BufferAddress,
    align: usize,
) -> (wgpu::Buffer, wgpu::BindGroup) {
    let buf = gpu.device.create_buffer(&wgpu::BufferDescriptor {
        label: Some("Chunk Uniforms Buffer"),
        mapped_at_creation: false,
        size,
        usage: wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST,
    });

    let bind_group = create_chunk_uniforms_bind_group(gpu, layout, &buf, align);

    (buf, bind_group)
}

fn create_chunk_uniforms_bind_group(
    gpu: &Gpu,
    layout: &wgpu::BindGroupLayout,
    buffer: &wgpu::Buffer,
    align: usize,
) -> wgpu::BindGroup {
    gpu.device.create_bind_group(&wgpu::BindGroupDescriptor {
        label: Some("Chunk Uniforms Bind Group"),
        layout,
        entries: &[wgpu::BindGroupEntry {
            binding: 0,
            resource: wgpu::BindingResource::Buffer(wgpu::BufferBinding {
                buffer,
                offset: 0,
                size: wgpu::BufferSize::new(align as wgpu::BufferAddress),
            }),
        }],
    })
}

/// A way of creating a [`wgpu::RenderPipeline`].
///
/// This is needed because some of the geometry we're rendering is opaque, and some of it
/// is transparent.
///
/// Transparent geometry has to be rendered after opaque geometry, and transparent geometry
/// has to be sorted by distance to the camera.
enum PipelineFlavor {
    /// The opaque pipeline writes to the depth buffer but does not use blending when writing
    /// to the color buffer.
    Opaque,
    /// The transparent pipeline does not write to the depth buffer, but uses blending
    /// when writing colors.
    Transparent,
}

fn create_pipeline_layout(
    gpu: &Gpu,
    chunk_uniforms_layout: &wgpu::BindGroupLayout,
) -> wgpu::PipelineLayout {
    let res = gpu.resources.read();

    gpu.device
        .create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
            label: Some("Quad Pipeline Layout"),
            bind_group_layouts: &[
                &res.frame_uniforms_layout,
                chunk_uniforms_layout,
                &res.texture_atlas_layout,
                &res.shadow_map_layout,
            ],
            push_constant_ranges: &[],
        })
}

fn create_shader_module(gpu: &Gpu) -> wgpu::ShaderModule {
    gpu.device
        .create_shader_module(wgpu::ShaderModuleDescriptor {
            label: Some("Quad Pipeline Shader Module"),
            source: wgpu::ShaderSource::Wgsl(include_str!("quad.wgsl").into()),
        })
}

/// Creates the [`wgpu::RenderPipeline`] responsible for rendering axis-aligned quad instances.
fn create_pipeline(
    gpu: &Gpu,
    layout: &wgpu::PipelineLayout,
    shader_module: &wgpu::ShaderModule,
    output_format: wgpu::TextureFormat,
    flavor: PipelineFlavor,
) -> wgpu::RenderPipeline {
    gpu.device
        .create_render_pipeline(&wgpu::RenderPipelineDescriptor {
            label: Some(match flavor {
                PipelineFlavor::Opaque => "Quad Opaque Pipeline",
                PipelineFlavor::Transparent => "Quad Transparent Pipeline",
            }),
            layout: Some(layout),
            vertex: wgpu::VertexState {
                buffers: &[wgpu::VertexBufferLayout {
                    array_stride: size_of::<QuadInstance>() as wgpu::BufferAddress,
                    attributes: &[
                        // flags
                        wgpu::VertexAttribute {
                            format: wgpu::VertexFormat::Uint32,
                            offset: 0,
                            shader_location: 0,
                        },
                        // texture
                        wgpu::VertexAttribute {
                            format: wgpu::VertexFormat::Uint32,
                            offset: 4,
                            shader_location: 1,
                        },
                    ],
                    step_mode: wgpu::VertexStepMode::Instance,
                }],
                entry_point: "vs_main",
                module: shader_module,
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
                module: shader_module,
                targets: &[Some(wgpu::ColorTargetState {
                    blend: Some(match flavor {
                        PipelineFlavor::Opaque => wgpu::BlendState::REPLACE,
                        PipelineFlavor::Transparent => wgpu::BlendState::ALPHA_BLENDING,
                    }),
                    format: output_format,
                    write_mask: wgpu::ColorWrites::ALL,
                })],
            }),
            depth_stencil: Some(wgpu::DepthStencilState {
                bias: wgpu::DepthBiasState::default(),
                depth_compare: wgpu::CompareFunction::LessEqual,
                depth_write_enabled: match flavor {
                    PipelineFlavor::Opaque => true,
                    PipelineFlavor::Transparent => false,
                },
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

fn create_shadow_pipeline(
    gpu: &Gpu,
    chunk_uniforms_layout: &wgpu::BindGroupLayout,
) -> wgpu::RenderPipeline {
    let shader_module = gpu
        .device
        .create_shader_module(wgpu::ShaderModuleDescriptor {
            label: Some("Quad Shadow Pipeline Shader Module"),
            source: wgpu::ShaderSource::Wgsl(include_str!("quad_shadow.wgsl").into()),
        });

    let res = gpu.resources.read();

    let shadow_pipeline_layout =
        gpu.device
            .create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
                label: Some("Quad Shadow Pipeline Layout"),
                bind_group_layouts: &[&res.frame_uniforms_layout, &chunk_uniforms_layout],
                push_constant_ranges: &[],
            });

    gpu.device
        .create_render_pipeline(&wgpu::RenderPipelineDescriptor {
            depth_stencil: Some(wgpu::DepthStencilState {
                bias: wgpu::DepthBiasState::default(),
                depth_compare: wgpu::CompareFunction::LessEqual,
                depth_write_enabled: true,
                format: crate::DEPTH_FORMAT,
                stencil: wgpu::StencilState::default(),
            }),
            fragment: None,
            label: Some("Quad Shadow Pipeline"),
            layout: Some(&shadow_pipeline_layout),
            multisample: wgpu::MultisampleState {
                alpha_to_coverage_enabled: false,
                count: 1,
                mask: !0,
            },
            multiview: None,
            primitive: wgpu::PrimitiveState {
                conservative: false,
                cull_mode: Some(wgpu::Face::Back),
                polygon_mode: wgpu::PolygonMode::Fill,
                front_face: wgpu::FrontFace::Cw,
                topology: wgpu::PrimitiveTopology::TriangleStrip,
                strip_index_format: None,
                unclipped_depth: false,
            },
            vertex: wgpu::VertexState {
                buffers: &[wgpu::VertexBufferLayout {
                    array_stride: size_of::<QuadInstance>() as wgpu::BufferAddress,
                    attributes: &[
                        // flags
                        wgpu::VertexAttribute {
                            format: wgpu::VertexFormat::Uint32,
                            offset: 0,
                            shader_location: 0,
                        },
                        // texture
                        wgpu::VertexAttribute {
                            format: wgpu::VertexFormat::Uint32,
                            offset: 4,
                            shader_location: 1,
                        },
                    ],
                    step_mode: wgpu::VertexStepMode::Instance,
                }],
                entry_point: "vs_main",
                module: &shader_module,
            },
        })
}
