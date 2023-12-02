use bns_core::{BlockId, Chunk};
use bns_rng::noises::Simplex2;
use bns_rng::FromRng;

use glam::IVec3;

use crate::biome::{Biome, BiomeId};
use crate::column_gen::ColumnGen;
use crate::GenCtx;

#[derive(FromRng)]
pub struct Plains {
    dirt_noise: Simplex2,
}

impl Biome for Plains {
    fn height(&self, _pos: glam::IVec2) -> f32 {
        8.0
    }

    fn geological_stage(&self, pos: IVec3, column: &ColumnGen, ctx: &GenCtx, chunk: &mut Chunk) {
        (super::utility::BasicGeologicalStage {
            biome_filter: BiomeId::Plains,
            min_dirt_depth: 3,
            max_dirt_depth: 5,
            grass: BlockId::Grass,
            dirt: BlockId::Dirt,
            stone: BlockId::Stone,
            dirt_noise: &self.dirt_noise,
        })
        .execute(pos, column, ctx, chunk);
    }

    fn debug_info(&self, buf: &mut String, pos: IVec3) {
        let _ = (buf, pos);
    }
}
