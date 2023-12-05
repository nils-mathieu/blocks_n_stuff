use bytemuck::{Pod, Zeroable};
use glam::{Mat2, Vec2};

use crate::color::Color;

/// A sprite instance that's passed to the GPU.
#[derive(Debug, Clone, Copy, Zeroable, Pod)]
#[repr(C)]
pub struct Sprite {
    /// The transformation of the sprite's UVs.
    pub uv_transform: Mat2,
    /// The transformation of the sprite.
    pub transform: Mat2,
    /// The position of the sprite.
    pub position: Vec2,
    /// The offset of the sprite's UVs.
    pub uv_offset: Vec2,
    /// A color to multiply the sprite's base color with.
    pub color: Color,
    /// Some padding bytes.
    pub _padding: [u32; 3],
}

impl Sprite {
    /// Creates a new dummy [`Sprite`] instance.
    pub fn dummy() -> Self {
        Self {
            uv_transform: Mat2::IDENTITY,
            transform: Mat2::IDENTITY,
            position: Vec2::ZERO,
            uv_offset: Vec2::ZERO,
            color: Color::WHITE,
            _padding: [0; 3],
        }
    }

    /// Returns the [`Sprite`] with a different UV rectangle.
    pub fn with_uv_rect(mut self, pos: Vec2, size: Vec2) -> Self {
        self.uv_transform = Mat2::from_cols(Vec2::new(size.x, 0.0), Vec2::new(0.0, size.y));
        self.uv_offset = pos;
        self
    }

    /// Returns the [`Sprite`] with a different position and size.
    pub fn with_rect(mut self, pos: Vec2, size: Vec2) -> Self {
        self.transform = Mat2::from_cols(Vec2::new(size.x, 0.0), Vec2::new(0.0, size.y));
        self.position = pos;
        self
    }
}
