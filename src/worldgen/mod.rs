//! The standard world generator.

use bns_core::{BlockId, Chunk, LocalPos};
use bns_rng::noises::Mixer;
use bns_rng::{FromRng, Noise};

use crate::world::{ChunkPos, WorldGenerator};

/// The standard [`WorldGenerator`] implementation.
#[derive(Debug, Clone, FromRng)]
pub struct StandardWorldGenerator {
    mixer: Mixer<2>,
}

impl WorldGenerator for StandardWorldGenerator {
    fn generate(&mut self, pos: ChunkPos) -> Chunk {
        let mut result = Chunk::empty();

        let height = (self.mixer.sample([pos.x as u64, pos.z as u64]) % 64) as i32;

        for local_pos in LocalPos::iter_all() {
            let block_pos = pos * Chunk::SIDE + local_pos.to_ivec3();

            if block_pos.y < height - 4 {
                *result.get_block_mut(local_pos) = BlockId::Stone;
            } else if block_pos.y < height {
                *result.get_block_mut(local_pos) = BlockId::Dirt;
            } else if block_pos.y == height {
                *result.get_block_mut(local_pos) = BlockId::Grass;
            }
        }

        result
    }
}
