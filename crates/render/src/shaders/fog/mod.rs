use crate::Gpu;

use super::common::CommonResources;

/// A simple render pipeline that renders fog using the depth buffer.
pub struct FogPipeline {
    /// The pipeline responsible for the fog.
    pipeline: wgpu::RenderPipeline,
}

impl FogPipeline {
    /// Creates a new [`FogPipeline`] instance.
    pub fn new(gpu: &Gpu, resources: &CommonResources, output_format: wgpu::TextureFormat) -> Self {
        let pipeline = create_shader(gpu, resources, output_format);
        Self { pipeline }
    }

    /// Renders the fog.
    #[profiling::function]
    pub fn render<'res>(&'res mut self, _gpu: &Gpu, rp: &mut wgpu::RenderPass<'res>) {
        rp.set_pipeline(&self.pipeline);
        rp.draw(0..4, 0..1);
    }
}

/// Creates a pipeline that's responsible for rendering the fog.
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
    resources: &CommonResources,
    output_format: wgpu::TextureFormat,
) -> wgpu::RenderPipeline {
    let shader_module = gpu
        .device
        .create_shader_module(wgpu::ShaderModuleDescriptor {
            label: Some("Fog Shader Module"),
            source: wgpu::ShaderSource::Wgsl(include_str!("fog.wgsl").into()),
        });

    let pipeline_layout = gpu
        .device
        .create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
            label: Some("Fog Pipeline Layout"),
            bind_group_layouts: &[
                &resources.frame_uniforms_layout,
                &resources.depth_buffer_layout,
            ],
            push_constant_ranges: &[],
        });

    gpu.device
        .create_render_pipeline(&wgpu::RenderPipelineDescriptor {
            label: Some("Fog Pipeline"),
            vertex: wgpu::VertexState {
                buffers: &[],
                entry_point: "vs_main",
                module: &shader_module,
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
