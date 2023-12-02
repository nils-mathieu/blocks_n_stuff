//! The standard world generator.

use bns_core::{BlockId, Chunk, ChunkPos, LocalPos};
use bns_rng::noises::Mixer;
use bns_rng::{FromRng, Rng};
use bns_worldgen_core::WorldGenerator;

use biome::{BiomeId, BiomeRegistry};
use biomemap::BiomeMap;
use column_gen::{ColumnGen, ColumnPos, ColumnState, Columns};
use glam::{IVec2, Vec3Swizzles};

mod biome;
mod biomemap;
mod biomes;
mod chunk_gen;
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

type ColumnGuard = parking_lot::lock_api::ArcRwLockReadGuard<parking_lot::RawRwLock, ColumnGen>;

impl GenCtx {
    /// The implementation of the function that do the following:
    ///
    /// 1. Acquire a read guard for a column.
    ///
    /// 2. Check if it's initialized up to `min_state`.
    ///
    /// 3. If it's not, upgrade the read guard to a write guard, and call `or_else` with the
    ///    write guard.
    ///
    /// 4. Downgrade the write guard to a read guard, and return it.
    ///
    /// # Remarks
    ///
    /// `or_else` is expected to initialize the column up to `min_state`, but it cannot assume
    /// that the state of the column is lower than `min_state` when it's called. Indeed, in case
    /// of a race to initialize the column between two threads, `or_else` may be called with a
    /// write guard that has already been initialized up to (or higher than) `min_state`. It is
    /// the responsability of `or_else` to check the state of the column before initializing it.
    fn get_column_ensure_impl(
        &self,
        pos: IVec2,
        min_state: ColumnState,
        or_else: impl FnOnce(&mut ColumnGen),
    ) -> ColumnGuard {
        let column = self.columns.get(pos);

        let mut guard = column.read_arc();
        if guard.state < min_state {
            drop(guard);
            let mut write_guard = column.write_arc();
            or_else(&mut write_guard);
            guard = parking_lot::lock_api::ArcRwLockWriteGuard::downgrade(write_guard);
        }

        guard
    }

    /// Returns a (potentially cached) column, making sure that the biome map for that column
    /// are generated.
    pub fn get_column_ensure_biomes(&self, pos: IVec2) -> ColumnGuard {
        self.get_column_ensure_impl(pos, ColumnState::Biomes, |column| {
            column.ensure_biomes(pos, self)
        })
    }

    /// Returns a (potentially cached) column, making sure that the heightmap for that column
    /// are generated.
    pub fn get_column_ensure_heightmap(&self, pos: IVec2) -> ColumnGuard {
        self.get_column_ensure_impl(pos, ColumnState::Heightmap, |column| {
            column.ensure_heightmap(pos, self)
        })
    }
}

/// The standard [`WorldGenerator`] implementation.
#[derive(FromRng)]
pub struct StandardWorldGenerator {
    /// The context required to generate new chunks.
    ctx: GenCtx,
}

impl WorldGenerator for StandardWorldGenerator {
    fn generate(&self, chunk_pos: ChunkPos) -> Chunk {
        let mut ret = Chunk::empty();

        // only generate chunks betweens -4 and 4
        if chunk_pos.y < -4 || chunk_pos.y > 4 {
            return ret;
        }

        let col = self.ctx.get_column_ensure_heightmap(chunk_pos.xz());

        for local_pos in LocalPos::iter_all() {
            let world_pos = chunk_pos * Chunk::SIDE + local_pos.to_ivec3();

            let block = match col.biome_map[ColumnPos::from_local_pos(local_pos)] {
                BiomeId::Void => BlockId::Air,
                BiomeId::Plains => BlockId::Grass,
                BiomeId::OakForest => BlockId::Stone,
                BiomeId::Desert => BlockId::Sand,
                BiomeId::Ocean => BlockId::Gravel,
                BiomeId::PineForest => BlockId::Podzol,
            };

            let height = col.height_map[ColumnPos::from_local_pos(local_pos)] as i32;

            if world_pos.y <= height {
                *ret.get_block_mut(local_pos) = block;
            } else if world_pos.y <= 0 {
                *ret.get_block_mut(local_pos) = BlockId::Water;
            }
        }

        ret
    }
}
