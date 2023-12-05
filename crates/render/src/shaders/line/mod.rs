use std::mem::{size_of, size_of_val};

mod instance;
pub use instance::*;

use wgpu::util::DeviceExt;
use wgpu::RenderPass;

use crate::Gpu;

/// Contains the state required to draw lines using GPU resources.
pub struct LinePipeline {
    /// The pipeline responsible for rendering lines.
    pipeline: wgpu::RenderPipeline,
    /// The buffer responsible for storing the line instances.
    buffer: wgpu::Buffer,
}

impl LinePipeline {
    /// Creates a new [`LinePipeline`] instance.
    pub fn new(gpu: &Gpu, output_format: wgpu::TextureFormat) -> Self {
        let pipeline = create_pipeline(gpu, output_format);
        let buffer = create_line_instance_buffer(gpu);

        Self { pipeline, buffer }
    }

    /// Renders the provided lines.
    ///
    /// # Remarks
    ///
    /// This function expects the bind group 0 to be bound to the frame uniforms.
    #[profiling::function]
    pub fn render<'res>(
        &'res mut self,
        gpu: &Gpu,
        rp: &mut RenderPass<'res>,
        lines: &[LineInstance],
    ) {
        if lines.is_empty() {
            return;
        }

        // Copy the lines into the GPU buffer, eventually resizing it if needed.
        if self.buffer.size() < size_of_val(lines) as u64 {
            self.buffer = gpu
                .device
                .create_buffer_init(&wgpu::util::BufferInitDescriptor {
                    contents: bytemuck::cast_slice(lines),
                    label: Some("Line Vertices Buffer"),
                    usage: wgpu::BufferUsages::VERTEX | wgpu::BufferUsages::COPY_DST,
                });
        } else {
            gpu.queue
                .write_buffer(&self.buffer, 0, bytemuck::cast_slice(lines));
        }

        // Draw all the lines as a batch.
        rp.set_pipeline(&self.pipeline);
        rp.set_vertex_buffer(0, self.buffer.slice(..));
        rp.draw(0..4, 0..lines.len() as u32);
    }
}

/// Creates the render pipeline that's responsible for drawing lines.
fn create_pipeline(gpu: &Gpu, output_format: wgpu::TextureFormat) -> wgpu::RenderPipeline {
    let res = gpu.resources.read();

    let shader_module = gpu
        .device
        .create_shader_module(wgpu::ShaderModuleDescriptor {
            label: Some("Line Shader Module"),
            source: wgpu::ShaderSource::Wgsl(include_str!("line.wgsl").into()),
        });

    let pipeline_layout = gpu
        .device
        .create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
            label: Some("Line Pipeline Layout"),
            bind_group_layouts: &[&res.frame_uniforms_layout],
            push_constant_ranges: &[],
        });

    gpu.device
        .create_render_pipeline(&wgpu::RenderPipelineDescriptor {
            depth_stencil: Some(wgpu::DepthStencilState {
                format: crate::DEPTH_FORMAT,
                depth_write_enabled: true,
                depth_compare: wgpu::CompareFunction::LessEqual,
                stencil: wgpu::StencilState::default(),
                bias: wgpu::DepthBiasState::default(),
            }),
            fragment: Some(wgpu::FragmentState {
                entry_point: "fs_main",
                module: &shader_module,
                targets: &[Some(wgpu::ColorTargetState {
                    blend: Some(wgpu::BlendState::ALPHA_BLENDING),
                    format: output_format,
                    write_mask: wgpu::ColorWrites::ALL,
                })],
            }),
            label: Some("Line Pipeline"),
            layout: Some(&pipeline_layout),
            primitive: wgpu::PrimitiveState {
                conservative: false,
                cull_mode: None,
                polygon_mode: wgpu::PolygonMode::Fill,
                front_face: wgpu::FrontFace::Cw,
                topology: wgpu::PrimitiveTopology::TriangleStrip,
                strip_index_format: None,
                unclipped_depth: false,
            },
            multisample: wgpu::MultisampleState {
                alpha_to_coverage_enabled: false,
                count: 1,
                mask: !0,
            },
            multiview: None,
            vertex: wgpu::VertexState {
                buffers: &[wgpu::VertexBufferLayout {
                    array_stride: size_of::<LineInstance>() as wgpu::BufferAddress,
                    step_mode: wgpu::VertexStepMode::Instance,
                    attributes: &[
                        // start
                        wgpu::VertexAttribute {
                            format: wgpu::VertexFormat::Float32x3,
                            offset: 0,
                            shader_location: 0,
                        },
                        // width
                        wgpu::VertexAttribute {
                            format: wgpu::VertexFormat::Float32,
                            offset: 12,
                            shader_location: 1,
                        },
                        // end
                        wgpu::VertexAttribute {
                            format: wgpu::VertexFormat::Float32x3,
                            offset: 16,
                            shader_location: 2,
                        },
                        // flags
                        wgpu::VertexAttribute {
                            format: wgpu::VertexFormat::Uint32,
                            offset: 28,
                            shader_location: 3,
                        },
                        // color
                        wgpu::VertexAttribute {
                            format: wgpu::VertexFormat::Uint32,
                            offset: 32,
                            shader_location: 4,
                        },
                    ],
                }],
                entry_point: "vs_main",
                module: &shader_module,
            },
        })
}

/// Creates a buffer that can be used to store line instances.
fn create_line_instance_buffer(gpu: &Gpu) -> wgpu::Buffer {
    gpu.device.create_buffer(&wgpu::BufferDescriptor {
        label: Some("Line Instance Buffer"),
        mapped_at_creation: false,
        size: 64 * size_of::<LineInstance>() as wgpu::BufferAddress,
        usage: wgpu::BufferUsages::VERTEX | wgpu::BufferUsages::COPY_DST,
    })
}
