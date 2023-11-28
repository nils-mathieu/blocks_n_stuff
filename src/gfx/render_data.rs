use bytemuck::{Pod, Zeroable};
use glam::Mat4;

pub use super::helpers::{UniformBuffer, VertexBuffer};
pub use super::shaders::quad::QuadInstance;

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

/// Contains all the state that needs to be drawn to the screen.
pub struct RenderData<'a> {
    /// The uniforms that are supposed to be overwritten every frame.
    pub frame_uniforms: &'a UniformBuffer<FrameUniforms>,
    /// A collection of quads to draw.
    pub quads: &'a VertexBuffer<QuadInstance>,
}
