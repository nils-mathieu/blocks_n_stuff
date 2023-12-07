use bns_core::{AppearanceMetadata, BlockId, Chunk, ChunkPos, Face, LocalPos};
use bns_rng::noises::{Mixer, SuperSimplex2, SuperSimplex3};
use bns_rng::{FromRng, Noise};

use glam::{IVec2, IVec3};

use crate::biome::{Biome, BiomeId};
use crate::chunk_gen::{PendingStructure, StructureTransformations};
use crate::column_gen::{ColumnGen, ColumnPos};
use crate::GenCtx;

use super::structures;

#[derive(FromRng)]
pub struct Plains {
    dirt_noise: SuperSimplex2,
    height_noise: [SuperSimplex2; 2],
    pebble_noise: Mixer<2>,
    daffodil_noise: Mixer<2>,
    diamond_noise: SuperSimplex3,
    tree_noise: Mixer<2>,
    tree_value: Mixer<2>,
    boulder_noise: Mixer<2>,
    boulder_value: Mixer<2>,
}

impl Plains {
    pub const HEIGHT_MAP_SCALE: f32 = 1.0 / 30.0;
    pub const HEIGHT_MAP_OFFSET: f32 = 5.0;
    pub const PEBBLE_PROBABILITY: u64 = 600;
    pub const DAFFODIL_PROBABILITY: u64 = 600;
    pub const TREE_PROBABILITY: u64 = 3000;
    pub const BOULDER_PROBABILITY: u64 = 5000;
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
            } else if world_pos.y == height + 1 && world_pos.y >= 4 {
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
        out: &mut Vec<PendingStructure>,
    ) {
        let origin = pos.origin();

        for local_pos in ColumnPos::iter_all() {
            if column.biome_stage(ctx).ids[local_pos] != BiomeId::Plains {
                continue;
            }

            let height = column.height_stage(ctx)[local_pos];

            if height < origin.y || height >= origin.y + Chunk::SIDE {
                continue;
            }

            if height < 4 {
                continue;
            }

            let world_pos = origin + IVec3::new(local_pos.x(), height, local_pos.z());

            if self
                .tree_noise
                .sample([world_pos.x as u64, world_pos.z as u64])
                % Self::TREE_PROBABILITY
                == 0
            {
                let value = self
                    .tree_value
                    .sample([world_pos.x as u64, world_pos.z as u64])
                    as usize;

                out.push(PendingStructure {
                    position: world_pos,
                    contents: structures::OAK_TREES[value % structures::OAK_TREES.len()].clone(),
                    transformations: StructureTransformations::IDENTITY,
                });
            }

            if self
                .boulder_noise
                .sample([world_pos.x as u64, world_pos.z as u64])
                % Self::BOULDER_PROBABILITY
                == 0
            {
                let value = self
                    .boulder_value
                    .sample([world_pos.x as u64, world_pos.z as u64])
                    as usize;

                out.push(PendingStructure {
                    position: world_pos,
                    contents: structures::BOULDERS[value % structures::BOULDERS.len()].clone(),
                    transformations: StructureTransformations::IDENTITY,
                });
            }
        }
    }

    fn debug_info(&self, w: &mut dyn std::fmt::Write, pos: glam::IVec3) -> std::fmt::Result {
        let _ = (w, pos);
        Ok(())
    }
}
