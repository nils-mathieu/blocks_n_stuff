use std::mem::size_of;

use crate::{Gpu, Texture, VertexBufferSlice};

mod instance;
pub use instance::*;

/// The render pipeline responsible for rendering sprites in the UI.
pub struct UiSpritePipeline {
    /// The render pipeline object.
    pipeline: wgpu::RenderPipeline,
}

impl UiSpritePipeline {
    /// Creates a new [`UiSpritePipeline`] instance.
    pub fn new(gpu: &Gpu, output_format: wgpu::TextureFormat) -> Self {
        let pipeline = create_pipeline(gpu, output_format);
        Self { pipeline }
    }

    /// Rendders the provided collection of sprite instances.
    pub fn render<'res>(
        &'res self,
        rp: &mut wgpu::RenderPass<'res>,
        instances: VertexBufferSlice<'res, Sprite>,
        texture: &'res Texture,
    ) {
        rp.set_pipeline(&self.pipeline);
        rp.set_bind_group(1, &texture.bind_group, &[]);
        rp.set_vertex_buffer(0, instances.buffer);
        rp.draw(0..4, 0..instances.len);
    }
}

fn create_pipeline(gpu: &Gpu, output_format: wgpu::TextureFormat) -> wgpu::RenderPipeline {
    let res = gpu.resources.read();

    let shader_module = gpu
        .device
        .create_shader_module(wgpu::ShaderModuleDescriptor {
            label: Some("UI Sprite Shader"),
            source: wgpu::ShaderSource::Wgsl(include_str!("ui_sprite.wgsl").into()),
        });

    let pipeline_layout = gpu
        .device
        .create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
            bind_group_layouts: &[&res.frame_uniforms_layout, &res.texture_layout],
            push_constant_ranges: &[],
            label: Some("UI Sprite Pipeline Layout"),
        });

    gpu.device
        .create_render_pipeline(&wgpu::RenderPipelineDescriptor {
            label: Some("UI Sprite Pipeline"),
            layout: Some(&pipeline_layout),
            vertex: wgpu::VertexState {
                module: &shader_module,
                entry_point: "vs_main",
                buffers: &[wgpu::VertexBufferLayout {
                    array_stride: size_of::<Sprite>() as wgpu::BufferAddress,
                    step_mode: wgpu::VertexStepMode::Instance,
                    attributes: &[
                        // uv_transform
                        wgpu::VertexAttribute {
                            offset: 0,
                            format: wgpu::VertexFormat::Float32x4,
                            shader_location: 0,
                        },
                        // transform
                        wgpu::VertexAttribute {
                            offset: 16,
                            format: wgpu::VertexFormat::Float32x4,
                            shader_location: 1,
                        },
                        // position
                        wgpu::VertexAttribute {
                            offset: 32,
                            format: wgpu::VertexFormat::Float32x2,
                            shader_location: 2,
                        },
                        // uv_offset
                        wgpu::VertexAttribute {
                            offset: 40,
                            format: wgpu::VertexFormat::Float32x2,
                            shader_location: 3,
                        },
                        // color
                        wgpu::VertexAttribute {
                            offset: 48,
                            format: wgpu::VertexFormat::Uint32,
                            shader_location: 4,
                        },
                    ],
                }],
            },
            fragment: Some(wgpu::FragmentState {
                entry_point: "fs_main",
                module: &shader_module,
                targets: &[Some(wgpu::ColorTargetState {
                    blend: Some(wgpu::BlendState::ALPHA_BLENDING),
                    format: output_format,
                    write_mask: wgpu::ColorWrites::ALL,
                })],
            }),
            depth_stencil: None,
            multisample: wgpu::MultisampleState {
                alpha_to_coverage_enabled: false,
                count: 1,
                mask: !0,
            },
            primitive: wgpu::PrimitiveState {
                cull_mode: None,
                front_face: wgpu::FrontFace::Ccw,
                polygon_mode: wgpu::PolygonMode::Fill,
                strip_index_format: None,
                topology: wgpu::PrimitiveTopology::TriangleStrip,
                conservative: false,
                unclipped_depth: false,
            },
            multiview: None,
        })
}
