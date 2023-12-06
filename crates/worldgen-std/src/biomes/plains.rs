use bns_core::{AppearanceMetadata, BlockId, Chunk, ChunkPos, Face, LocalPos};
use bns_rng::noises::{Mixer, SuperSimplex2, SuperSimplex3};
use bns_rng::{FromRng, Noise};

use glam::IVec2;

use crate::biome::{Biome, BiomeId};
use crate::column_gen::ColumnGen;
use crate::GenCtx;

#[derive(FromRng)]
pub struct Plains {
    dirt_noise: SuperSimplex2,
    height_noise: [SuperSimplex2; 2],
    pebble_noise: Mixer<2>,
    daffodil_noise: Mixer<2>,
    diamond_noise: SuperSimplex3,
}

impl Plains {
    pub const HEIGHT_MAP_SCALE: f32 = 1.0 / 30.0;
    pub const HEIGHT_MAP_OFFSET: f32 = 5.0;
    pub const PEBBLE_PROBABILITY: u64 = 600;
    pub const DAFFODIL_PROBABILITY: u64 = 600;
}

impl Biome for Plains {
    fn height(&self, pos: IVec2) -> f32 {
        let pos = pos.as_vec2() * Self::HEIGHT_MAP_SCALE;

        let mut ret = Self::HEIGHT_MAP_OFFSET;

        ret += self.height_noise[0].sample(pos.into());
        ret += self.height_noise[1].sample((pos * 0.5).into()) * 0.5;

        ret
    }

    fn build(&self, pos: ChunkPos, column: &ColumnGen, ctx: &GenCtx, chunk: &mut Chunk) {
        let biome_ids = &column.biome_stage(ctx).ids;

        for local_pos in LocalPos::iter_all() {
            if biome_ids[local_pos.into()] != BiomeId::Plains {
                continue;
            }

            let world_pos = pos.origin() + local_pos.to_ivec3();
            let height = column.height_stage(ctx)[local_pos.into()];

            let dirt_depth = bns_rng::utility::floor_i32(
                self.dirt_noise
                    .sample([world_pos.x as f32 / 8.0, world_pos.z as f32 / 8.0])
                    * 2.0
                    + 3.0,
            );

            if world_pos.y <= height {
                if world_pos.y < height - dirt_depth {
                    if self.diamond_noise.sample([
                        world_pos.x as f32 / 8.0,
                        world_pos.y as f32 / 8.0,
                        world_pos.z as f32 / 8.0,
                    ]) > 0.7
                    {
                        chunk.set_block(local_pos, BlockId::DiamondOre.into());
                    } else {
                        chunk.set_block(local_pos, BlockId::Stone.into());
                    }
                } else if world_pos.y <= 2 {
                    chunk.set_block(local_pos, BlockId::Sand.into());
                } else if world_pos.y < height {
                    chunk.set_block(local_pos, BlockId::Dirt.into());
                } else {
                    chunk.set_block(local_pos, BlockId::Grass.into());
                }
            } else if world_pos.y <= 0 {
                chunk.set_block(local_pos, BlockId::Water.into());
            } else if world_pos.y == height + 1 {
                if self
                    .pebble_noise
                    .sample([world_pos.x as u64, world_pos.z as u64])
                    % Self::PEBBLE_PROBABILITY
                    == 0
                {
                    unsafe {
                        *chunk.get_block_mut(local_pos) = BlockId::Pebbles;
                        *chunk.get_appearance_mut(local_pos) = AppearanceMetadata { flat: Face::Y };
                    }
                }

                if self
                    .daffodil_noise
                    .sample([world_pos.x as u64, world_pos.z as u64])
                    % Self::DAFFODIL_PROBABILITY
                    == 0
                {
                    unsafe {
                        *chunk.get_block_mut(local_pos) = BlockId::Daffodil;
                        *chunk.get_appearance_mut(local_pos) = AppearanceMetadata { flat: Face::Y };
                    }
                }
            }
        }
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
