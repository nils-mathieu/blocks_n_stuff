use bns_core::{BlockId, Chunk, ChunkPos};
use bns_rng::noises::SuperSimplex2;
use bns_rng::FromRng;

use crate::biome::{Biome, BiomeId};
use crate::column_gen::ColumnGen;
use crate::GenCtx;

#[derive(FromRng)]
pub struct Desert {
    dirt_noise: SuperSimplex2,
}

impl Biome for Desert {
    fn height(&self, _pos: glam::IVec2) -> f32 {
        8.0
    }

    fn geological_stage(&self, pos: ChunkPos, column: &ColumnGen, ctx: &GenCtx, chunk: &mut Chunk) {
        (super::utility::BasicGeologicalStage {
            biome_filter: BiomeId::Desert,
            min_dirt_depth: 4,
            max_dirt_depth: 5,
            grass: BlockId::Sand,
            dirt: BlockId::Sand,
            dirt_noise: &self.dirt_noise,
        })
        .execute(pos, column, ctx, chunk);
    }

    fn debug_info(&self, w: &mut dyn std::fmt::Write, pos: glam::IVec3) -> std::fmt::Result {
        let _ = (w, pos);
        Ok(())
    }
}
