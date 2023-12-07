use bns_core::{BlockId, Chunk, ChunkPos, LocalPos};
use bns_rng::noises::SuperSimplex2;
use bns_rng::{FromRng, Noise};

use crate::biome::{Biome, BiomeId};
use crate::chunk_gen::PendingStructure;
use crate::column_gen::ColumnGen;
use crate::GenCtx;

#[derive(FromRng)]
pub struct Ocean {
    dirt_noise: SuperSimplex2,
    // negative value = deep ocean, positive value = shallow ocean
    floor_noise: SuperSimplex2,
}

impl Ocean {
    pub const FLOOR_SCALE: f32 = 1.0 / 200.0;
}

impl Biome for Ocean {
    fn height(&self, pos: glam::IVec2) -> f32 {
        let pos = pos.as_vec2();

        let floor_value = self.floor_noise.sample((Self::FLOOR_SCALE * pos).into());

        if floor_value < 0.0 {
            -30.0
        } else {
            -10.0
        }
    }

    fn build(&self, pos: ChunkPos, column: &ColumnGen, ctx: &GenCtx, chunk: &mut Chunk) {
        let biome_ids = &column.biome_stage(ctx).ids;

        for local_pos in LocalPos::iter_all() {
            if biome_ids[local_pos.into()] != BiomeId::Ocean {
                continue;
            }

            let world_pos = pos.origin() + local_pos.to_ivec3();
            let height = column.height_stage(ctx)[local_pos.into()];

            let gravel = bns_rng::utility::floor_i32(
                self.dirt_noise
                    .sample([world_pos.x as f32 / 8.0, world_pos.z as f32 / 8.0])
                    * 5.0
                    + 3.0,
            );

            if world_pos.y <= height {
                if world_pos.y >= -1 {
                    chunk.set_block(local_pos, BlockId::Sand.into());
                } else if world_pos.y < height - gravel {
                    chunk.set_block(local_pos, BlockId::Stone.into());
                } else {
                    chunk.set_block(local_pos, BlockId::Gravel.into());
                }
            } else if world_pos.y < 0 {
                chunk.set_block(local_pos, BlockId::Water.into());
            }
        }
    }

    fn register_structures(
        &self,
        pos: ChunkPos,
        column: &ColumnGen,
        ctx: &GenCtx,
        out: &mut Vec<PendingStructure>,
    ) {
        let _ = (pos, column, ctx, out);
    }

    fn debug_info(&self, w: &mut dyn std::fmt::Write, pos: glam::IVec3) -> std::fmt::Result {
        let _ = (w, pos);
        Ok(())
    }
}
