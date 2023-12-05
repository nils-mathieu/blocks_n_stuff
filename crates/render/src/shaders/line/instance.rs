use bitflags::bitflags;
use bytemuck::{Pod, Zeroable};
use glam::Vec3;

use crate::color::Color;

bitflags! {
    /// Some flags associated with a [`LineVertex`].
    #[derive(Debug, Clone, Copy)]
    #[repr(transparent)]
    pub struct LineFlags: u32 {
        /// Whether the line should appear above all geometry in the world.
        const ABOVE = 1 << 0;
    }
}

unsafe impl Zeroable for LineFlags {}
unsafe impl Pod for LineFlags {}

/// A vertex that's used to construct a line.
#[repr(C)]
#[derive(Debug, Clone, Copy, Zeroable, Pod)]
pub struct LineInstance {
    /// The start position of the vertex, in world space.
    pub start: Vec3,
    /// The width of the line.
    pub width: f32,
    /// The end position of the vertex, in world space.
    pub end: Vec3,
    /// Some flags associated with the line.
    pub flags: LineFlags,
    /// The color of the vertex.
    pub color: Color,
}
