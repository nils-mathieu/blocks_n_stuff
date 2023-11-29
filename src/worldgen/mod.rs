//! The standard world generator.

use crate::world::{BlockId, ChunkData, ChunkPos, LocalPos, WorldGenerator};

/// The standard [`WorldGenerator`] implementation.
pub struct StandardWorldGenerator {}

impl StandardWorldGenerator {
    /// Creates a new [`StandardWorldGenerator`].
    pub fn new() -> Self {
        Self {}
    }
}

impl WorldGenerator for StandardWorldGenerator {
    fn generate(&mut self, pos: ChunkPos) -> Box<ChunkData> {
        let mut result = ChunkData::empty();

        if pos.y == 0 {
            for pos in LocalPos::iter_surface(0) {
                result[pos] = BlockId::Stone;
            }
        }

        result
    }
}
