use std::mem::size_of;

use bns_render_preprocessor::preprocess;

use crate::data::QuadInstance;
use crate::Gpu;

/// Creates the [`wgpu::RenderPipeline`] responsible for rendering axis-aligned quad instances.
///
/// # Attachments
///
/// This pipeline expects a single color attachment. Its format must be of `output_format`.
///
/// It also expects a depth attachment.
///
/// # Layout
///
/// The provided `layout` is expected to include bind groups for the following resources:
///
/// 0. `frame_uniforms` (bind group 0)
/// 1. `chunk_uniforms` (bind group 1)
pub fn create_shader(
    gpu: &Gpu,
    layout: &wgpu::PipelineLayout,
    output_format: wgpu::TextureFormat,
) -> wgpu::RenderPipeline {
    let shader_module = gpu
        .device
        .create_shader_module(wgpu::ShaderModuleDescriptor {
            label: Some("Quad Pipeline Shader Module"),
            source: wgpu::ShaderSource::Wgsl(
                preprocess!("crates/render/src/shaders/quad.wgsl").into(),
            ),
        });

    gpu.device
        .create_render_pipeline(&wgpu::RenderPipelineDescriptor {
            label: Some("Quad Pipeline"),
            layout: Some(layout),
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
