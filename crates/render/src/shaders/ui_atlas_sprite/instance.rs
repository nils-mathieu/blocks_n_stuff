use std::mem::size_of;

use bytemuck::{Pod, Zeroable};
use glam::{Mat2, Vec2};

use crate::color::Color;

/// A sprite that's sampled from the global texture atlas (the one used for quads).
#[derive(Debug, Clone, Copy, Zeroable, Pod)]
#[repr(C)]
pub struct AtlasSprite {
    /// The transformation matrix of the sprite.
    pub transform: Mat2,
    /// The position of the sprite.
    pub position: Vec2,
    /// The index of the texture in the atlas.
    pub texture_id: u32,
    /// The color of the sprite.
    pub color: Color,
}

impl AtlasSprite {
    pub(crate) fn layout() -> wgpu::VertexBufferLayout<'static> {
        wgpu::VertexBufferLayout {
            array_stride: size_of::<Self>() as wgpu::BufferAddress,
            step_mode: wgpu::VertexStepMode::Instance,
            attributes: &[
                wgpu::VertexAttribute {
                    offset: 0,
                    shader_location: 0,
                    format: wgpu::VertexFormat::Float32x4,
                },
                wgpu::VertexAttribute {
                    offset: 16,
                    shader_location: 1,
                    format: wgpu::VertexFormat::Float32x2,
                },
                wgpu::VertexAttribute {
                    offset: 24,
                    shader_location: 2,
                    format: wgpu::VertexFormat::Uint32,
                },
                wgpu::VertexAttribute {
                    offset: 28,
                    shader_location: 3,
                    format: wgpu::VertexFormat::Uint32,
                },
            ],
        }
    }

    /// Creates a new dummy [`AtlasSprite`] instance.
    pub fn dummy() -> Self {
        Self {
            transform: Mat2::ZERO,
            position: Vec2::ZERO,
            color: Color::WHITE,
            texture_id: 0,
        }
    }

    /// Returns the [`AtlasSprite`] with a different position and size.
    pub fn with_rect(mut self, pos: Vec2, size: Vec2) -> Self {
        self.transform = Mat2::from_cols(Vec2::new(size.x, 0.0), Vec2::new(0.0, size.y));
        self.position = pos;
        self
    }

    /// Returns the [`AtlasSprite`] with a different color.
    pub fn with_color(mut self, color: Color) -> Self {
        self.color = color;
        self
    }
}
