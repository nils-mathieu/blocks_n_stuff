//! The standard world generator.

use bns_core::{BlockId, Chunk, LocalPos};

use crate::world::{ChunkPos, WorldGenerator};

/// The standard [`WorldGenerator`] implementation.
pub struct StandardWorldGenerator {}

impl StandardWorldGenerator {
    /// Creates a new [`StandardWorldGenerator`].
    pub fn new() -> Self {
        Self {}
    }
}

impl WorldGenerator for StandardWorldGenerator {
    fn generate(&mut self, pos: ChunkPos) -> Chunk {
        let mut result = Chunk::empty();

        if pos.y == 0 {
            for pos in LocalPos::iter_surface(0) {
                *result.get_block_mut(pos) = BlockId::Stone;
            }
        }

        result
    }
}
