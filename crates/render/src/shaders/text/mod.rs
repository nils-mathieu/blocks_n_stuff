mod font;

mod instance;
pub use instance::*;

use std::mem::size_of;

use wgpu::util::DeviceExt;

use super::common::CommonResources;
use crate::{Gpu, VertexBufferSlice};

/// Pipeline responsible for rendering text.
pub struct TextPipeline {
    /// The bind group that contains the font texture.
    font: wgpu::BindGroup,
    /// The pipeline responsible for rendering text.
    pipeline: wgpu::RenderPipeline,
}

impl TextPipeline {
    /// Creates a new [`TextPipeline`] instance.
    pub fn new(gpu: &Gpu, resources: &CommonResources, output_format: wgpu::TextureFormat) -> Self {
        let font_layout = create_font_layout(gpu);
        let font = create_font(gpu, &font_layout, resources);
        let pipeline = create_pipeline(gpu, &font_layout, resources, output_format);
        Self { font, pipeline }
    }

    /// Renders the provided text instances.
    pub fn render<'res>(
        &'res self,
        _gpu: &Gpu,
        rp: &mut wgpu::RenderPass<'res>,
        data: VertexBufferSlice<'res, CharacterInstance>,
    ) {
        rp.set_pipeline(&self.pipeline);
        rp.set_bind_group(1, &self.font, &[]);
        rp.set_vertex_buffer(0, data.buffer);
        rp.draw(0..4, 0..data.len);
    }
}

fn create_font_layout(gpu: &Gpu) -> wgpu::BindGroupLayout {
    gpu.device
        .create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
            label: Some("Font Bind Group Layout"),
            entries: &[
                wgpu::BindGroupLayoutEntry {
                    binding: 0,
                    count: None,
                    visibility: wgpu::ShaderStages::FRAGMENT,
                    ty: wgpu::BindingType::Texture {
                        multisampled: false,
                        sample_type: wgpu::TextureSampleType::Float { filterable: false },
                        view_dimension: wgpu::TextureViewDimension::D2Array,
                    },
                },
                wgpu::BindGroupLayoutEntry {
                    binding: 1,
                    count: None,
                    visibility: wgpu::ShaderStages::FRAGMENT,
                    ty: wgpu::BindingType::Sampler(wgpu::SamplerBindingType::NonFiltering),
                },
            ],
        })
}

fn create_font(
    gpu: &Gpu,
    layout: &wgpu::BindGroupLayout,
    resources: &CommonResources,
) -> wgpu::BindGroup {
    let data = font::load();

    let texture = gpu.device.create_texture_with_data(
        &gpu.queue,
        &wgpu::TextureDescriptor {
            dimension: wgpu::TextureDimension::D2,
            format: wgpu::TextureFormat::R8Unorm,
            label: Some("Font Texture"),
            size: wgpu::Extent3d {
                width: 8,
                height: 8,
                depth_or_array_layers: font::BASIC_LEGACY.len() as u32,
            },
            mip_level_count: 1,
            sample_count: 1,
            usage: wgpu::TextureUsages::TEXTURE_BINDING,
            view_formats: &[],
        },
        wgpu::util::TextureDataOrder::LayerMajor,
        &data,
    );

    let view = texture.create_view(&wgpu::TextureViewDescriptor::default());

    gpu.device.create_bind_group(&wgpu::BindGroupDescriptor {
        label: Some("Font Bind Group"),
        layout,
        entries: &[
            wgpu::BindGroupEntry {
                binding: 0,
                resource: wgpu::BindingResource::TextureView(&view),
            },
            wgpu::BindGroupEntry {
                binding: 1,
                resource: wgpu::BindingResource::Sampler(&resources.pixel_sampler),
            },
        ],
    })
}

fn create_pipeline(
    gpu: &Gpu,
    font: &wgpu::BindGroupLayout,
    common_resources: &CommonResources,
    output_format: wgpu::TextureFormat,
) -> wgpu::RenderPipeline {
    let layout = gpu
        .device
        .create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
            label: Some("Text Pipeline Layout"),
            bind_group_layouts: &[&common_resources.frame_uniforms_layout, &font],
            push_constant_ranges: &[],
        });

    let shader_module = gpu
        .device
        .create_shader_module(wgpu::ShaderModuleDescriptor {
            label: Some("Text Shader Module"),
            source: wgpu::ShaderSource::Wgsl(include_str!("text.wgsl").into()),
        });

    gpu.device
        .create_render_pipeline(&wgpu::RenderPipelineDescriptor {
            label: Some("Text Pipeline"),
            layout: Some(&layout),
            vertex: wgpu::VertexState {
                module: &shader_module,
                entry_point: "vs_main",
                buffers: &[wgpu::VertexBufferLayout {
                    array_stride: size_of::<CharacterInstance>() as wgpu::BufferAddress,
                    step_mode: wgpu::VertexStepMode::Instance,
                    attributes: &[
                        // flags
                        wgpu::VertexAttribute {
                            format: wgpu::VertexFormat::Uint32,
                            offset: 0,
                            shader_location: 0,
                        },
                        // color
                        wgpu::VertexAttribute {
                            format: wgpu::VertexFormat::Uint32,
                            offset: 4,
                            shader_location: 1,
                        },
                        // position
                        wgpu::VertexAttribute {
                            format: wgpu::VertexFormat::Float32x2,
                            offset: 8,
                            shader_location: 2,
                        },
                        // size
                        wgpu::VertexAttribute {
                            format: wgpu::VertexFormat::Float32x2,
                            offset: 16,
                            shader_location: 3,
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
            multisample: wgpu::MultisampleState {
                alpha_to_coverage_enabled: false,
                count: 1,
                mask: !0,
            },
            primitive: wgpu::PrimitiveState {
                cull_mode: None,
                polygon_mode: wgpu::PolygonMode::Fill,
                conservative: false,
                front_face: wgpu::FrontFace::Ccw,
                strip_index_format: None,
                topology: wgpu::PrimitiveTopology::TriangleStrip,
                unclipped_depth: false,
            },
            depth_stencil: None,
            multiview: None,
        })
}
