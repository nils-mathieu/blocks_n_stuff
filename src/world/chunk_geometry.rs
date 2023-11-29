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

    /// BUilds the geometry of this chunk based on its content.
    ///
    /// Note that the neighboring chunks are *not* taken into account for culling, and the outer
    /// faces of the chunk are never built.
    pub fn build_inner(&mut self, data: &Chunk, context: &mut ChunkBuildContext) {
        context.reset();

        // TODO: actually perform some culling.
        for local_pos in LocalPos::iter_all() {
            match data.get_block(local_pos).info().appearance {
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
