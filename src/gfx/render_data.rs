use bytemuck::{Pod, Zeroable};
use glam::{Mat4, Vec4};

use crate::world::ChunkPos;

pub use super::shaders::quad::QuadInstance;

/// A view into an existing buffer.
#[derive(Debug, Clone, Copy)]
pub struct BufferSlice<'a> {
    /// The buffer that the slice is a part of.
    pub buffer: &'a wgpu::Buffer,
    /// The number of bytes between the start of the buffer and the end of the slice.
    pub len: u32,
}

impl<'a> BufferSlice<'a> {
    /// Creates a new [`BufferSlice`] instance.
    pub fn new(buffer: &'a wgpu::Buffer, len: u32) -> Self {
        Self { buffer, len }
    }
}

/// The uniforms that are modified every frame.
///
/// This includes the camera matrix, among other things.
#[repr(C)]
#[derive(Debug, Clone, Copy, Zeroable, Pod)]
pub struct FrameUniforms {
    /// The camera matrix, responsible for transforming world-space coordinates into clip-space
    /// coordinates.
    ///
    /// This includes the projection matrix and the view matrix.
    pub camera: Mat4,
}

/// Contains information about a chunk.
///
/// This uniform is updated between each chunk draw call.
#[derive(Debug, Clone, Copy, Zeroable, Pod)]
#[repr(C)]
pub struct ChunkUniforms {
    /// The position of the chunk.
    pub position: ChunkPos,

    // We need to pad the type to make it 64 bytes long because that's the minimum dynamic offset
    // alignment.
    pub _padding: [u32; 13],
}

/// Contains all the state that needs to be drawn to the screen.
pub struct RenderData<'a> {
    /// The color that the output image should be cleared to.
    pub clear_color: Vec4,
    /// The value of [`FrameUniforms`] for this frame.
    pub frame_uniforms: FrameUniforms,
    /// The chunk uniforms.
    ///
    /// This must have the same length as `quads`.
    pub chunk_uniforms: &'a [ChunkUniforms],
    /// A collection of quads to draw.
    pub quads: &'a [BufferSlice<'a>],
}
