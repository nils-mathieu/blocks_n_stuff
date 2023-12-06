use bns_core::{BlockId, Chunk, ChunkPos};
use bns_rng::noises::SuperSimplex2;
use bns_rng::FromRng;

use crate::biome::{Biome, BiomeId};
use crate::column_gen::ColumnGen;
use crate::GenCtx;

#[derive(FromRng)]
pub struct PineForest {
    dirt_noise: SuperSimplex2,
}

impl Biome for PineForest {
    fn height(&self, _pos: glam::IVec2) -> f32 {
        25.0
    }

    fn build(&self, pos: ChunkPos, column: &ColumnGen, ctx: &GenCtx, chunk: &mut Chunk) {
        (super::utility::BasicGeologicalStage {
            biome_filter: BiomeId::PineForest,
            min_dirt_depth: 5,
            max_dirt_depth: 6,
            grass: BlockId::Podzol,
            dirt: BlockId::Dirt,
            dirt_noise: &self.dirt_noise,
        })
        .execute(pos, column, ctx, chunk);
    }

    fn register_structures(
        &self,
        pos: ChunkPos,
        column: &ColumnGen,
        ctx: &GenCtx,
        structures: &mut crate::structure::StructureRegistry,
    ) {
        let _ = (pos, column, ctx, structures);
    }

    fn debug_info(&self, w: &mut dyn std::fmt::Write, pos: glam::IVec3) -> std::fmt::Result {
        let _ = (w, pos);
        Ok(())
    }
}
