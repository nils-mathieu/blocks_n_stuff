use bns_core::ChunkPos;

/// Contains information about a chunk that's in the process of being generated.
pub struct ChunkGen {
    /// The position of the chun being generated.
    pos: ChunkPos,
}

impl ChunkGen {
    /// Creates a new [`ChunkGen`] with the provided position.
    #[inline]
    pub fn new(pos: ChunkPos) -> Self {
        Self { pos }
    }
}
