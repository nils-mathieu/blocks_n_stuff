use std::borrow::Cow;

use crate::Gpu;

/// Creates a pipeline that's responsible for rendering the skybox.
///
/// # Arrachments
///
/// This pipeline expects a single color attachment. Its format must be of `output_format`.
///
///
/// # Layout
///
/// The provided `frame_uniforms_layout` is expected to include the bind group for the
/// frame uniforms.
pub fn create_shader(
    gpu: &Gpu,
    frame_uniforms_layout: &wgpu::BindGroupLayout,
    output_format: wgpu::TextureFormat,
) -> wgpu::RenderPipeline {
    let shader_module = gpu
        .device
        .create_shader_module(wgpu::ShaderModuleDescriptor {
            label: Some("Skybox Shader Module"),
            source: wgpu::ShaderSource::Wgsl(Cow::Borrowed(include_str!("skybox.wgsl"))),
        });

    let pipeline_layout = gpu
        .device
        .create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
            label: Some("Skybox Pipeline Layout"),
            bind_group_layouts: &[frame_uniforms_layout],
            push_constant_ranges: &[],
        });

    gpu.device
        .create_render_pipeline(&wgpu::RenderPipelineDescriptor {
            label: Some("Skybox Pipeline"),
            vertex: wgpu::VertexState {
                buffers: &[],
                entry_point: "vs_main",
                module: &shader_module,
            },
            fragment: Some(wgpu::FragmentState {
                entry_point: "fs_main",
                module: &shader_module,
                targets: &[Some(wgpu::ColorTargetState {
                    blend: None,
                    format: output_format,
                    write_mask: wgpu::ColorWrites::ALL,
                })],
            }),
            depth_stencil: Some(wgpu::DepthStencilState {
                format: crate::DEPTH_FORMAT,
                depth_write_enabled: false,
                depth_compare: wgpu::CompareFunction::Always,
                stencil: wgpu::StencilState::default(),
                bias: wgpu::DepthBiasState::default(),
            }),
            layout: Some(&pipeline_layout),
            multisample: wgpu::MultisampleState {
                alpha_to_coverage_enabled: false,
                count: 1,
                mask: !0,
            },
            multiview: None,
            primitive: wgpu::PrimitiveState {
                conservative: false,
                cull_mode: None,
                front_face: wgpu::FrontFace::Ccw,
                polygon_mode: wgpu::PolygonMode::Fill,
                strip_index_format: None,
                topology: wgpu::PrimitiveTopology::TriangleStrip,
                unclipped_depth: false,
            },
        })
}
