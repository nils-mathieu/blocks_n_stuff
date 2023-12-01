//! The standard world generator.

use glam::Vec3Swizzles;
use std::sync::Arc;

use bns_core::{BlockId, Chunk, LocalPos};
use bns_rng::{FromRng, Rng};

use crate::world::{ChunkPos, WorldGenerator};

mod climate;
pub use climate::*;

/// The state that's shared between all world generator clones (which are expected to work
/// together on multiple threads).
struct Shared {
    /// The climate generator.
    climate: ClimateGenerator,
}

/// The standard [`WorldGenerator`] implementation.
#[derive(Clone)]
pub struct StandardWorldGenerator {
    /// The state that's shared between all generators.
    shared: Arc<Shared>,
}

impl FromRng for StandardWorldGenerator {
    fn from_rng(rng: &mut impl Rng) -> Self {
        Self {
            shared: Arc::new(Shared {
                climate: ClimateGenerator::from_rng(rng),
            }),
        }
    }
}

impl WorldGenerator for StandardWorldGenerator {
    fn generate(&mut self, pos: ChunkPos) -> Chunk {
        let mut ret = Chunk::empty();

        for local_pos in LocalPos::iter_all() {
            let world_pos = pos * Chunk::SIDE + local_pos.to_ivec3();
            let climate = self.shared.climate.sample_climate(world_pos.xz());

            if world_pos.y <= climate.height {
                *ret.get_block_mut(local_pos) = match climate.height {
                    ..=-16 => BlockId::Gravel,
                    -15..=2 => BlockId::Sand,
                    3..=16 => BlockId::Grass,
                    _ => BlockId::Stone,
                };
            } else if world_pos.y <= 0 {
                *ret.get_block_mut(local_pos) = BlockId::Water;
            }
        }

        ret
    }
}
