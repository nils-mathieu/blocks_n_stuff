use bitflags::bitflags;
use bytemuck::{Pod, Zeroable};
use glam::{IVec3, Mat4};

use crate::{Renderer, Vertices};

/// A lifetime-erased storage for [`RenderData`].
///
/// Using this type allows creating a [`RenderData`] instance without having to reallocate buffers
/// every frame.
pub struct RenderDataStorage {
    chunk_uniforms_align: u32,
    chunk_uniforms: Vec<u8>,
    quad_vertices: Vec<QuadVertices<'static>>,
}

impl RenderDataStorage {
    /// Creates a new [`RenderDataStorage`] instance.
    pub fn new(renderer: &Renderer) -> Self {
        Self {
            chunk_uniforms_align: renderer.chunk_uniforms_alignment,
            chunk_uniforms: Vec::new(),
            quad_vertices: Vec::new(),
        }
    }

    /// Creates a new [`RenderData`] instance using the data stored in the storage.
    ///
    /// This avoids creating new allocations.
    pub fn build<'a, 'res>(&'a mut self) -> RenderData<'a, 'res>
    where
        'a: 'res,
    {
        // Clearing those vectors is necessary for the function to be sound!
        // We need to make sure that no data is left in the vectors from the previous frame, as
        // it might now be invalid.
        //
        // We know that the `'res` lifetime we're providing won't exceed ours ('a).
        self.chunk_uniforms.clear();
        self.quad_vertices.clear();

        RenderData {
            chunk_uniforms_align: self.chunk_uniforms_align,
            chunk_uniforms: &mut self.chunk_uniforms,
            frame_uniforms: FrameUniforms::default(),
            clear_color: [0.0; 4],
            quad_vertices: unsafe { std::mem::transmute(&mut self.quad_vertices) },
        }
    }
}

/// Represents an instance buffer to be rendered.
pub(crate) struct QuadVertices<'res> {
    /// The index of the chunk that the quads in the buffer belong to.
    pub chunk_index: u32,
    /// The quad instances of the chunk.
    ///
    /// This is expected to be a collection of `len` instances of [`QuadInstance`].
    pub vertices: wgpu::BufferSlice<'res>,
    /// The number of [`QuadInstance`] instances in the buffer slice.
    pub len: u32,
}

/// The uniform data that is uploaded to the GPU once per frame.
#[derive(Debug, Default, Clone, Copy, Zeroable, Pod)]
#[repr(C)]
pub struct FrameUniforms {
    /// The projection and view matrix of the camera.
    pub camera: Mat4,
}

/// Contains information about a chunk.
///
/// This uniform is uploaded to the GPU once per frame, but it has to be rebound every time a
/// chunk is rendered with a dynamic offset.
#[derive(Debug, Clone, Copy, Zeroable, Pod)]
#[repr(C)]
pub struct ChunkUniforms {
    /// The position of the chunk.
    pub position: IVec3,
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

        /// The bits that are used to store the `texture` field.
        ///
        /// This constant represents the value `1023`.
        const TEXTURE_MASK = 0b1111111111 << 22;
    }
}

impl QuadInstance {
    /// Creates a new [`QuadInstance`] from the provided local X position.
    ///
    /// # Panics
    ///
    /// This function panics if `x` is greater than or equal to 32.
    #[inline]
    #[track_caller]
    pub fn from_x(x: u32) -> Self {
        debug_assert!(x < 32);
        Self::from_bits_retain(x << 7)
    }

    /// Creates a new [`QuadInstance`] from the provided local Y position.
    ///
    /// # Panics
    ///
    /// This function panics if `y` is greater than or equal to 32.
    #[inline]
    #[track_caller]
    pub fn from_y(y: u32) -> Self {
        debug_assert!(y < 32);
        Self::from_bits_retain(y << 12)
    }

    /// Creates a new [`QuadInstance`] from the provided local Z position.
    ///
    /// # Panics
    ///
    /// This function panics if `z` is greater than or equal to 32.
    #[inline]
    #[track_caller]
    pub fn from_z(z: u32) -> Self {
        debug_assert!(z < 32);
        Self::from_bits_retain(z << 17)
    }

    /// Creates a new [`QuadInstance`] from the provided local position.
    ///
    /// # Panics
    ///
    /// This function panics if any of the provided coordinates are out of bounds.
    #[inline]
    #[track_caller]
    pub fn from_chunk_index(index: usize) -> Self {
        debug_assert!(index < 32 * 32 * 32);
        Self::from_bits_retain((index as u32) << 7)
    }

    /// Creates a new [`QuadInstance`] from the provided [`TextureId`].
    ///
    /// # Panics
    ///
    /// This function panics in debug builds if `texture` is greater than or equal to 1024.
    #[inline]
    #[track_caller]
    pub fn from_texture(texture: u32) -> Self {
        assert!(texture < 1024);
        Self::from_bits_retain(texture << 22)
    }
}

unsafe impl Zeroable for QuadInstance {}
unsafe impl Pod for QuadInstance {}

/// The data required to render a frame.
///
/// An instance of this type can be created using the [`RenderDataStorage`] type.
pub struct RenderData<'a, 'res> {
    /// The target alignment of the chunk uniforms.
    ///
    /// `chunk_uniforms` is supposed to always be aligned to this value.
    pub(crate) chunk_uniforms_align: u32,
    /// Contains the chunk uniform data.
    ///
    /// This should be thought of as a [`ChunksUniform`] array.
    ///
    /// The reason we need to store this as a byte array is because the alignment of the dynamic
    /// offsets within a bind group is not known at compile time. For this reason, we need to
    /// manually compute the offset and write the data to the buffer ourselvse.
    pub(crate) chunk_uniforms: &'a mut Vec<u8>,

    /// The frame uniforms for the frame.
    pub(crate) frame_uniforms: FrameUniforms,
    /// The clear color for the frame.
    pub(crate) clear_color: [f64; 4],

    /// The vertices of the quads to render.
    ///
    /// The buffer slices in this array are expected to point to an instance buffer containing
    /// instances of [`QuadInstance`].
    pub(crate) quad_vertices: &'a mut Vec<QuadVertices<'res>>,
}

impl<'a, 'res> RenderData<'a, 'res> {
    /// Inserts a new [`QuadVertices`] instance into the render data.
    pub fn add_quad_vertices(
        &mut self,
        chunk: ChunkUniforms,
        vertices: &'res impl Vertices<Vertex = QuadInstance>,
    ) {
        let len = vertices.len();

        if len == 0 {
            return;
        }

        let chunk_index = self.chunk_uniforms.len() / self.chunk_uniforms_align as usize;
        self.chunk_uniforms
            .extend_from_slice(bytemuck::bytes_of(&chunk));
        // Extend the chunk uniforms to the next alignment.
        self.chunk_uniforms
            .resize(self.chunk_uniforms_align as usize * (chunk_index + 1), 0);

        self.quad_vertices.push(QuadVertices {
            chunk_index: chunk_index as u32,
            vertices: vertices.slice(),
            len,
        });
    }

    /// Sets the clear color for the frame.
    #[inline]
    pub fn clear_color(&mut self, value: [f64; 4]) {
        self.clear_color = value;
    }

    /// Set the [`FrameUniforms`] instance for the frame.
    #[inline]
    pub fn frame_uniforms(&mut self, value: FrameUniforms) {
        self.frame_uniforms = value;
    }
}
