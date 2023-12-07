use bitflags::bitflags;
use bytemuck::{Pod, Zeroable};
use glam::IVec3;

/// A quad instance, as sent to the GPU.
///
#[derive(Debug, Clone, Copy, Zeroable, Pod)]
#[repr(C)]
pub struct QuadInstance {
    /// Some flags associated with this instance.
    pub flags: QuadFlags,
    /// The index of the texture to use for this quad.
    pub texture: u32,
}

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
    /// | 22-24 | `offset`   | The offset of the block.          |
    /// | 25-28 | `occluded` | Whether the quad is occluded.     |
    /// | 29    | `overlay`  | Whether the quad is an overlay.   |
    /// | 30    | `liquid`   | Whether it's a liquid quad.       |
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
    /// - `offset` is an offset applied to the face, by increments of 1/8th of a block. The offset
    ///   is in the direction opposite to the face's normal. For example, a face facing the
    ///   positive X axis with an offset of 1 will be pushed back by 1/8th of a block along the
    ///   negative X axis.
    ///
    /// - `occluded` is a bitfleld that describes which sides of the quad are occldued by other
    ///    neighboring blocks. The bits are stored in the following order:
    ///    - `0b0001`: The "top" of the quad is occluded.
    ///    - `0b0010`: The "bottom" of the quad is occluded.
    ///    - `0b0100`: The "left" of the quad is occluded.
    ///    - `0b1000`: The "right" of the quad is occluded.
    ///
    /// - `overlay`: whether the quad is an overlay. If this bit is set, the quad will be rendered
    ///   with a slight offset in the direction of its normal.
    ///
    /// - `liquid`: whether the quad is a liquid quad. If this bit is set, the quad will be used
    ///   when rendering underwater fog and reflections. It will also be subject to animation.
    #[derive(Debug, Clone, Copy)]
    #[repr(transparent)]
    pub struct QuadFlags: u32 {
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

        /// The quad is not offset from its original position.
        const OFFSET_0 = 0b000 << 22;
        /// The quad is pushed back 1/8th of a block.
        const OFFSET_1 = 0b001 << 22;
        /// The quad is pushed back 2/8th of a block.
        const OFFSET_2 = 0b010 << 22;
        /// The quad is pushed back 3/8th of a block.
        const OFFSET_3 = 0b011 << 22;
        /// The quad is pushed back 4/8th of a block.
        const OFFSET_4 = 0b100 << 22;
        /// The quad is pushed back 5/8th of a block.
        const OFFSET_5 = 0b101 << 22;
        /// The quad is pushed back 6/8th of a block.
        const OFFSET_6 = 0b110 << 22;
        /// The quad is pushed back 7/8th of a block.
        const OFFSET_7 = 0b111 << 22;

        /// The bits that are used to store the offset of the quad.
        ///
        /// This constant value represents the value 7.
        const OFFSET_MASK = 0b111 << 22;

        /// Whether the "top" of the quad is occluded.
        const OCCLUDED_TOP = 0b0001 << 25;
        /// Whether the "bottom" of the quad is occluded.
        const OCCLUDED_BOTTOM = 0b0010 << 25;
        /// Whether the "left" of the quad is occluded.
        const OCCLUDED_LEFT = 0b0100 << 25;
        /// Whether the "right" of the quad is occluded.
        const OCCLUDED_RIGHT = 0b1000 << 25;

        /// The bits that are used to store the occlusion of the quad.
        ///
        /// This constant represents the value `15`.
        const OCCLUDED_MASK = 0b1111 << 25;

        /// The quad is not an overlay.
        ///
        /// When set, the quad will be rendered with a slight offset in the direction of its normal.
        const OVERLAY = 1 << 29;

        /// Whether the quad is a liquid quad.
        ///
        /// When set, the quad will be used when rendering underwater fog and reflections. It will
        /// also be subject to animation.
        const LIQUID = 1 << 30;
    }
}

impl QuadFlags {
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
}

unsafe impl Zeroable for QuadFlags {}
unsafe impl Pod for QuadFlags {}

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
