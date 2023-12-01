//! The standard world generator.

use bns_core::{BlockId, Chunk, LocalPos};
use bns_rng::noises::{Mixer, Voronoi};
use bns_rng::{FromRng, Noise, Rng};

use crate::world::{ChunkPos, WorldGenerator};

/// The standard [`WorldGenerator`] implementation.
#[derive(Clone)]
pub struct StandardWorldGenerator {
    voronoi: Voronoi,
    height: Mixer<2>,
}

impl FromRng for StandardWorldGenerator {
    fn from_rng(rng: &mut impl Rng) -> Self {
        Self {
            voronoi: Voronoi::from_rng(rng),
            height: Mixer::from_rng(rng),
        }
    }
}

impl WorldGenerator for StandardWorldGenerator {
    fn generate(&mut self, chunk_pos: ChunkPos) -> Chunk {
        let mut ret = Chunk::empty();

        for local_pos in LocalPos::iter_all() {
            let world_pos = chunk_pos * Chunk::SIDE + local_pos.to_ivec3();
            let noise = self
                .voronoi
                .sample([world_pos.x as f32 / 32.0, world_pos.z as f32 / 32.0]);
            let height = self.height.sample([noise[0] as u64, noise[1] as u64]) % 32;

            if world_pos.y < height as i32 {
                *ret.get_block_mut(local_pos) = BlockId::Stone;
            }
        }

        ret
    }
}
