use bitflags::bitflags;
use glam::IVec2;

bitflags! {
    /// Some flags associated with a [`QuadInstance`].
    #[derive(Debug, Clone, Copy)]
    #[repr(transparent)]
    pub struct QuadInstanceFlags: u32 {
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
    }
}

/// The instance data passed to shaders.
#[repr(C)]
#[derive(Debug, Clone, Copy)]
pub struct QuadInstance {
    /// The position of the quad in the world.
    pub position: IVec2,
    /// Some flags associated with the quad.
    pub flags: QuadInstanceFlags,
}
