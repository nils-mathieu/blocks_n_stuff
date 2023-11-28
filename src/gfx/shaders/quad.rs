/// Creates the [`wgpu::RenderPipeline`] used to render axis-aligned quads to the screen.
///
/// # Color attachments
///
/// This pipeline uses a single output color attachment. Its format must be of `output_format`.
pub fn create(
    device: &wgpu::Device,
    instant_uniforms: &wgpu::BindGroupLayout,
    output_format: wgpu::TextureFormat,
) -> wgpu::RenderPipeline {
    let shader_module = device.create_shader_module(wgpu::ShaderModuleDescriptor {
        label: Some("Quad Pipeline Shader Module"),
        source: wgpu::ShaderSource::Wgsl(include_str!("quad.wgsl").into()),
    });

    // When multiple pipeline work on the same thing (i.e. custom geometry needing the same
    // uniforms), we'll probably benefit from using the same pipeline layout every time.
    // We can call that a "pipeline group"?
    let pipeline_layout = device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
        label: Some("Quad Pipeline Layout"),
        bind_group_layouts: &[&instant_uniforms],
        push_constant_ranges: &[],
    });

    device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
        label: Some("Quad Pipeline"),
        layout: Some(&pipeline_layout),
        vertex: wgpu::VertexState {
            buffers: &[],
            entry_point: "vs_main",
            module: &shader_module,
        },
        primitive: wgpu::PrimitiveState {
            conservative: false,
            cull_mode: None,
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
        depth_stencil: None,
        multisample: wgpu::MultisampleState {
            alpha_to_coverage_enabled: false,
            count: 1,
            mask: !0,
        },
        multiview: None,
    })
}
