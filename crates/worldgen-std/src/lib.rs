//! The standard world generator.

use bns_core::{BlockId, Chunk, ChunkPos, LocalPos};
use bns_rng::{FromRng, Noise};
use bns_worldgen_core::WorldGenerator;

use glam::Vec3Swizzles;

mod biome;
use biome::BiomeId;

mod biomemap;
use biomemap::BiomeMap;

/// The standard [`WorldGenerator`] implementation.
#[derive(Clone, FromRng)]
pub struct StandardWorldGenerator {
    /// The map used to determine what biome should generate at a given position.
    biomemap: BiomeMap,
}

impl WorldGenerator for StandardWorldGenerator {
    fn generate(&mut self, chunk_pos: ChunkPos) -> Chunk {
        let mut ret = Chunk::empty();

        if chunk_pos.y != 0 {
            return ret;
        }

        for local_pos in LocalPos::iter_surface(0) {
            let world_pos = chunk_pos * Chunk::SIDE + local_pos.to_ivec3();
            match self.biomemap.sample(world_pos.xz()) {
                BiomeId::Void => (),
                BiomeId::Plains => {
                    *ret.get_block_mut(local_pos) = BlockId::Grass;
                }
                BiomeId::Desert => {
                    *ret.get_block_mut(local_pos) = BlockId::Sand;
                }
                BiomeId::OakForest => {
                    *ret.get_block_mut(local_pos) = BlockId::Gravel;
                }
                BiomeId::PineForest => {
                    *ret.get_block_mut(local_pos) = BlockId::Diorite;
                }
                BiomeId::DeepOcean | BiomeId::ShallowOcean => {
                    *ret.get_block_mut(local_pos) = BlockId::Water;
                }
            }
        }

        ret
    }

    fn debug_info(&self, _buf: &mut String) {}
}
