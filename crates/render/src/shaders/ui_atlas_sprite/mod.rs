use crate::{Gpu, VertexBufferSlice};

mod instance;
pub use instance::*;

use super::common::CommonResources;

/// The render pipeline responsible for rendering sprites in the UI using the global texture atlas.
pub struct UiAtlasSpritePipeline {
    /// The render pipeline object.
    pipeline: wgpu::RenderPipeline,
}

impl UiAtlasSpritePipeline {
    /// Creates a new [`UiSpritePipeline`] instance.
    pub fn new(gpu: &Gpu, output_format: wgpu::TextureFormat) -> Self {
        let pipeline = create_pipeline(gpu, output_format);
        Self { pipeline }
    }

    /// Rendders the provided collection of sprite instances.
    pub fn render<'res>(
        &'res self,
        res: &'res CommonResources,
        rp: &mut wgpu::RenderPass<'res>,
        instances: VertexBufferSlice<'res, AtlasSprite>,
    ) {
        rp.set_pipeline(&self.pipeline);
        rp.set_vertex_buffer(0, instances.buffer);
        rp.set_bind_group(1, &res.texture_atlas_bind_group, &[]);
        rp.draw(0..4, 0..instances.len);
    }
}

fn create_pipeline(gpu: &Gpu, output_format: wgpu::TextureFormat) -> wgpu::RenderPipeline {
    let res = gpu.resources.read();

    let shader_module = gpu
        .device
        .create_shader_module(wgpu::ShaderModuleDescriptor {
            label: Some("UI Atlas Sprite Shader"),
            source: wgpu::ShaderSource::Wgsl(include_str!("ui_atlas_sprite.wgsl").into()),
        });

    let pipeline_layout = gpu
        .device
        .create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
            bind_group_layouts: &[&res.frame_uniforms_layout, &res.texture_atlas_layout],
            push_constant_ranges: &[],
            label: Some("UI Atlas Sprite Pipeline Layout"),
        });

    gpu.device
        .create_render_pipeline(&wgpu::RenderPipelineDescriptor {
            label: Some("UI Atlas Sprite Pipeline"),
            layout: Some(&pipeline_layout),
            vertex: wgpu::VertexState {
                module: &shader_module,
                entry_point: "vs_main",
                buffers: &[AtlasSprite::layout()],
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
