//! The standard world generator.

use bns_core::{BlockId, Chunk, ChunkPos, LocalPos};
use bns_rng::noises::Mixer;
use bns_rng::{FromRng, Rng};
use bns_worldgen_core::WorldGenerator;

use glam::Vec3Swizzles;

use biome::{BiomeId, BiomeRegistry};
use biomemap::BiomeMap;
use column_gen::{ColumnPos, Columns};

mod biome;
mod biomemap;
mod biomes;
mod chunk_gen;
mod column_gen;

/// Contains the context required to generate new chunks.
pub struct GenCtx {
    /// The map used to determine what biome should generate at a given position.
    pub biomes: BiomeMap,
    /// The registry of all biomes that can be generated.
    pub biome_registry: BiomeRegistry,
    /// The cache of new columns.
    pub columns: Columns,

    /// The noises used to randomly find samples in the biome map.
    pub heightmap_noises: [Mixer<2>; 8],
}

impl FromRng for GenCtx {
    fn from_rng(rng: &mut impl Rng) -> Self {
        Self {
            biomes: BiomeMap::from_rng(rng),
            biome_registry: BiomeRegistry::from_rng(rng),
            columns: Columns::default(),
            heightmap_noises: FromRng::from_rng(rng),
        }
    }
}

/// The standard [`WorldGenerator`] implementation.
#[derive(FromRng)]
pub struct StandardWorldGenerator {
    /// The context required to generate new chunks.
    ctx: GenCtx,
}

impl WorldGenerator for StandardWorldGenerator {
    fn generate(&self, chunk_pos: ChunkPos) -> Chunk {
        let mut ret = Chunk::empty();

        // only generate chunks betweens -4 and 4
        if chunk_pos.y < -4 || chunk_pos.y > 4 {
            return ret;
        }

        let col = self.ctx.columns.get(chunk_pos.xz());

        for local_pos in LocalPos::iter_all() {
            let world_pos = chunk_pos * Chunk::SIDE + local_pos.to_ivec3();

            let block = match col.biome_map(&self.ctx)[ColumnPos::from_local_pos(local_pos)] {
                BiomeId::Void => BlockId::Air,
                BiomeId::Plains => BlockId::Grass,
                BiomeId::OakForest => BlockId::Stone,
                BiomeId::Desert => BlockId::Sand,
                BiomeId::Ocean => BlockId::Gravel,
                BiomeId::PineForest => BlockId::Podzol,
            };

            let height = col.height_map(&self.ctx)[ColumnPos::from_local_pos(local_pos)] as i32;

            if world_pos.y == 0 {
                *ret.get_block_mut(local_pos) = block;
            }

            if world_pos.y <= height {
                *ret.get_block_mut(local_pos) = block;
            } else if world_pos.y <= 0 {
                *ret.get_block_mut(local_pos) = BlockId::Water;
            }
        }

        ret
    }
}
