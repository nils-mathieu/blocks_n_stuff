use bns_core::{AppearanceMetadata, BlockId, Chunk, Face, LocalPos};
use bns_rng::noises::{Mixer, Simplex2};
use bns_rng::{FromRng, Noise};

use glam::IVec3;

use crate::biome::{Biome, BiomeId};
use crate::column_gen::ColumnGen;
use crate::GenCtx;

#[derive(FromRng)]
pub struct OakForest {
    dirt_noise: Simplex2,
    pebble_noise: Mixer<2>,
    daffodil_noise: Mixer<2>,
}

impl OakForest {
    pub const PEBBLE_PROBABILITY: u64 = 100;
    pub const DAFFODIL_PROBABILITY: u64 = 300;
}

impl Biome for OakForest {
    fn height(&self, _pos: glam::IVec2) -> f32 {
        14.0
    }

    fn geological_stage(&self, pos: IVec3, column: &ColumnGen, ctx: &GenCtx, chunk: &mut Chunk) {
        let biome_ids = &column.biome_stage(ctx).ids;

        for local_pos in LocalPos::iter_all() {
            if biome_ids[local_pos.into()] != BiomeId::OakForest {
                continue;
            }

            let world_pos = pos * Chunk::SIDE + local_pos.to_ivec3();
            let height = column.height_stage(ctx)[local_pos.into()];

            let dirt_depth = bns_rng::utility::floor_i32(
                self.dirt_noise
                    .sample([world_pos.x as f32 / 8.0, world_pos.z as f32 / 8.0])
                    * 2.0
                    + 3.0,
            );

            if world_pos.y <= height {
                if world_pos.y < height - dirt_depth {
                    chunk.set_block(local_pos, BlockId::Stone);
                } else if world_pos.y <= 2 {
                    chunk.set_block(local_pos, BlockId::Sand);
                } else if world_pos.y < height {
                    chunk.set_block(local_pos, BlockId::Dirt);
                } else {
                    chunk.set_block(local_pos, BlockId::Grass);
                }
            } else if world_pos.y <= 0 {
                chunk.set_block(local_pos, BlockId::Water);
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

    fn debug_info(&self, buf: &mut String, pos: IVec3) {
        let _ = (buf, pos);
    }
}
