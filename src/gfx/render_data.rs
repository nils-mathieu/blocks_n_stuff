use bytemuck::{Pod, Zeroable};
use glam::Mat4;

/// The uniforms that are modified every frame.
///
/// This includes the camera matrix, among other things.
#[repr(C)]
#[derive(Debug, Clone, Copy, Zeroable, Pod)]
pub struct InstantUniforms {
    /// The camera matrix, responsible for transforming world-space coordinates into clip-space
    /// coordinates.
    ///
    /// This includes the projection matrix and the view matrix.
    pub camera: Mat4,
}

/// Contains all the state that needs to be drawn to the screen.
pub struct RenderData {
    /// The uniforms that are shared across a single frame.
    pub instant_uniforms: InstantUniforms,
}
