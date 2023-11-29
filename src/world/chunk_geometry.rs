use std::sync::Arc;

use bns_core::{BlockAppearance, Chunk, LocalPos};
use bns_render::data::QuadInstance;
use bns_render::{DynamicVertexBuffer, Gpu};

/// Contains some resources useful for building a chunk.
///
/// This mostly includes temporary buffers.
pub struct ChunkBuildContext {
    gpu: Arc<Gpu>,
    quads: Vec<QuadInstance>,
}

impl ChunkBuildContext {
    /// Creates a new [`ChunkBuildContext`].
    pub fn new(gpu: Arc<Gpu>) -> Self {
        Self {
            gpu,
            quads: Vec::new(),
        }
    }

    /// Resets the context.
    #[inline]
    fn reset(&mut self) {
        self.quads.clear();
    }
}

/// The built geometry of a chunk. This is a wrapper around a vertex buffer that
/// contains the quad instances of the chunk.
pub struct ChunkGeometry {
    /// The quad instances of the chunk.
    ///
    /// When `None`, the vertex buffer has not been created.
    pub quads: Option<DynamicVertexBuffer<QuadInstance>>,
}

impl ChunkGeometry {
    /// Creates a new [`ChunkGeometry`] instance.
    pub fn new() -> Self {
        Self { quads: None }
    }

    /// Builds the geometry of a chunk.
    pub fn build(&mut self, neighborhood: ChunkNeighborhood, context: &mut ChunkBuildContext) {
        context.reset();

        // TODO: actually perform some culling.
        for local_pos in LocalPos::iter_all() {
            match neighborhood.this.get_block(local_pos).info().appearance {
                BlockAppearance::Invisible => (),
                BlockAppearance::Regular { top, bottom, side } => {
                    context.quads.extend_from_slice(&[
                        QuadInstance::from_texture(side as u32)
                            | QuadInstance::X
                            | QuadInstance::from_chunk_index(local_pos.index()),
                        QuadInstance::from_texture(side as u32)
                            | QuadInstance::NEG_X
                            | QuadInstance::from_chunk_index(local_pos.index()),
                        QuadInstance::from_texture(top as u32)
                            | QuadInstance::Y
                            | QuadInstance::from_chunk_index(local_pos.index()),
                        QuadInstance::from_texture(bottom as u32)
                            | QuadInstance::NEG_Y
                            | QuadInstance::from_chunk_index(local_pos.index()),
                        QuadInstance::from_texture(side as u32)
                            | QuadInstance::Z
                            | QuadInstance::from_chunk_index(local_pos.index()),
                        QuadInstance::from_texture(side as u32)
                            | QuadInstance::NEG_Z
                            | QuadInstance::from_chunk_index(local_pos.index()),
                    ]);
                }
            }
        }

        if context.quads.is_empty() {
            self.quads = None;
        } else {
            self.quads
                .get_or_insert_with(|| {
                    DynamicVertexBuffer::new(context.gpu.clone(), context.quads.len() as u32)
                })
                .replace(&context.quads);
        }
    }
}

/// The neighborhood of a chunk.
pub struct ChunkNeighborhood<'a> {
    /// The data of the chunk that's being built.
    pub this: &'a Chunk,
    /// The chunk that's on the positive X side of this chunk.
    pub x: &'a Chunk,
    /// The chunk that's on the negative X side of this chunk.
    pub nx: &'a Chunk,
    /// The chunk that's on the positive Y side of this chunk.
    pub y: &'a Chunk,
    /// The chunk that's on the negative Y side of this chunk.
    pub ny: &'a Chunk,
    /// The chunk that's on the positive Z side of this chunk.
    pub z: &'a Chunk,
    /// The chunk that's on the negative Z side of this chunk.
    pub nz: &'a Chunk,
}
