//! The standard world generator.

use bns_core::{BlockId, Chunk, ChunkPos, LocalPos};
use bns_rng::noises::Mixer;
use bns_rng::{FromRng, Rng};
use bns_worldgen_core::WorldGenerator;

use cache::Cache;
use glam::{IVec2, IVec3, Vec3Swizzles};

use biome::BiomeRegistry;
use biomemap::BiomeMap;
use column_gen::ColumnPos;
use parking_lot::RwLock;
use structure::StructureRegistry;

mod biome;
mod biomemap;
mod biomes;
mod cache;
mod chunk_gen;
mod column_gen;
mod structure;

/// Contains the context required to generate new chunks.
pub struct GenCtx {
    /// The map used to determine what biome should generate at a given position.
    pub biomes: BiomeMap,
    /// The registry of all biomes that can be generated.
    pub biome_registry: BiomeRegistry,
    /// The cache that stores the generation data to avoid having to recompute
    /// it constantly.
    pub cache: Cache,

    /// The noises used to randomly find samples in the biome map.
    pub heightmap_noises: [Mixer<2>; 8],
}

impl FromRng for GenCtx {
    fn from_rng(rng: &mut impl Rng) -> Self {
        Self {
            biomes: BiomeMap::from_rng(rng),
            biome_registry: BiomeRegistry::from_rng(rng),
            cache: Cache::default(),
            heightmap_noises: FromRng::from_rng(rng),
        }
    }
}

/// The standard [`WorldGenerator`] implementation.
pub struct StandardWorldGenerator {
    /// The context required to generate new chunks.
    ctx: GenCtx,
    /// The list of structures that can be generated.
    structures: RwLock<StructureRegistry>,
}

impl FromRng for StandardWorldGenerator {
    fn from_rng(rng: &mut impl Rng) -> Self {
        Self {
            ctx: GenCtx::from_rng(rng),
            structures: RwLock::new(StructureRegistry::default()),
        }
    }
}

impl WorldGenerator for StandardWorldGenerator {
    #[profiling::function]
    fn generate(&self, chunk_pos: ChunkPos) -> Chunk {
        let mut ret = Chunk::empty();

        // Only generate chunks betweens -4 and 4.
        if chunk_pos.y < -4 || chunk_pos.y > 4 {
            return ret;
        }

        let col = self.ctx.cache.get_column(chunk_pos.xz());
        for &biome in &col.biome_stage(&self.ctx).unique_biomes {
            self.ctx.biome_registry[biome]
                .implementation
                .build(chunk_pos, &col, &self.ctx, &mut ret);
        }
        self.structures.read().write_chunk(chunk_pos, &mut ret);

        // Add a layer of bedrock at the bottom of the world.
        if chunk_pos.y == -4 {
            for pos in LocalPos::iter_surface(0) {
                unsafe { *ret.get_block_mut(pos) = BlockId::Bedrock };
            }
        }

        ret
    }

    #[profiling::function]
    fn request_cleanup(&self, center: ChunkPos, h_radius: u32, v_radius: u32) {
        self.ctx.cache.request_cleanup(center, h_radius, v_radius);
    }

    fn debug_info(&self, w: &mut dyn std::fmt::Write, pos: IVec3) -> std::fmt::Result {
        self.ctx.biomes.debug_info(w, pos.xz())?;

        let col_pos = IVec2::new(pos.x.div_euclid(Chunk::SIDE), pos.z.div_euclid(Chunk::SIDE));
        let local_pos = ColumnPos::from_world_pos(pos.xz());
        let column = self.ctx.cache.get_column(col_pos);
        let biomes = column.biome_stage(&self.ctx);
        let biome = biomes.ids[local_pos];
        writeln!(w, "Biome: {:?}", biome)?;
        self.ctx.biome_registry[biome]
            .implementation
            .debug_info(w, pos)?;
        Ok(())
    }
}
