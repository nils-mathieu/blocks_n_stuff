use std::sync::OnceLock;

use bns_core::ChunkPos;

/// Contains information about a chunk that's in the process of being generated.
pub struct ChunkGen {
    /// The position of the chun being generated.
    pos: ChunkPos,
    /// When set, indicates that the chunk has requested the structures that it needs to spawn
    /// from nearby biomes.
    structures: OnceLock<()>,
}

impl ChunkGen {
    /// Creates a new [`ChunkGen`] with the provided position.
    #[inline]
    pub fn new(pos: ChunkPos) -> Self {
        Self {
            pos,
            structures: OnceLock::new(),
        }
    }

    /// Ensures that the chunk has has requested the structures that it needs to spawn.
    pub fn structures(&self) {
        self.structures.get_or_init(|| {
            let _ = self.pos;
        });
    }
}
