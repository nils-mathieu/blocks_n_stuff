use bitflags::bitflags;
use bns_core::LocalPos;
use bytemuck::{Pod, Zeroable};

use super::RenderResources;
use crate::gfx::Gpu;

bitflags! {
    /// Some flags that are stored in a [`QuadInstance`] to describe it.
    ///
    /// # Representation
    ///
    /// This bit set stores the following fields:
    ///
    /// | Bits  | Field      | Description                       |
    /// |-------|------------|-----------------------------------|
    /// | 0-2   | `facing`   | The direction the quad is facing. |
    /// | 3-4   | `rotate`   | The rotation of the quad.         |
    /// | 5     | `mirror_x` | Whether the quad is mirrored.     |
    /// | 6     | `mirror_y` | Whether the quad is mirrored.     |
    /// | 7-11  | `x`        | The local X position of the quad. |
    /// | 12-16 | `y`        | The local Y position of the quad. |
    /// | 17-21 | `z`        | The local Z position of the quad. |
    ///
    /// - `facing` can be one of the following values:
    ///
    ///   - `0b000`: The quad is facing the positive X axis.
    ///   - `0b001`: The quad is facing the negative X axis.
    ///   - `0b010`: The quad is facing the positive Y axis.
    ///   - `0b011`: The quad is facing the negative Y axis.
    ///   - `0b100`: The quad is facing the positive Z axis.
    ///   - `0b101`: The quad is facing the negative Z axis.
    ///
    /// - `rotate` can be one of the following values:
    ///   - `0b00`: The quad is not rotated.
    ///   - `0b01`: The quad is rotated 90 degrees clockwise.
    ///   - `0b10`: The quad is rotated 180 degrees clockwise.
    ///   - `0b11`: The quad is rotated 270 degrees clockwise.
    ///
    /// - `mirror_x`: whether the quad is mirrored along the X axis.
    /// - `mirror_y`: whether the quad is mirrored along the Y axis.
    ///
    /// - `x`, `y`, and `z` are the local position of the quad. They are stored as 5-bit unsigned
    ///   integers, which means that they can range from 0 to 31.
    #[derive(Debug, Clone, Copy)]
    #[repr(transparent)]
    pub struct QuadInstance: u32 {
        /// Indicates that the quad is facing the positive X axis.
        const X = 0b000;
        /// Indicates that the quad is facing the negative X axis.
        const NEG_X = 0b001;
        /// Indicates that the quad is facing the positive Y axis.
        const Y = 0b010;
        /// Indicates that the quad is facing the negative Y axis.
        const NEG_Y = 0b011;
        /// Indicates that the quad is facing the positive Z axis.
        const Z = 0b100;
        /// Indicates that the quad is facing the negative Z axis.
        const NEG_Z = 0b101;

        /// Indicates that the quad is not rotated.
        const ROTATE_0 = 0b00 << 3;
        /// Indicates that the quad is rotated 90 degrees clockwise.
        const ROTATE_90 = 0b01 << 3;
        /// Indicates that the quad is rotated 180 degrees clockwise.
        const ROTATE_180 = 0b10 << 3;
        /// Indicates that the quad is rotated 270 degrees clockwise.
        const ROTATE_270 = 0b11 << 3;

        /// Indicates that the quad is mirrored along the X axis.
        const MIRROR_X = 1 << 5;
        /// Indicates that the quad is mirrored along the Y axis.
        const MIRROR_Y = 1 << 6;

        /// The bits that are used to store the `x` field.
        ///
        /// This constant represents the value `31`.
        const X_MASK = 0b11111 << 7;
        /// The bits that are used to store the `y` field.
        ///
        /// This constant represents the value `31`.
        const Y_MASK = 0b11111 << 12;
        /// The bits that are used to store the `z` field.
        ///
        /// This constant represents the value `31`.
        const Z_MASK = 0b11111 << 17;
    }
}

impl QuadInstance {
    /// Creates a new [`QuadInstance`] from the provided local position.
    pub fn from_local_pos(local_pos: LocalPos) -> Self {
        let index = local_pos.index() as u32;
        Self::from_bits_retain(index << 7)
    }
}

unsafe impl Zeroable for QuadInstance {}
unsafe impl Pod for QuadInstance {}

/// Creates the [`wgpu::RenderPipeline`] used to render axis-aligned quads to the screen.
///
/// # Color attachments
///
/// This pipeline uses a single output color attachment. Its format must be of `output_format`.
///
/// # Layout
///
/// The layout of this pipeline is the [`RenderResources::world_pipeline_layout`] layout.
pub fn create(
    gpu: &Gpu,
    resources: &RenderResources,
    output_format: wgpu::TextureFormat,
) -> wgpu::RenderPipeline {
    let shader_module = gpu
        .device
        .create_shader_module(wgpu::ShaderModuleDescriptor {
            label: Some("Quad Pipeline Shader Module"),
            source: wgpu::ShaderSource::Wgsl(include_str!("quad.wgsl").into()),
        });

    gpu.device
        .create_render_pipeline(&wgpu::RenderPipelineDescriptor {
            label: Some("Quad Pipeline"),
            layout: Some(&resources.world_pipeline_layout),
            vertex: wgpu::VertexState {
                buffers: &[wgpu::VertexBufferLayout {
                    array_stride: std::mem::size_of::<QuadInstance>() as wgpu::BufferAddress,
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
                format: wgpu::TextureFormat::Depth32Float,
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
