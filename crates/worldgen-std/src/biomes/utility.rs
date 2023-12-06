use bns_core::{BlockId, Chunk, ChunkPos, LocalPos};
use bns_rng::noises::SuperSimplex2;
use bns_rng::Noise;

use crate::biome::BiomeId;
use crate::column_gen::ColumnGen;
use crate::GenCtx;

/// Information required to create a basic geological stage function for a biome.
pub struct BasicGeologicalStage<'a> {
    pub biome_filter: BiomeId,
    pub grass: BlockId,
    pub dirt: BlockId,
    pub min_dirt_depth: i32,
    pub max_dirt_depth: i32,
    pub dirt_noise: &'a SuperSimplex2,
}

impl BasicGeologicalStage<'_> {
    /// Executes a basic geological stage pass using the provided information.
    pub fn execute(self, pos: ChunkPos, column: &ColumnGen, ctx: &GenCtx, chunk: &mut Chunk) {
        let biome_ids = &column.biome_stage(ctx).ids;

        for local_pos in LocalPos::iter_all() {
            if biome_ids[local_pos.into()] != self.biome_filter {
                continue;
            }

            let world_pos = pos.origin() + local_pos.to_ivec3();
            let height = column.height_stage(ctx)[local_pos.into()];

            let dirt_depth = bns_rng::utility::floor_i32(
                self.dirt_noise
                    .sample([world_pos.x as f32 / 8.0, world_pos.z as f32 / 8.0])
                    * (self.max_dirt_depth - self.min_dirt_depth) as f32
                    + self.min_dirt_depth as f32,
            );

            if world_pos.y <= height {
                if world_pos.y < height - dirt_depth {
                    chunk.set_block(local_pos, BlockId::Stone.into());
                } else if world_pos.y <= 2 {
                    chunk.set_block(local_pos, BlockId::Sand.into());
                } else if world_pos.y < height {
                    chunk.set_block(local_pos, self.dirt.into());
                } else {
                    chunk.set_block(local_pos, self.grass.into());
                }
            } else if world_pos.y <= 0 {
                chunk.set_block(local_pos, BlockId::Water.into());
            }
        }
    }
}
