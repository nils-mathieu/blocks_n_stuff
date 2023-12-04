use super::{get_chunk_alignment, ChunkUniforms, QuadInstance};
use crate::{Gpu, VertexBufferSlice};

/// A instance buffer that's ready to be rendered by the [`QuadPipeline`].
pub(super) struct QuadBuffer<'res> {
    /// The chunk uniforms that is associated with the quad instances in the buffer.
    ///
    /// This is the offset within the chunk uniforms buffer to use when setting the bind group.
    pub(super) chunk_idx: u32,
    /// The quad instances of the chunk.
    pub(super) slice: VertexBufferSlice<'res, QuadInstance>,
}

/// A collection type used to properly lay out [`QuadInstance`]s and [`ChunkUniforms`] in a buffer.
pub struct Quads<'res> {
    /// The alignment of the [`ChunkUniforms`] instances in the buffer.
    ///
    /// When a new instance is added to the buffer, it must be aligned to this value.
    pub(super) chunk_align: usize,
    /// The chunk uniforms that are used by the quads in the buffer.
    ///
    /// # Remarks
    ///
    /// This buffer is supposed to contain a bunch of [`ChunkUniforms`] instances. However, because
    /// it will be indexed using a dynamic offset, it's alignment depends on the minimum alignment
    /// available on the GPU. This means that the buffer may contain padding between the
    /// [`ChunkUniforms`] instances (and it will, because the minimum alignment varies between
    /// 64 and 256 bytes).
    pub(super) chunks: Vec<u8>,
    /// The quad instances that are used by the quads in the buffer.
    ///
    /// Those instances must be drawn using the opaque-specialized pipeline.
    pub(super) opaque_buffers: Vec<QuadBuffer<'res>>,
    /// The quad instances that are used by the quads in the buffer.
    ///
    /// Those instances must be drawn in order by the transparent-specialized pipeline.
    pub(super) transparent_buffers: Vec<QuadBuffer<'res>>,
}

impl<'res> Quads<'res> {
    /// Creates a new [`Quads`] instance.
    pub fn new(gpu: &Gpu) -> Self {
        Self {
            chunk_align: get_chunk_alignment(gpu),
            chunks: Vec::new(),
            opaque_buffers: Vec::new(),
            transparent_buffers: Vec::new(),
        }
    }

    /// Resets the [`Quads`] instance with a potentially longer lifetime, allowing it to be used
    /// again without having to reallocate the buffers.
    pub fn reset<'res2>(mut self) -> Quads<'res2> {
        self.opaque_buffers.clear();
        self.transparent_buffers.clear();

        // SAFETY:
        //  1. The buffer is empty, meaning that no references are actually being transmuted into
        //     a potentially longer lifetime.
        //  2. Two types that only differ in lifetime always have the same memory layout.
        let opaque_buffers = unsafe { std::mem::transmute(self.opaque_buffers) };
        let transparent_buffers = unsafe { std::mem::transmute(self.transparent_buffers) };

        Quads {
            chunk_align: self.chunk_align,
            chunks: self.chunks,
            opaque_buffers,
            transparent_buffers,
        }
    }

    /// Registers a [`ChunkUniforms`] instance to be used.
    pub fn register_chunk(&mut self, chunk: &ChunkUniforms) -> u32 {
        let index = self.chunks.len() / self.chunk_align;

        self.chunks.extend_from_slice(bytemuck::bytes_of(chunk));
        self.chunks.resize(self.chunk_align * (index + 1), 0);

        index as u32
    }

    /// Registers a new instance buffer that's ready to be rendered by the [`QuadPipeline`].
    ///
    /// The instances stored in the provided buffer are assumed to be opaque.
    pub fn register_opaque_quads(
        &mut self,
        chunk_idx: u32,
        quads: VertexBufferSlice<'res, QuadInstance>,
    ) {
        self.opaque_buffers.push(QuadBuffer {
            chunk_idx,
            slice: quads,
        });
    }

    /// Registers a new instance buffer that's ready to be rendered by the [`QuadPipeline`].
    ///
    /// The instances stored in the provided buffer are assumed to be transparent.
    pub fn register_transparent_quads(
        &mut self,
        chunk_idx: u32,
        quads: VertexBufferSlice<'res, QuadInstance>,
    ) {
        self.transparent_buffers.push(QuadBuffer {
            chunk_idx,
            slice: quads,
        });
    }
}
