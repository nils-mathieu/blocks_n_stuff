use bitflags::bitflags;
use bytemuck::{Pod, Zeroable};
use glam::IVec3;

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
    /// | 22-31 | `texture`  | The index of the quad's texture.  |
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
    ///
    /// - `texture` is the index of the quad's texture. It is stored as a 10-bit unsigned integer,
    ///   which means that it can range from 0 to 1023.
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

        /// The bits that are used to store the index of the voxel within its chunk.
        const CHUNK_INDEX_MASK = Self::X_MASK.bits() | Self::Y_MASK.bits() | Self::Z_MASK.bits();

        /// The bits that are used to store the `texture` field.
        ///
        /// This constant represents the value `1023`.
        const TEXTURE_MASK = 0b1111111111 << 22;
    }
}

impl QuadInstance {
    /// Creates a new [`QuadInstance`] from the provided local X position.
    ///
    /// # Remarks
    ///
    /// This function may return an invalid value if `x` is greater than or equal to
    /// [`Chunk::SIDE`].
    #[inline]
    #[track_caller]
    pub fn from_x(x: i32) -> Self {
        Self::from_bits_retain((x as u32) << 7)
    }

    /// Creates a new [`QuadInstance`] from the provided local Y position.
    ///
    /// # Remarks
    ///
    /// This function may return an invalid value if `y` is greater than or equal to
    /// [`Chunk::SIDE`].
    #[inline]
    #[track_caller]
    pub fn from_y(y: i32) -> Self {
        Self::from_bits_retain((y as u32) << 12)
    }

    /// Creates a new [`QuadInstance`] from the provided local Z position.
    ///
    /// # Remarks
    ///
    /// This function may return an invalid value if `z` is greater than or equal to
    /// [`Chunk::SIDE`].
    #[inline]
    #[track_caller]
    pub fn from_z(z: i32) -> Self {
        Self::from_bits_retain((z as u32) << 17)
    }

    /// Creates a new [`QuadInstance`] from the provided local position.
    ///
    /// # Remarks
    ///
    /// This function may return an invalid value if the provided index is greater than
    /// [`Chunk::SIZE`].
    #[inline]
    #[track_caller]
    pub fn from_chunk_index(index: usize) -> Self {
        Self::from_bits_retain((index as u32) << 7)
    }

    /// The maximum number of textures that can be represented by a [`QuadInstance`].
    pub const MAX_TEXTURES: u32 = 1024;

    /// Creates a new [`QuadInstance`] from the provided texture index.
    ///
    /// # Remarks
    ///
    /// This function may return an invalid value if the provided texture index is larger
    /// than [`QuadInstance::MAX_TEXTURES`].
    #[inline]
    #[track_caller]
    pub fn from_texture(texture: u32) -> Self {
        Self::from_bits_retain(texture << 22)
    }
}

unsafe impl Zeroable for QuadInstance {}
unsafe impl Pod for QuadInstance {}

/// Contains information about a chunk.
///
/// This uniform is uploaded to the GPU once per frame, but it has to be rebound every time a
/// chunk is rendered with a dynamic offset.
#[derive(Debug, Clone, Copy, Zeroable, Pod)]
#[repr(C)]
pub struct ChunkUniforms {
    /// The position of the chunk, in world-space.
    pub position: IVec3,
}
