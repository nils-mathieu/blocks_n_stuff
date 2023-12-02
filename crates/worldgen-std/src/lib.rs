//! The standard world generator.

use bns_core::{BlockId, Chunk, ChunkPos, LocalPos};
use bns_rng::noises::Mixer;
use bns_rng::{FromRng, Rng};
use bns_worldgen_core::WorldGenerator;

use glam::Vec3Swizzles;

use biome::BiomeRegistry;
use biomemap::BiomeMap;
use column_gen::Columns;

mod biome;
mod biomemap;
mod biomes;
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
pub struct StandardWorldGenerator {
    /// The context required to generate new chunks.
    ctx: GenCtx,
}

impl FromRng for StandardWorldGenerator {
    fn from_rng(rng: &mut impl Rng) -> Self {
        Self {
            ctx: GenCtx::from_rng(rng),
        }
    }
}

impl WorldGenerator for StandardWorldGenerator {
    fn generate(&self, chunk_pos: ChunkPos) -> Chunk {
        let mut ret = Chunk::empty();

        // Only generate chunks betweens -4 and 4.
        if chunk_pos.y < -4 || chunk_pos.y > 4 {
            return ret;
        }

        let col = self.ctx.columns.get(chunk_pos.xz());
        for &biome in &col.biome_stage(&self.ctx).unique_biomes {
            self.ctx.biome_registry[biome]
                .implementation
                .geological_stage(chunk_pos, &col, &self.ctx, &mut ret);
        }

        // Add a layer of bedrock at the bottom of the world.
        if chunk_pos.y == -4 {
            for pos in LocalPos::iter_surface(0) {
                unsafe { *ret.get_block_mut(pos) = BlockId::Bedrock };
            }
        }

        ret
    }

    fn request_cleanup(&self, center: ChunkPos, h_radius: u32, _v_radius: u32) {
        self.ctx.columns.request_cleanup(center.xz(), h_radius);
    }

    fn debug_info(&self, buf: &mut String, pos: glam::IVec3) {
        let _ = (buf, pos);
    }
}
